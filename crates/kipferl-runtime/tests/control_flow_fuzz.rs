//! Differential fuzzing of nested try/except/finally, with and loop statements.
//!
//! Each case is a function built from random nested control flow with a fixed
//! seed. Kipferl compiles every finally clause once per way out of its try
//! statement and must keep the value stack balanced on each of them, so the
//! cases and their traces must match the reference interpreter line for line.
use std::fmt::Write as _;
use std::io;
use std::process::{Command, Output};

const SEED: u64 = 0x6b69_7066_6572_6c21;
const CASES: usize = 1000;
const CASES_PER_PROGRAM: usize = 40;
/// The chance of a compound statement at each nesting depth; deeper ones are simple.
const COMPOUND_PERCENT: [u64; 4] = [80, 70, 50, 30];
/// Every exit compiles its finally clauses again, so large cases outgrow the 16-bit jumps.
const MAX_CASE_LINES: usize = 50;

const PRELUDE: &str = r#"
log = []

class E(Exception):
    pass

class CM:
    def __init__(self, n, suppress):
        self.n = n
        self.suppress = suppress
    def __enter__(self):
        log.append("enter%d" % self.n)
        return self
    def __exit__(self, kind, value, traceback):
        log.append("exit%d:%s" % (self.n, kind.__name__ if kind else None))
        return self.suppress

def run(number, function):
    log.clear()
    try:
        result = function()
    except Exception as error:
        result = ("raised", type(error).__name__, error.args)
    print(number, repr(result), log)
"#;

/// A small deterministic generator: the cases must not change between runs.
struct Rng(u64);

impl Rng {
    const fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next().checked_rem(bound).unwrap_or(0)
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

#[derive(Clone, Copy, Default)]
struct Scope {
    depth: usize,
    /// The innermost loop variable, if the statement is inside a loop.
    loop_var: Option<usize>,
    /// The statement can re-raise with a bare `raise`.
    in_handler: bool,
    /// Exits from finally clauses supersede pending ones: generate more of them.
    in_finally: bool,
}

struct Generator {
    rng: Rng,
    source: String,
    marker: u64,
}

impl Generator {
    fn line(&mut self, indent: usize, text: &str) {
        for _ in 0..indent {
            self.source.push_str("    ");
        }
        self.source.push_str(text);
        self.source.push('\n');
    }

    const fn marker(&mut self) -> u64 {
        self.marker = self.marker.wrapping_add(1);
        self.marker
    }

    fn block(&mut self, indent: usize, scope: Scope) {
        let least = if scope.in_finally { 2 } else { 1 };
        let statements = self.rng.below(2).wrapping_add(least);
        for _ in 0..statements {
            self.statement(indent, scope);
        }
    }

    fn finally_clause(&mut self, indent: usize, scope: Scope) {
        let clause = Scope {
            in_finally: true,
            ..scope
        };
        self.block(indent, clause);
    }

    fn exit(&mut self, indent: usize, scope: Scope) {
        let marker = self.marker();
        // a jump out of a finally clause supersedes the exit that runs the clause
        let raise_percent = if scope.in_finally { 15 } else { 40 };
        let text = if self.rng.chance(raise_percent) {
            match self.rng.below(5) {
                // outside a handler, CPython raises RuntimeError
                0 if scope.in_handler || scope.in_finally || self.rng.chance(20) => {
                    "raise".to_owned()
                }
                0..=1 => format!("raise ValueError({marker})"),
                _ => format!("raise E({marker})"),
            }
        } else {
            match (self.rng.below(4), scope.loop_var) {
                (0, Some(_)) => "break".to_owned(),
                (1, Some(_)) => "continue".to_owned(),
                _ => format!("return {marker}"),
            }
        };
        self.line(indent, &text);
    }

