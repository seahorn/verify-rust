# Parked investigations

Backlog of started-or-considered work not currently pursued. Five-field format.

## Pin down which fat bounds-check assume prunes Extend-for-SmallVec

**Status:** parked 2026-07-24 (root cause found; last mile remains)
**Context:** ROOT CAUSE IS KNOWN: `-fat-bnd-check` prunes the path. `sea bpf`
   (no fat instrumentation) proves the same assertion non-vacuously
   (`assertion passed`), while `sea fpf` reports `vacuity failed` on identical
   bitcode/flags. Trigger is `Extend for SmallVec` with a **symbolic element
   count**; `resize`/`append`/`drain`/`insert_many` inherit it. Ruled out: the
   bound (be_fail=0, loops proven bounded), the memory manager (extra-widemem ==
   widemem), `checked_next_power_of_two`, symbolic values, `push`/`truncate`/
   `reserve`, and a standalone `NonNull` round-trip. Pre-existing on dev14.
**Why parked:** the remaining step is identifying *which* instrumented access
   carries the undischargeable `assume(sea.is_dereferenceable(..))` — SeaHorn
   internals work rather than black-box bisection.
**To resume:** bisect with `--no-fat-fns <mangled-name>,...` to exclude candidate
   functions from instrumentation until the vacuity clears; that names the
   offending access. Diagnostics: `--enable-fat-vacuity-check=true` (per-location
   report), `--keep-temps --temp-dir=DIR` (with `-S` the `.bc` files are text).
   Minimal repro is in the journal entry (`test_probe_extend_sym_count`).
   Cross-check any fix against `sea bpf`, which already proves the property.
**Effort estimate:** ~half-day, needs SeaHorn opsem/instrumentation familiarity.
**References:** journal/2026-07/2026-07-24-vacuity-audit-of-all-unsat-tests.md

## Flag the 6 unflagged vacuous tests in blacklist.vac.txt

**Status:** parked 2026-07-24
**Context:** The vacuity audit found 9 fully-vacuous green tests. Only 3 are in
   `.github/workflows/blacklist.vac.txt` (`result-unwrap_or_else`,
   `smallvec-bound2 test_resize`/`test_resize2`). Unflagged: `smallvec-bound2`
   `test_append`, `test_drain`, `test_insert_many`, `test_resize_with`,
   `test_resize_with2`, plus `custom-vec-zst-alignment-issue`. They pass the
   unsat CI run while proving nothing.
**Why parked:** flagging them in the vac blacklist only records the problem;
   the real work is making them non-vacuous (see next entry). Also an
   outward-facing CI change, wants a deliberate decision.
**To resume:** decide per test — fix (preferred) or flag. If flagging, add the
   CI test names to `blacklist.vac.txt` with the journal entry as justification.
**Effort estimate:** ~15 min to flag; see next entry for fixing.
**References:** journal/2026-07/2026-07-24-vacuity-audit-of-all-unsat-tests.md

## Resolve the 8 vacuity-audit TIMEOUTs

**Status:** parked 2026-07-24
**Context:** 8 bounded tests could not be classified within the 90-100s per-test
   budget: `smallvec/test_insert_from_slice`, `smallvec-allocation-bound2/`
   `{test_drain,test_insert_many}`, `smallvec-bound4/test_push`,
   `tinyvec-arrayvec{,-aop}/{test_retain,test_splice}`. Notably
   `tinyvec-arrayvec/test_retain` is one of the two tests whitelisted on
   2026-07-23, so its non-vacuity is asserted but not proven.
**Why parked:** each needs minutes of solver time; the cheap majority was done
   first.
**To resume:** re-run `./build-18/verify --vac --entry=<e> <bc>` with a 600-900s
   budget, one at a time. Record reach/vacuous counts in the audit journal.
**Effort estimate:** ~1-2 h wall-clock, mostly unattended.
**References:** journal/2026-07/2026-07-24-vacuity-audit-of-all-unsat-tests.md

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
