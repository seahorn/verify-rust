# CI blacklist status (`.github/workflows/blacklist.txt`)

Status of the jobs excluded from CI in the default (unsat) run. The blacklist
was created 2025-05-05 (commit `4b9d374`) against **dev14** — every entry is a
dev14-era failure, not an LLVM-18 regression. Re-evaluated on **dev18** after
the port fixes (`devirt=sea-dsa`, `horn-explicit-sp0=true`, ExtraWide `ptrEq`
and `storeIntToMem` fixes).

Run recipe used for re-evaluation:

```
export LD_LIBRARY_PATH=<z3-4.8.9/lib>:<yices-2.6.1/lib>:<build-rel/run/lib>
./build-18/verify --expect=unsat --entry=<entry> build-18/src/rust-jobs/<job>/<job>_llvm.bc
```

## Whitelisted on dev18 (removed from the blacklist)

Both were dev14 *timeouts*; the dev18 frontend pipeline (`sea-dsa` devirt +
`explicit-sp0`) removes the blow-up that timed them out. CI ctest timeout is
2000s, so both have ample margin.

| Test | dev14 | dev18 |
|---|---|---|
| `custom-vec-raw_vec_unsat_test` (`entrypt`) | timeout | unsat, ~2s, 52 asserts, Result TRUE |
| `tinyvec-arrayvec_test_retain_unsat_test` (`test_retain`) | timeout (expected) | unsat, ~210s, Result TRUE |

## Still blacklisted (3) — not fixable by SeaHorn changes

None are LLVM-18 regressions; none are addressed by this session's ExtraWide
fixes. Each needs a **test change**, tracked here.

### `cfg-simplify_unsat_test` — tautology assertion
`src/rust-jobs/cfg-simplify/lib.rs`. The assertion
`sentinel >= (v as u32)*(v as u32)` is only reached when `v == 0` (the
`for i in 0..v { return; }` loop returns early otherwise), so it reduces to
`sentinel_u32 >= 0` — always true. `-O3` folds it away and SeaHorn reports
"no assertion was found." Original author marked this "(expected)."
- **Fix:** rewrite so the reached assertion is non-trivial (e.g. make the bound
  depend on an opaque nondet value that survives `-O3`).

### `smallvec-allocation_test_with_capacity_unsat_test` — spurious `sat`
`src/rust-jobs/smallvec-allocation/lib.rs::test_with_capacity`. Reports `sat`.
`cap` is unbounded (only `assume!(cap >= 8)`); the cex picks `cap ~ 2.6e18`, so
`with_capacity(cap)` drives a huge/overflowing allocation size into the *static*
allocator model, which returns a fixed concrete address regardless of the
requested size → the `v.capacity() == cap` assertion is refuted spuriously.
Not an ExtraWide bug of the kind fixed this session.
- **Fix:** bound `cap` in the test (e.g. `assume!(cap <= 16)`) so the modeled
  allocation is representable. Changes what is verified, hence a deliberate test
  edit rather than a silent whitelist.

### `smallvec-allocation_test_insert_from_slice_unsat_test` — spurious `sat`
Same file, `test_insert_from_slice`. Reports `sat`; "failed" on dev14 too.
Same class of allocation/aliasing modeling limitation as `test_with_capacity`.
- **Fix:** narrow the symbolic inputs / bound the allocation, then re-evaluate.