    fn statement(&mut self, indent: usize, scope: Scope) {
        let compound_percent = COMPOUND_PERCENT.get(scope.depth).copied().unwrap_or(0);
        if !self.rng.chance(compound_percent) {
            if self.rng.chance(if scope.in_finally { 5 } else { 20 }) {
                let marker = self.marker();
                self.line(indent, &format!("log.append({marker})"));
            } else if self.rng.chance(50) {
                let condition = scope.loop_var.map_or_else(
                    || "len(log) % 2 == 1".to_owned(),
                    |var| format!("i{var} == {}", self.rng.below(3)),
                );
                self.line(indent, &format!("if {condition}:"));
                self.exit(indent.wrapping_add(1), scope);
            } else {
                self.exit(indent, scope);
            }
            return;
        }
        let depth = scope.depth.wrapping_add(1);
        let inner = Scope { depth, ..scope };
        let body = indent.wrapping_add(1);
        match self.rng.below(10) {
            0..=1 => {
                let count = self.rng.below(3).wrapping_add(1);
                self.line(indent, &format!("for i{depth} in range({count}):"));
                let looped = Scope {
                    loop_var: Some(depth),
                    ..inner
                };
                self.block(body, looped);
            }
            2 => {
                self.line(indent, &format!("i{depth} = 0"));
                self.line(indent, &format!("while i{depth} < 2:"));
                self.line(body, &format!("i{depth} += 1"));
                let looped = Scope {
                    loop_var: Some(depth),
                    ..inner
                };
                self.block(body, looped);
            }
            3 => {
                let marker = self.marker();
                let suppress = if self.rng.chance(30) { "True" } else { "False" };
                self.line(indent, &format!("with CM({marker}, {suppress}):"));
                self.block(body, inner);
            }
            4..=7 => {
                self.line(indent, "try:");
                self.block(body, inner);
                self.line(indent, "finally:");
                self.finally_clause(body, inner);
            }
            kind => {
                self.line(indent, "try:");
                self.block(body, inner);
                let caught = if self.rng.chance(70) {
                    "E"
                } else {
                    "Exception"
                };
                self.line(indent, &format!("except {caught} as error:"));
                let marker = self.marker();
                self.line(body, &format!("log.append(({marker}, error.args))"));
                let handler = Scope {
                    in_handler: true,
                    ..inner
                };
                self.block(body, handler);
                if kind == 9 || self.rng.chance(50) {
                    self.line(indent, "else:");
                    self.block(body, inner);
                }
                if kind == 9 {
                    self.line(indent, "finally:");
                    self.finally_clause(body, inner);
                }
            }
        }
    }

    fn case(&mut self, number: usize) -> String {
        let start = self.source.len();
        loop {
            self.line(0, &format!("def case{number}():"));
            self.block(1, Scope::default());
            self.line(1, "return 'end'");
            self.line(0, &format!("run({number}, case{number})"));
            let case = self.source.get(start..).unwrap_or_default().to_owned();
            if case.lines().count() <= MAX_CASE_LINES {
                return case;
            }
            self.source.truncate(start);
        }
    }
}

fn run(program: &str, source: &str) -> io::Result<Output> {
    Command::new(program).args(["-c", source]).output()
}

fn check(condition: bool, message: impl std::fmt::Display) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message.to_string()))
    }
}

#[test]
fn nested_exits_match_cpython() -> io::Result<()> {
    let mut generator = Generator {
        rng: Rng(SEED),
        source: String::new(),
        marker: 0,
    };
    let mut first = 0;
    while first < CASES {
        generator.source = PRELUDE.to_owned();
        let last = first.saturating_add(CASES_PER_PROGRAM).min(CASES);
        let cases: Vec<String> = (first..last).map(|number| generator.case(number)).collect();
        let source = generator.source.clone();
        let expected = run("python3", &source)?;
        let actual = run(env!("CARGO_BIN_EXE_pocketpy-kipferl"), &source)?;
        check(
            expected.status.success(),
            format!(
                "CPython failed:\n{}",
                String::from_utf8_lossy(&expected.stderr)
            ),
        )?;
        let expected_text = String::from_utf8_lossy(&expected.stdout);
        let actual_text = String::from_utf8_lossy(&actual.stdout);
        let mut actual_lines = actual_text.lines();
        for (case, expected_line) in cases.iter().zip(expected_text.lines()) {
            let actual_line = actual_lines.next();
            if actual_line != Some(expected_line) {
                let mut message = String::new();
                let _ = writeln!(message, "case differs from CPython:\n{case}");
                let _ = writeln!(message, "--- CPython\n{expected_line}");
                let _ = writeln!(
                    message,
                    "--- Kipferl\n{}\n{}",
                    actual_line.unwrap_or("<no output>"),
                    String::from_utf8_lossy(&actual.stderr)
                );
                return Err(io::Error::other(message));
            }
        }
        check(
            actual.status.success() && actual.stderr.is_empty(),
            format!(
                "Kipferl failed:\n{}",
                String::from_utf8_lossy(&actual.stderr)
            ),
        )?;
        first = last;
    }
    Ok(())
}
