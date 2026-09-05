# Package audit refreshed for Kipferl 0.7.2

Fresh source compilation on the published 0.7.2 macOS ARM64 runtime using the same available pinned releases and artifacts as the earlier dotted-import audit. The intervening changes include the completed Kipferl rename; this comparison measures their actual effect rather than assuming new compatibility.

Compared 1000 ranked projects on `macos-aarch64`. 999 reused identical pinned metadata, releases, and selected artifacts.

| Result | Before: top 100 | After: top 100 | Before: top 1,000 | After: top 1,000 |
| --- | ---: | ---: | ---: | ---: |
| limits | 0 | 0 | 2 | 2 |
| native_only | 15 | 15 | 178 | 178 |
| python_requirement | 0 | 0 | 5 | 5 |
| source_only | 0 | 0 | 1 | 1 |
| syntax | 81 | 81 | 770 | 770 |
| unverified | 4 | 4 | 44 | 44 |

0 previously syntax-blocked releases now complete source compilation. The new report contains 44 compilation-complete distributions, of which 24 contain Python source. These remain **unverified** until imports, dependencies, and behavior are tested.

The first blocker changed in 0 packages, including 0 that remain syntax-blocked at another source location or diagnostic. This comparison uses the source file, final source line number, and SyntaxError message; it ignores checker traceback wrapper differences.

Baseline metadata was unavailable for: ddtrace. These rows cannot establish an identical-release comparison.

Before runtime: `5797c5f7ff3779270ef3f41b05088f6f08f92817e9a2be24343b4eed7736ca76`. After runtime: `1f54af5ee829e94d74e928c9317b12ed3be304c0aeab087dfd50283a5d3dbfbd`.

See [the comparison JSON](release-0.7.2-comparison.json) for every transition and exact report/policy hashes, and [the current audit](popularity-audit.json) for complete current evidence.
