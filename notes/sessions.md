# Sessions log

Chronological index of significant sessions and what they produced. Newest first.

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
