# Sessions log

Chronological index of significant sessions and what they produced. Newest first.

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
