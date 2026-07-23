# Parked investigations

Backlog of started-or-considered work not currently pursued. Five-field format.

## Make cfg-simplify's assertion survive -O3

**Status:** parked 2026-07-23
**Context:** `cfg-simplify_unsat_test` is blacklisted with "no assertion found".
   The assertion `sentinel >= (v as u32)*(v as u32)` is only reached when `v == 0`,
   reducing to `sentinel_u32 >= 0` (always true); `-O3` folds it away so SeaHorn
   sees no assertion. It's a test-design tautology, not a SeaHorn/port bug.
**Why parked:** the two dev14 timeouts (raw_vec, tinyvec retain) were the
   higher-value, verified whitelist wins; this needs a Rust edit + re-eval.
**To resume:** rewrite so the reached assertion is non-trivial — make the bound
   depend on an opaque nondet value that survives `-O3` (e.g. relate `sentinel`
   to a second `verifier::any!()` under an assume). Rebuild the job, confirm
   `unsat` with a real assertion, then remove from `.github/workflows/blacklist.txt`.
**Effort estimate:** ~20 min.
**References:** journal/2026-07/2026-07-23-blacklist-whitelist-two-dev14-timeouts.md

## Bound smallvec-allocation tests so the static allocator can model them

**Status:** parked 2026-07-23
**Context:** `smallvec-allocation_test_with_capacity` and `..._test_insert_from_slice`
   both report spurious `sat`. `cap`/`len` are unbounded, so the cex drives a
   huge allocation (cex `cap ≈ 2.6e18`) into the static allocator, which returns a
   fixed address regardless of size → capacity assertion refuted spuriously. Not
   an ExtraWide bug of the kind fixed this session (ptrEq / storeIntToMem).
**Why parked:** changes what the test verifies (a deliberate test edit), so it
   wanted an explicit decision rather than a silent whitelist.
**To resume:** add an upper bound (e.g. `assume!(cap <= 16)` / bound `len`),
   rebuild, confirm `unsat` is real (non-vacuous), then whitelist. Confirm the
   bound still exercises the intended spill/heap path.
**Effort estimate:** ~30 min for both.
**References:** journal/2026-07/2026-07-23-blacklist-whitelist-two-dev14-timeouts.md

## Distill the dev14→dev18 port record into durable facts + journal

**Status:** parked 2026-07-23
**Context:** durable/dev14-to-dev18-port-record.md is the pre-btf migration doc,
   imported whole (~1900 lines, well over the 300-line durable cap). Its
   structural claims are durable `[FACT]`s but its pass/fail snapshots are stale
   (superseded by this session's fixes).
**Why parked:** the record is faithful and usable as-is; distillation is cleanup,
   not blocking.
**To resume:** split into small durable `[FACT]` notes (the five pin sites; the
   three failure classes; the ExtraWide ptrEq/storeIntToMem fixes; the panic
   abstraction) + dated journal entries for the investigations, leaving a thin
   durable pointer. Cross-link.
**Effort estimate:** ~half-day.
**References:** durable/dev14-to-dev18-port-record.md
