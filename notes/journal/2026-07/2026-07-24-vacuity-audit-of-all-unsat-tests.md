# Vacuity audit of all 194 CI unsat tests — the suite is mostly sound, 9 are not

Triggered by the question "does vacuity pass with no bound?", which exposed that
an `unsat` verdict alone says nothing about whether anything was proved.

[FACT] An `unsat` in this suite is only meaningful if at least one property
assertion is *reachable*. `./build-18/verify --vac` reports this per assertion:
`Assertion passed: <loc>` = assertion holds (with `backedge!!` = loop bounded OK);
`Vacuity failed: <loc>` = assertion **unreachable**, i.e. proved nothing. The
wrapper fails the vac run when `vacuity_failed - vacuity_passed` is non-empty.
→ build-18/verify:145-200 (`check_vacuity`, `assertion_passed`/`vacuity_failed` sets)

[FACT] Only 13 jobs run per-`test_` CI tests (`sea_discover_tests`); every other
job runs a single `entrypt` test (`sea_add_unsat_test`). Enumerating `test_`
symbols for a non-discover job invents tests CI never runs.
→ `grep -rl discover src/rust-jobs/*/CMakeLists.txt` = 13 jobs

## Results (194 CI unsat tests: 49 no-bound, 145 bounded)

| class | no-bound | bounded | meaning |
|---|---|---|---|
| NONVAC | 29 | 108 | real, non-vacuous proof |
| PARTIAL | 6 | 5 | real proof + dead panic branches (fine) |
| **VACUOUS** | **2** | **7** | passes `unsat`, proves nothing |
| NOVACLINES | 7 | 13 | no vacuity lines emitted (inconclusive) |
| TIMEOUT | 0 | 8 | unclassified at 90-100s |
| SAT/other | 5 | 3 | mostly `*-error` sat-tests mis-included |

[OBS 2026-07-24] ~137/194 are confirmed real proofs. The suite is **not**
broadly vacuous.

## The 9 fully-vacuous green tests

[OBS 2026-07-24] `smallvec-bound2`: 7 of its 11 tests are vacuous —
`test_append, test_drain, test_insert_many, test_resize, test_resize2,
test_resize_with, test_resize_with2`. Only `test_resize` and `test_resize2` are
in `blacklist.vac.txt`; **the other 5 are vacuous but unflagged**.

[OBS 2026-07-24] `test_resize2` asserts `v.len() != resize_point` immediately
after `v.resize(resize_point, ..)` — false by construction. It passes `unsat`
*only because* it is vacuous. Clearest possible proof that these greens are
meaningless.

[OBS 2026-07-24] The cause is **not** the bound. `smallvec-allocation-bound2`
runs the same-named tests at the same `bound: 2` and is uniformly NONVAC. The
difference is the smallvec implementation:
- `smallvec-bound2` → crates.io `smallvec 1.10.0` (resolved 1.13.2), 331-line
  tests-only lib.rs → **vacuous**
- `smallvec-allocation-bound2` → smallvec **vendored** into its own 2440-line
  lib.rs → **non-vacuous**

[OBS 2026-07-24] Within `smallvec-bound2`, the concrete variant
`test_resize_conc` (const LEN/RESIZE_POINT, no `verifier::any!`) is NONVAC while
its symbolic twin `test_resize` is vacuous — so the pruning is tied to symbolic
loop trip counts through the crates.io implementation.

[SUPERSEDED → bisection below] [HYP 2026-07-24] The crates.io smallvec prunes
every path via `unreachable_unchecked()` when the model cannot resolve the
`SmallVecData` discriminant. **Refuted** by the probe bisection below.

## Bisection of the smallvec-bound2 vacuity (probe tests, since removed)

Probes were appended to `src/rust-jobs/smallvec-bound2/lib.rs`, rebuilt via
`ninja src/rust-jobs/smallvec-bound2/CMakeFiles/smallvec-bound2.ir`, measured
with `--vac`, then removed (tree restored).

[OBS 2026-07-24] Non-vacuous (reach=1): `SmallVec::new` + `len()`; `push` with a
concrete count; `push` in a **symbolic**-trip-count loop; `truncate` with a
symbolic point; `checked_next_power_of_two` on a symbolic input; `extend` with a
concrete count; `extend`/`resize` with a **symbolic fill value** but concrete
count.

[OBS 2026-07-24] Vacuous (reach=0): `resize` with a **symbolic** resize point on
an empty vec; `resize` with a *concrete* point after a **symbolic**-length push
loop. So the trigger is a symbolic length/count reaching the iterator-driven bulk
operations — `resize`/`extend`/`append`/`drain`/`insert_many` — which is exactly
the set of vacuous tests. `push`/`truncate` (non-iterator paths) are unaffected.

[OBS 2026-07-24] Refuted along the way: the bound (vacuous at 2 and 3 alike);
`checked_next_power_of_two` (the construct the vendored copy replaced —
non-vacuous on its own); symbolic *values* (only symbolic *counts* matter).

