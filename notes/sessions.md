# Sessions log

Chronological index of significant sessions and what they produced. Newest first.

## 2026-08-12 — First full benchmark run on dev18 (both tools, 304/304)

**Session:** `llvm18-port` (session `10a63551`, continued).
**Theme:** Run the ReFrame benchmark end to end on dev18 — it had never
completed on this toolchain, in CI or locally.

**Key outputs:**
- 304/304 cases passed (151 pairs x 2 tools + 2 fixtures), 80 min, serial. So
  `all_benchmarks.txt` (`cadd21b`) is confirmed accurate.
- journal/2026-08/2026-08-12-first-local-benchmark-run-on-dev18.md — results and
  the finding that seabmc's 8.9x median lead is per-invocation overhead, not
  solving power (total solver time 325.9s vs 325.5s, a 0.1% difference).
- durable/running-the-benchmark-locally.md — the four things the host does not
  provide, and the reproducible recipe.
- durable/building-llvm-stack-on-ubuntu-24.04.md §1b — the `-lstdc++` failure
  recurs on clang-18; `--gcc-install-dir` is available there (unlike clang-14),
  but installing `libstdc++-14-dev` would likely break the coexisting dev14 build.
- `benchmark/benchmark_postprocess.csv` regenerated from this run.

**Critical corrections:**
- I recommended `sudo apt install libstdc++-14-dev` to the user as the permanent
  fix before reading durable/building-llvm-stack-on-ubuntu-24.04.md §1, which
  documents that exact package as a trap for clang-14. The advice turned out to
  be right — no `build-14` is needed on this box — but I did not know that when I
  gave it, and the notes I had already read said otherwise. Right answer, no
  grounds for it.
- Two launches were wasted diagnosing "seahorn failures" that were environment
  faults (unlinkable clang++-18, unset `LD_LIBRARY_PATH`). Both present as
  ordinary case failures. Sanity-check one job before a long run.

**Status:** complete. CSVs regenerated; committed PDFs deliberately untouched
(they belong to the AoP/EoP study, a different dataset).

**Next-session pickup candidates:**
- The `run benchmarks` CI step still has never executed on dev18.
- Merge `52f0369`; the 7 remaining vacuity-audit TIMEOUTs.

## 2026-08-08 — EoP made the default; panic-override linkage bug fixed

**Session:** `llvm18-port` (session `10a63551`, continued).
**Theme:** Act on the kani sweep's finding that seabmc's abort-on-panic mode
assumes panic-class defects away.

**Key outputs:**
- `072dcb5` — 20 no_std jobs converted to error-on-panic; 13 more that declared
  `panic_error` without the CMake token (silently AoP) now get it. 3 jobs found
  reachable panics and became `sea_add_sat_test`, each keeping a `-aop` twin.
- `60a1010` — `KEEP_EXTERNAL` on the four primitives in `sea-lib/seahorn.c`.
  LTO was internalizing them, so `llvm-link --override` renamed the definition
  and SeaHorn stopped recognising the error site. Unblocks EoP for std jobs:
  all 12 that previously said "no assertion was found" now verify.
- durable/panic-modes-aop-vs-eop.md, journal/2026-08/2026-08-08-eop-conversion-
  and-override-linkage-bug.md.

**Critical corrections:**
- I attributed the verifier-primitive definitions to `sea-lib/src/bindings.rs`;
  it only declares them. They live in `sea-lib/seahorn.c`.
- I proposed and reported as working a fix where the stub defines rather than
  declares the primitives. It does not work — I grepped for `__VERIFIER_error.2`
  and got 0, but the symbol had been renamed `.4`. Match `\.[0-9]+`.
- I first tried `ABSTRACT_PANIC_ERROR` on the std jobs without understanding the
  linkage, broke 8 of 12, and had to revert before finding the real cause.

**Status:** complete; suite 216/4 (all pre-existing failures).

**Next-session pickup candidates:**
- `--vac` the 12 std jobs to check EoP is doing real work there.
- Root-cause the two unexplained sat findings (custom-vec-raw_vec,
  custom-vec-zst-alignment-issue).
- Merge `52f0369`; commit or drop the uncommitted kani 0.53 bump.

## 2026-08-07 — Port merged to main; CI on dev18; Class 1 failures fixed

**Session:** `llvm18-port` (session `10a63551`, continued).
**Theme:** Get the port through CI. Port the CI toolchain to dev18, then fix the
11 tests that failed once CI actually ran.

**Key outputs:**
- `295a344` — CI builds against `ghcr.io/seahorn/seahorn-llvm18:nightly` with
  `clang-18` and `nightly-2024-07-01`. PR #8 rebase-merged to `main`.
