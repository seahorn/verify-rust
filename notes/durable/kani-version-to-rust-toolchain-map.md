# Kani release → bundled Rust nightly

[FACT] Each Kani release pins the exact nightly it was built against, in
`rust-toolchain.toml` at the release tag. Kani ships and uses that toolchain for
codegen, so "which Kani works with our Rust" is answered by this file, not by
crates.io metadata (`kani-verifier` publishes no `rust_version` field).

Verify for any tag:

```bash
gh api "repos/model-checking/kani/contents/rust-toolchain.toml?ref=kani-<ver>" \
  -q .content | base64 -d | grep -oE 'nightly-[0-9-]+'
```

| Kani | released | bundled nightly |
|---|---|---|
| 0.43.0 | 2023-12 | `nightly-2023-12-14` |
| 0.52.0 | 2024-06-05 | `nightly-2024-05-28` |
| **0.53.0** | **2024-07-03** | **`nightly-2024-07-01`** |
| 0.54.0 | 2024-08-13 | `nightly-2024-08-07` |
| 0.55.0 | 2024-09-05 | `nightly-2024-09-03` |
| 0.56.0 | 2024-10-09 | `nightly-2024-10-03` |

(Verified 2026-08-07 against the tags above.)

## Why 0.53.0 is the one for this repo

[FACT] `rust-toolchain.toml` pins `channel = "nightly-2024-07-01"`. Kani 0.53.0
bundles **exactly** that nightly — an exact match, not merely a close one.

The committed pin is 0.43.0 in two places, carrying `nightly-2023-12-14`:
- `docker/verify-rust.Dockerfile` — `cargo install --locked kani-verifier
  --version ^0.43`, commented *"latest version to work with Rust 1.64"* (stale:
  that comment predates the dev18 port).
- `benchmark/reframe_test.py` — `kani_build_tests.KANI_VERSION = '0.43.0'`.

[SUPERSEDED 2026-08-12 — the edits described here are committed and verified]
~~[OBS 2026-08-07] Both files carry **uncommitted working-tree edits** bumping to
`=0.53.0` ... [OPEN] None of that is verified.~~

[OBS 2026-08-12] The bump is committed (`242e1db`) in both places, and the
`rm -f rust-toolchain.toml` + `rustup override set 1.81.0` workaround is gone
from `reframe_test.py::kani_build_tests.prepare_build`. **The removal is
verified, not hypothesised**: a full 304-case benchmark run codegened and
verified 151 harnesses with `rust-toolchain.toml` in place and no rustup
override. → ../journal/2026-08/2026-08-12-first-local-benchmark-run-on-dev18.md

[FACT] `--locked` is required on `cargo install kani-verifier`, not cosmetic.
Without it cargo re-resolves to `home 0.5.12`, which needs `edition2024`
(cargo >= 1.85) and fails to parse under this 1.81 toolchain. The comment at
`reframe_test.py:48-52` records this.

## Caveat

[SUPERSEDED 2026-08-12] ~~Kani is not installed in this workspace, and neither is
CBMC or ReFrame.~~ Kani 0.53.0 and its CBMC bundle are installed
(`~/.kani/kani-0.53.0`, `cargo-kani --version` → 0.53.0). ReFrame is still not on
the host and must go in a venv — PEP 668 blocks `pip install --user` on 24.04.
→ running-the-benchmark-locally.md

## See also

- ../loose-ends/parked.md ("Upgrade kani and cross-check against SeaHorn")
- ../journal/2026-08/2026-08-07-ci-ported-to-dev18.md
