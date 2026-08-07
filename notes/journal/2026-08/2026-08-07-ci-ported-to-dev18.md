# CI ported to dev18; first green-path run

The port landed on `main` (PR #8, rebase-merged) with CI still building against
the dev14 image. Fixed, and the first real CI run followed.

## The gap

[OBS 2026-08-07] The port moved the tree to LLVM 18 / `nightly-2024-07-01` but
touched neither the Dockerfile nor the workflow. `git diff main..HEAD --
.github/ docker/` was two blacklist lines. CI would have failed at configure
time: `CMakeLists.txt` looks for `llvm-nm-18` / `llvm-link-18` / `lld-18`
(lines 29-39), while the workflow passed `-DCMAKE_C_COMPILER=clang-14` against
`SEAHORN_ROOT=/home/usea/seahorn` from `seahorn/seahorn-llvm14:nightly`.

## The fix

[FACT] The dev18 image is `ghcr.io/seahorn/seahorn-llvm18:nightly` — note the
registry moves from Docker Hub to **GHCR**. It is public; verify with an
anonymous pull token:

```bash
TOKEN=$(curl -s "https://ghcr.io/token?scope=repository:seahorn/seahorn-llvm18:pull&service=ghcr.io" \
  | python3 -c 'import sys,json;print(json.load(sys.stdin)["token"])')
curl -s -o /dev/null -w "%{http_code}\n" -H "Authorization: Bearer $TOKEN" \
  -H "Accept: application/vnd.oci.image.index.v1+json" \
  https://ghcr.io/v2/seahorn/seahorn-llvm18/manifests/nightly     # 200
```

→ Reference for the whole dev18 CI shape:
`github.com/seahorn/verify-c-common/blob/master/docker/verify-c-common.Dockerfile`

Changes in commit `295a344`:
- `docker/verify-rust.Dockerfile` — GHCR dev18 base; `rustup install` /
  `component add rust-src` / `cargo install cbindgen` moved to
  `nightly-2024-07-01` to match `rust-toolchain.toml`. cbindgen must track the
  toolchain because the real build uses it (`CMakeLists.txt`,
  `cmake/SeaHornBuild.cmake`).
- `.github/workflows/main.yml` — `clang-18`/`clang++-18` in **two** places: the
  `run seahorn proofs` step and the `run benchmarks` step.

`-DSEA_LINK` is not passed; `CMakeLists.txt:33` already defaults to
`llvm-link-18`.

## First run

[OBS 2026-08-07] Run `31130269881` on `main`: `Build proofs` **succeeded** — the
dev18 image, toolchain and cbindgen all work — and 210 tests configured and ran
under `clang-18`. Result **199 passed / 11 failed / 0 timeouts** out of 210
(213 minus the 3 blacklisted). Nothing timed out under CI's 2000s cap, versus 9
timeouts on the stricter local 400s run.

The 11 failures were exactly the Class 1 set, since fixed —
→ 2026-08-07-rustc-folds-assertions-under-llvm18.md

[OPEN] The `run benchmarks` step is skipped when ctest fails, so it has **never
executed on dev18**. The kani pin is still `^0.43` in the Dockerfile. First run
of that step is the next place to expect trouble.
→ ../../loose-ends/parked.md ("Upgrade kani and cross-check against SeaHorn")

## Diagnostic trap worth remembering

[OBS 2026-08-07] PR #8 produced **no workflow run at all** — not a pending
`action_required` run, nothing. Hours were spent ruling out trigger config
(correct and unchanged since 2025-05-08), fork policy (PR #7 was also cross-repo
from the same fork and did run), repo Actions settings, and org/billing policy.

The actual cause was a **GitHub Actions major outage** (incident opened
2026-08-06T15:22Z, ~7h before the PR). Confirmed by a direct push to `main`
matching `push: branches: main` also producing nothing.

[FACT] The tell is *zero run creation* rather than failing or queued runs. Check
the platform before the configuration:

```bash
curl -s https://www.githubstatus.com/api/v2/summary.json \
  | python3 -c "import sys,json;d=json.load(sys.stdin);print(d['status']['description']);\
[print(' ',c['name'],c['status']) for c in d['components'] if c['status']!='operational']"
```

Also worth knowing for this repo: `push: branches: main` means **only** pushes to
`main` fire. Pushing a feature branch — to the fork or to upstream — never
triggers CI; the PR does.

## See also

- ../../durable/dev14-to-dev18-port-record.md
- 2026-08-07-rustc-folds-assertions-under-llvm18.md
