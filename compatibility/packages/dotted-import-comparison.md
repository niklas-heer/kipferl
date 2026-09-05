# Package audit after native dotted imports

The runtime now parses dotted imports and implements parent initialization, root and alias binding, circular imports, and failed-import cleanup. Both screens use nonexecuting normal module compilation.

Compared 1000 ranked projects on `macos-aarch64`. 999 reused identical pinned metadata, releases, and selected artifacts.

| Result | Before: top 100 | After: top 100 | Before: top 1,000 | After: top 1,000 |
| --- | ---: | ---: | ---: | ---: |
| limits | 0 | 0 | 2 | 2 |
| native_only | 15 | 15 | 178 | 178 |
| python_requirement | 0 | 0 | 5 | 5 |
| source_only | 0 | 0 | 1 | 1 |
| syntax | 81 | 81 | 774 | 770 |
| unverified | 4 | 4 | 40 | 44 |

4 previously syntax-blocked releases now complete source compilation. The new report contains 44 compilation-complete distributions, of which 24 contain Python source. These remain **unverified** until imports, dependencies, and behavior are tested.

The first blocker changed in 170 packages, including 166 that remain syntax-blocked at another source location or diagnostic. This comparison uses the source file, final source line number, and SyntaxError message; it ignores checker traceback wrapper differences.

Newly compilation-complete releases: entrypoints==0.4, pytest-metadata==3.1.1, zope-event==6.2, python-http-client==3.3.7.

Baseline metadata was unavailable for: ddtrace. These rows cannot establish an identical-release comparison.

Before runtime: `1750884ef55d811fa1c548518301adc9a7ad04d334891ab665ec532f660cc676`. After runtime: `5797c5f7ff3779270ef3f41b05088f6f08f92817e9a2be24343b4eed7736ca76`.

See [the comparison JSON](dotted-import-comparison.json) for every transition and exact report/policy hashes, and [the current audit](popularity-audit.json) for complete current evidence.