[OBS 2026-07-24] **Pre-existing, not a dev18 regression.** The same tests are
equally vacuous under dev14 (`build-14/verify --vac`, seahorn-dev14):
`test_resize` reach=0 vac=1, `test_append` reach=0 vac=2, both `unsat`. The
dev14-era `blacklist.vac.txt` flags only `test_resize2`, so the other five were
already passing vacuously before the port.

[FACT] The test bodies are **byte-identical** to their non-vacuous twins in
`smallvec-allocation-bound2` (same symbolic inputs, same `bound: 2`). The sole
difference is the implementation under test: crates.io `smallvec 1.10.0` vs a
vendored copy whose `try_reserve` was rewritten for verifiability. Therefore the
vacuity **cannot be fixed by editing the test bodies** — they are already correct.

## Modeling-gap investigation (partial — root cause NOT fully isolated)

[FACT] `sea fpf` runs `seapp -fat-bnd-check` first, which instruments every
memory access with a **hard** `assume(sea.is_dereferenceable(ptr, n))`. The final
IR for `test_resize` contains **156** such assumes. A hard assume that the model
cannot discharge silently prunes the path — the exact signature of a vacuous
`unsat`.
→ dump with `--keep-temps --temp-dir=DIR`; note `-S` makes the `.bc` files text
→ `grep -A1 "call i1 @sea.is_dereferenceable" *.cut.o.bc` shows the assume pairs

[FACT] `sea fpf` exposes a purpose-built diagnostic: `--enable-fat-vacuity-check=true`
(pipes through as `--horn-bnd-chk-vac=true`), which reports per-location vacuity
for the bounds-check instrumentation.
→ `sea fpf --help` under the `fat-bnd-check` stage

[OBS 2026-07-24] With that diagnostic on, the dead regions localize to
crates.io smallvec's **heap/spill** paths — `lib.rs:722` (`heap()` accessor),
`lib.rs:1117` (`push`'s `if *len == cap` spill branch) — plus `panicking.rs:404`.
For an inline-only `CAP=2` vec those *should* be dead, so they are consistent
with correct modelling and do not by themselves explain the failure.

[OPEN] The property assertion after `resize` remains unreachable even with the
fat vacuity check enabled (`vacuity failed (unsat)` on the job's own vassert
line, verdict `unsat`). So: the vec provably never spills, no panic path is
reachable, no loop exceeds its bound (backedge assertions pass) — and yet the
statement after `resize` cannot be reached. The specific constraint that kills
the path is **not yet identified**. Continuing this needs SeaHorn-internals
debugging (which assume is false on the surviving path), not more black-box
bisection. Parked in loose-ends/parked.md.

### Minimal reproducer (6 lines)

Append to `src/rust-jobs/smallvec-bound2/lib.rs`, rebuild with
`ninja src/rust-jobs/smallvec-bound2/CMakeFiles/smallvec-bound2.ir`, run
`./build-18/verify --vac --entry=test_probe_resize_only <bc>` → `reach=0 vac=1`:

```rust
#[no_mangle]
fn test_probe_resize_only() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();
    let rp: usize = verifier::any!();
    verifier::assume!(rp <= CAP);
    v.resize(rp, verifier::any!());
    verifier::vassert!(v.len() == rp);   // UNREACHABLE
}
```

Swapping `rp` for a literal makes it reachable — that one-token change is the
whole delta between a real proof and a vacuous one.

[OBS 2026-07-24] `custom-vec-zst-alignment-issue` is fully vacuous (both
assertions, lines 61-62) and **not** vac-blacklisted. **Not** caused by the
`wrapping_byte_add` change in `2ea6285` — rebuilt from the original `inttoptr`
source and it is equally vacuous. Pre-existing dev18 behaviour.

[OBS 2026-07-24] `result-unwrap_or_else` is vacuous but already vac-blacklisted;
this session's change to it was cosmetic (`fn entrypt` → `pub extern "C"`).

## Corrections made during this audit

[SUPERSEDED → this note] "custom-vec-raw_vec's unsat is vacuous, the whitelist
was unsound." **Wrong.** It has 11 reachable proven assertions (`reach=11,
be_fail=0`). I was misled by my own classifier: it required the literal
`backedge` substring after `Assertion passed`, which hid every non-backedge
assertion and made real proofs look empty. The whitelist stands.

[SUPERSEDED → this note] "Removing `bound: 16` from smallvec-allocation makes 21
entries pass in ~1s, so the bound is the problem." **Wrong and backwards.**
Without a bound the loops never unroll: the backedge assertion fires
(`Assertion failed ... backedge!!`) and every property assertion goes
`vacuity failed` — those fast unsats proved nothing. The bound is **required**;
under it `test_insert_from_slice`'s `sat` is a real, non-vacuous counterexample.

## Why this matters

Green is not proof. Two of this session's earlier conclusions were wrong in
opposite directions because verdicts were read without vacuity. Any future claim
that a test "passes" should cite reachable-assertion counts, not just `unsat`.

## See also

- 2026-07-23-blacklist-whitelist-two-dev14-timeouts.md
- ../../loose-ends/parked.md
