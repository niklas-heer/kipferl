# Package audit after the first language patches

The runtime adds trailing commas in parenthesized imports/function signatures and adjacent plain string/bytes literals. The audit now compiles in normal module mode without executing package source.

Compared 1000 ranked projects on `macos-aarch64`. 999 reused identical pinned metadata, releases, and selected artifacts.

| Result | Before: top 100 | After: top 100 | Before: top 1,000 | After: top 1,000 |
| --- | ---: | ---: | ---: | ---: |
| limits | 0 | 0 | 2 | 2 |
| native_only | 15 | 15 | 178 | 178 |
| python_requirement | 0 | 0 | 5 | 5 |
| source_only | 0 | 0 | 1 | 1 |
| syntax | 82 | 81 | 782 | 774 |
| unverified | 3 | 4 | 32 | 40 |

8 previously syntax-blocked releases now complete source compilation. The new report contains 40 compilation-complete distributions, of which 20 contain Python source. These remain **unverified** until imports, dependencies, and behavior are tested.

The original global-statement diagnostics included a checker limitation: dynamic `compile()` rejected constructs accepted during normal module compilation. Improvements from correcting that checker must not all be attributed to parser patches.

The first blocker changed in 391 packages, including 383 that remain syntax-blocked at another source location or diagnostic. This comparison uses the source file, final source line number, and SyntaxError message; it ignores the changed checker traceback wrapper.

Baseline metadata was unavailable for: ddtrace. These rows cannot establish an identical-release comparison.

Before runtime: `a5a8337e08f25d85719fda9ab717ccad2f383c1ba51e0d55bfbb5c3403f9f56a`. After runtime: `1750884ef55d811fa1c548518301adc9a7ad04d334891ab665ec532f660cc676`.

See [the comparison JSON](language-patch-comparison.json) for every transition and exact report/policy hashes, and [the current audit](popularity-audit.json) for complete current evidence.
