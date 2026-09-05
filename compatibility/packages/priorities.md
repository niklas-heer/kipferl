# Compatibility priorities after the 0.7.2 re-audit

The 0.7.2 rerun covers the same [August popularity ranking](popularity.json) and the same 999 available release metadata pins; ddtrace remains metadata-limited. It confirms the previous results: 770 syntax blockers and 44 compilation-complete distributions, including 24 with Python source and 20 without. No category or first compiler blocker changed. These remain unverified candidates, not approved working libraries. See the [0.7.2 comparison](release-0.7.2-comparison.md) and [current evidence](popularity-audit.json) for exact runtime identities.

The historical September 5, 2026 [first language-patch comparison](language-patch-comparison.md) and [dotted-import comparison](dotted-import-comparison.md) preserve the earlier improvements below. The refreshed report keeps those priorities relevant; it does not resolve or test the dependency closure or application behavior of the 1,000 projects.

## Completed work and measured effect

- Trailing commas in parenthesized imports and function/lambda parameters now work. The initial audit's largest diagnostic group contained 292 import cases and 30 function signatures, all with a trailing comma before the closing parenthesis.
- Adjacent plain strings and bytes concatenate correctly, including comments, bracketed newlines, defaults, and embedded NUL bytes. Adjacent f-string combinations remain unsupported.
- Syntax checks use nonexecuting normal module compilation. The original dynamic `compile()` checker produced nine `global` diagnostics that were not evidence of missing normal-module language support. Generated application/module wrappers use the same compilation semantics when they execute.
- Native dotted imports initialize parents first, bind roots or aliases, and preserve cache identity through cycles and failures. Failed attempts release their cache entry while retained functions/classes keep their own globals. Package metadata, relative from-imports, reexports, attribute shadowing, and package reload have CPython-oracle tests. Bundled applications preserve the same alias behavior after the source tree is removed.

| Measure | Original | First patches | Dotted imports |
| --- | ---: | ---: | ---: |
| Syntax-blocked releases | 782 | 774 | 770 |
| Compilation-complete distributions, still unverified | 32 | 40 | 44 |
| Compilation-complete distributions containing Python source | 12 | 20 | 24 |
| Source files checked | 1,546 | 2,026 | 2,200 |

The first patches added eight compilation-complete candidates: certifi, llama-parse, rfc3986-validator, rfc3987-syntax, llama-index-indices-managed-llama-cloud, alabaster, opentelemetry-distro, and sphinxcontrib-jsmath. Dotted imports add four: entrypoints 0.4, pytest-metadata 3.1.1, zope-event 6.2, and python-http-client 3.3.7. None of the prior compilation-complete candidates regressed. This is not proof that imports, dependencies, or main APIs work.

All 170 releases whose first blocker was a dotted import progress; 166 stop at a later source location or diagnostic. Top-100 categories remain unchanged: 81 syntax blockers, 15 native-wheel constraints, and four compilation-complete distributions. The comparison uses source files, final line numbers, and SyntaxError messages, ignoring traceback wrappers.

## Next investigations

| Construct or diagnostic group | Packages hitting it first | Implementation considerations |
| --- | ---: | --- |
| Generator-expression diagnostic `expected ')', got 'for'` | 89 | Requires lazy evaluation, scope/capture semantics, filters, and cleanup; existing eager comprehensions are insufficient. |
| Nonliteral-default diagnostic `expected a literal, got '@id'` | 69 | Evaluate defaults when the function is defined and retain them per function. Other default-expression diagnostics also need source inspection. |
| `expected statement end` | 65 | Dotted imports are removed from this bucket. Inspect remaining async, exception-chaining, and other constructs separately. |
| Parameter diagnostic `expected '@id', got ','` | 62 | Investigate keyword-only separators and argument binding with small reproducers. |
| String escapes | 57 | Separate escaped newlines, Unicode and octal escapes, and invalid/unknown escape handling. |

These are first-diagnostic counts, not complete feature inventories or estimates of fully working packages. Newly exposed failures raise some buckets even though no package regressed. Before calling a candidate supported, resolve its dependencies and run reviewed import/API tests. Native libraries, version requirements, and missing APIs can still block a source-compatible package.

Source inspection and minimal compile-only probes also identify implicit continuation gaps: `(1 + 2)` works on one line but fails when the newline precedes `+`; the same happens with `and`. Ordinary assignments to `match` expose a soft-keyword gap. Escape handling is incomplete rather than uniformly absent: some octal bytes compile while others do not. These deserve separate small reproducers before choosing the next patch; their package counts overlap mixed diagnostic buckets and are not yet quantified by construct.

Dynamic `__import__` supports absolute positional fromlists; nonzero levels remain unsupported, while relative from-import statements work. `sys.modules` mirrors loading and successful modules but is not an authoritative user-mutable cache. Namespace packages, custom import finders, and custom `__path__` searches remain outside this patch.

Download counts include automation and transitive dependencies; they are a prioritization signal, not unique users or package quality. Each result covers one selected release/artifact, one runtime hash, and one target.

Use `kipferl deps audit --limit 100`, the website's package-audit filters, or [the CSV](popularity-audit.csv) to inspect individual evidence. `mise run catalog-check` validates the stored report offline. `mise run package-audit` builds and screens the runtime embedded for the current host, using policy-bound checkpoints. Reuse explicit metadata pins when comparing runtimes so a new upstream release does not masquerade as a compiler improvement.
