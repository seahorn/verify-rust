# Project: verify-rust (SeaHorn Rust verification) research notes

These notes live at `notes/` in the verify-rust repo (tracked, shared).

The **better-than-fish** skill governs how to add and maintain notes. Re-read its
SKILL.md for conventions. In short: three tiers by durability, confidence markers
on every claim, supersede-don't-delete on corrections.

## Tiers

- `durable/` — `[FACT]`: source/build/design-grounded, near-time-invariant.
- `empirical/` — `[EMP]`: repeated observations, version-stamped ("Verified against").
- `journal/YYYY-MM/` — `[OBS YYYY-MM-DD]`, `[HYP]`: dated single findings.
- `loose-ends/parked.md` — parked work, five-field format.
- `sessions.md` — chronological index of significant sessions.

Markers: `[FACT] [EMP] [OBS] [HYP] [OPEN] [SUPERSEDED → path]`. What persists is
true — a `[HYP]` that survives earns promotion; a wrong `[FACT]` is superseded,
never deleted.

## Triggers

- "note this" / "save this" → journal entry, today's date.
- "wrong, X" / "actually X" → grep the related note, mark `[SUPERSEDED → …]`,
  write the corrected note, add a "why I was misled" line. Never delete silently.
- "park this" → loose-ends/parked.md, five-field format.
- "digest" → weekly/monthly distillation; promote repeated claims upward.
- "what do we know about X" → grep notes first, then answer.

## Evidence standard (project-specific)

This user checks. Label hypotheses as `[HYP]`, not `[OBS]`/`[FACT]`. Give a
verification command with every `[FACT]`. `sat`/`unsat` claims must say whether
the proof is non-vacuous (assertion markers survive) — a "no assertion found"
pass is not a proof.

## Code / build anchors for citations

- verify-rust repo: `/home/siddharth/seahorn/verify-rust`
- SeaHorn build (dev18): `/home/siddharth/seahorn/seahorn/build-rel/run`
- Solver libs (LD_LIBRARY_PATH): `deps/z3-4.8.9/lib`, `deps/yices-2.6.1/lib`,
  `build-rel/run/lib` (all under the seahorn tree)
- Per-job bitcode: `build-18/src/rust-jobs/<job>/<job>_llvm.bc`
- Run one job: `./build-18/verify --expect=unsat --entry=<e> <bc>`
- ExtraWide mem manager (fixed this port): seahorn
  `lib/seahorn/BvOpSem2ExtraWideMemMgr.cc` (ptrEq, storeIntToMem)
- CI blacklist: `.github/workflows/blacklist.txt` (unsat), `.cex.txt`, `.vac.txt`
- LTO-flag probe artifact: `lto-flag-probe.sh` (kept at notes root)

## State of play

Most recent: `sessions.md` top entry. Landscape: `durable/` (port record + build
notes). Open test-edits: `loose-ends/parked.md`.
