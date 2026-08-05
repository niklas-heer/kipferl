## PocketPy vendor patchset (Kipferl)

PocketPy is vendored as an amalgamated `pocketpy/vendor/pocketpy.c` + `pocketpy/vendor/pocketpy.h`.
For Kipferl CPython-compatibility, we maintain a small patchset that must be re-applied after updating PocketPy.

### Apply

From repo root:

```bash
./scripts/apply-pocketpy-patches.sh
```

The script is idempotent: it applies missing patches and skips patches that are already applied.

### Verify

```bash
python3 scripts/verify-pocketpy-patches.py
python3 scripts/verify-pocketpy-patches.py --check-upstream
```

Verification is marker-based (it checks for `kipferl patch:` anchors in `pocketpy/vendor/pocketpy.c`).
With `--check-upstream`, it also downloads pristine upstream `pocketpy.c` for the current `pocketpy/POCKETPY_VERSION` and asserts those anchors are not present there.