- `52f0369` — the 11 Class 1 "no assertion found" tests fixed and verified
  (11/11 unsat, 11/11 non-vacuous, 4/4 sampled mutants caught). Local suite
  209/213. On the fork, one commit ahead of `origin/main`; **not yet merged**.
- journal/2026-08/2026-08-07-rustc-folds-assertions-under-llvm18.md — the root
  cause (rustc folds the assertion before SeaHorn runs) and the fix pattern.
- journal/2026-08/2026-08-07-ci-ported-to-dev18.md — CI port + first run.
- durable/kani-version-to-rust-toolchain-map.md — kani 0.53.0 bundles exactly
  `nightly-2024-07-01`.
- journal/2026-08/2026-08-07-kani-vs-seabmc-sweep.md — full 275-harness kani vs
  seabmc comparison. No case where seabmc fails and kani passes; 7 genuine kani
  failures, 4 of them harnesses missing `kani::should_panic`. The committed
  benchmark CSVs are superseded (their disagreements do not reproduce).

**Critical corrections:**
- I wrote that opaque pointers changed SeaHorn's fat-pointer representation.
  They did not — ExtraWide's `(base, offset)` is SeaHorn-side and identical on
  both toolchains. What changed is checked-arithmetic canonicalisation, which
  changed the `(base, offset)` values reaching it. Caught by the user; the same
  message also overstated "most fixes trace to opaque pointers" (devirt does;
  the ZST provenance bug, `horn-explicit-sp0` and the panic bloat do not).
- I chased a missing CI run through trigger config, fork policy, org policy and
  billing before checking the platform. It was a GitHub Actions outage. Zero
  *run creation* is a platform tell, not a configuration tell.
- I reported `option-unwrap-or` as `sat` after the fix; I had restored the source
  but not rebuilt the bitcode, so the mutant was still in it.
- I ran the kani sweep at 8 workers; a single cbmc takes 22-30GB, so the box
  OOM-killed tmux and the agent. I misread the resulting dead jobs as session
  teardown three times before checking dmesg. Cap with `ulimit -v`.
- My first sweep tally counted 32 disagreements. 25 were my own timeout and
  memory limits: a CBMC resource abort also prints `VERIFICATION:- FAILED`.
- `pkill -f "<pattern>"` twice killed the shell running it, because the pattern
  text is in that shell's own command line. Use the `swee[p].sh` bracket form.

**Status:** complete apart from the merge of `52f0369`.

**Next-session pickup candidates:**
- Merge `52f0369` (a new PR is needed; #8 is closed) and watch the first
  benchmark-step run — it has never executed on dev18.
- loose-ends/parked.md: the kani upgrade + cross-check (researched, not started).

## 2026-07-23 — LLVM 18 port: blacklist re-eval + notes bootstrap

**Session:** `llvm18-port` branch work (session `10a63551`).
**Theme:** Re-evaluate the CI blacklist against the dev18 port fixes; bootstrap
the better-than-fish notebook and migrate existing notes into it.

**Key outputs:**
- Whitelisted 2 dev14 timeouts now passing on dev18 (`custom-vec-raw_vec`,
  `tinyvec-arrayvec test_retain`) — edit to `.github/workflows/blacklist.txt`.
- journal/2026-07/2026-07-23-blacklist-whitelist-two-dev14-timeouts.md — the
  finding, with reproduce commands and the 3 remaining (test-design) failures.
- loose-ends/parked.md — 3 parked test-edit tasks (cfg-simplify tautology, two
  smallvec unbounded-alloc, distill the port record).
- Bootstrapped btf structure: CLAUDE.md, durable/, empirical/, journal/,
  loose-ends/. Migrated the two flat reasoning docs into durable/ with markers.

**Critical corrections:**
- I claimed the better-than-fish plugin "does not exist in this context" after
  searching only `~/.claude/skills/` and the top of `~/.claude/plugins/`. It is
  in fact installed and enabled — the SKILL.md lives under
  `~/.claude/plugins/cache/better-than-fish/…`. Misled by an incomplete search
  (never checked the plugin cache) and by the absence of any btf structure in the
  repo (the notebook had never been bootstrapped here). Corrected once found.
- Separately, I did **not** claim the user "hasn't used it in a while" — that
  premise was not mine and I did not adopt it.

**Status:** complete (blacklist edit + notes bootstrap committed on `llvm18-port`).

**Next-session pickup candidates:**
- loose-ends/parked.md: the 3 test-edit tasks (2 are ~20–30 min each and would
  recover 3 more blacklisted jobs).
- Distill durable/dev14-to-dev18-port-record.md (over the length cap).
