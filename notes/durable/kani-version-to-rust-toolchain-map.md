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

[OBS 2026-08-07] Both files carry **uncommitted working-tree edits** bumping to
`=0.53.0`, adding a `cargo-kani setup` step to the Dockerfile, and deleting the
`rm -f rust-toolchain.toml` + `rustup override set 1.81.0` workaround from
`reframe_test.py::kani_build_tests.prepare_build` (that workaround existed to
stop the repo's toolchain pin colliding with Kani's; with 0.53.0 matching
exactly it should be redundant).

[OPEN] **None of that is verified.** Kani is not installed here, CI has never run
the benchmark step on dev18, and the edits' claim that 0.53.0 and
`nightly-2024-07-01` both report `rustc 1.81.0-nightly (6868c831a 2024-06-30)`
has not been checked in this workspace. Treat the removal of the workaround as
[HYP] until a benchmark run passes.

## Caveat

[FACT] Kani is not installed in this workspace, and neither is CBMC or ReFrame
(`which cargo-kani cbmc reframe` → all empty as of 2026-08-07). Running anything
Kani-side requires `cargo install kani-verifier --version 0.53.0` followed by
`cargo-kani setup`, which downloads a large bundle including CBMC.

## See also

- ../loose-ends/parked.md ("Upgrade kani and cross-check against SeaHorn")
- ../journal/2026-08/2026-08-07-ci-ported-to-dev18.md
