# Compatibility priorities after the first language patches

The September 5, 2026 rerun uses the same [August popularity ranking](popularity.json), the same 999 available release metadata pins, and the patched macOS ARM64 runtime `1750884ef55d811fa1c548518301adc9a7ad04d334891ab665ec532f660cc676`. ddtrace remains metadata-limited. See the [before/after comparison](language-patch-comparison.md) and [current evidence](popularity-audit.json).

## Completed work and measured effect

- Trailing commas in parenthesized imports and function/lambda parameters now work. The initial audit's largest diagnostic group contained 292 import cases and 30 function signatures, all with a trailing comma before the closing parenthesis.
- Adjacent plain strings and bytes now concatenate correctly, including comments, bracketed newlines, defaults, and embedded NUL bytes. Adjacent f-string combinations remain unsupported.
- Syntax checks use nonexecuting normal module compilation. The original dynamic `compile()` checker produced nine `global` diagnostics that were not evidence of missing normal-module language support. Generated application/module wrappers now use the same compilation semantics when they execute.

| Measure | Before | After |
| --- | ---: | ---: |
| Syntax-blocked releases | 782 | 774 |
| Compilation-complete distributions, still unverified | 32 | 40 |
| Compilation-complete distributions containing Python source | 12 | 20 |
| Source files checked | 1,546 | 2,026 |

Eight additional releases finish source compilation: certifi, llama-parse, rfc3986-validator, rfc3987-syntax, llama-index-indices-managed-llama-cloud, alabaster, opentelemetry-distro, and sphinxcontrib-jsmath. This is not proof that their imports, dependencies, or main APIs work.

The first blocker changed in 391 packages. Of these, 383 still stop at another source location or diagnostic. These counts compare source files, final line numbers, and SyntaxError messages rather than the changed checker traceback wrapper.

## Next investigations

The most common remaining diagnostic, `expected statement end`, accounts for 224 packages. Source inspection separates it into 170 dotted imports, 28 async declarations, 24 exception-chaining cases, and two other constructs. Treating this message as one missing feature would misdirect the next patch.

| Construct or diagnostic group | Packages hitting it first | Implementation considerations |
| --- | ---: | --- |
| Dotted imports such as `import os.path` | 170 | Preserve parent initialization order and root-versus-alias bindings, alongside parsing. |
| Generator-expression diagnostic `expected ')', got 'for'` | 69 | Requires lazy evaluation, scope/capture semantics, filters, and cleanup; existing eager comprehensions are insufficient. |
| String escapes | 54 | Separate escaped newlines, Unicode and octal escapes, and invalid/unknown escape handling. |
| Nonliteral-default diagnostic `expected a literal, got '@id'` | 53 | Evaluate defaults when the function is defined and retain them per function. |
| Parameter diagnostic `expected '@id', got ','` | 47 | Investigate keyword-only separators and argument binding with small reproducers. |

The next focused parser investment is dotted imports. Before calling any additional package supported, resolve the dependencies of promising compilation-complete candidates and run reviewed import/API tests. Some candidates may depend on packages already blocked by native libraries, version requirements, or missing APIs.

Download counts include automation and transitive dependencies; they are a prioritization signal, not unique users or package quality. Each result covers one selected release/artifact, one runtime hash, and one target. The screen stops at the first compiler failure, so counts describe currently visible blockers rather than all blockers in each package.

Use `kipferl deps audit --limit 100`, the website's package-audit filters, or [the CSV](popularity-audit.csv) to inspect individual evidence. `mise run catalog-check` validates the stored report offline. `mise run package-audit` builds and screens the runtime embedded for the current host, using policy-bound checkpoints. Reuse explicit metadata pins when comparing runtimes so a new upstream release does not masquerade as a compiler improvement.
