# Two dev14 blacklist timeouts now pass on dev18 — whitelisted

Re-evaluation of `.github/workflows/blacklist.txt` (the default/unsat CI run)
after the dev18 port fixes.

[FACT] The CI blacklist was created 2025-05-05 (commit `4b9d374`, an ancestor of
`main`) against dev14. Every entry is therefore a dev14-era failure, not an
LLVM-18 regression. → `git merge-base --is-ancestor 4b9d374 main` = true.

[OBS 2026-07-23] Two entries that were dev14 *timeouts* now verify to `unsat`
within the 2000s CI ctest cap on dev18, so they were removed from the blacklist:

| test (entry) | dev14 | dev18 |
|---|---|---|
| `custom-vec-raw_vec_unsat_test` (`entrypt`) | timeout | unsat, ~2s, 52 asserts, Result TRUE |
| `tinyvec-arrayvec_test_retain_unsat_test` (`test_retain`) | timeout (expected) | unsat, ~210s, Result TRUE |

Our reading: the dev18 frontend pipeline chosen this session (`devirt=sea-dsa`
instead of type-based devirt, plus `horn-explicit-sp0=true`) removes the loop-
unroll / byte-split blow-up that timed these out on dev14. Both are genuine,
non-vacuous proofs (raw_vec: 52 `verifier.assert` markers survive; both report
`BRUNCH_STAT Result TRUE`), not "no assertion found" passes.

## Reproduce

```bash
export LD_LIBRARY_PATH=\
/home/siddharth/seahorn/seahorn/deps/z3-4.8.9/lib:\
/home/siddharth/seahorn/seahorn/deps/yices-2.6.1/lib:\
/home/siddharth/seahorn/seahorn/build-rel/run/lib
cd /home/siddharth/seahorn/verify-rust
./build-18/verify --expect=unsat --entry=entrypt \
  build-18/src/rust-jobs/custom-vec-raw_vec/custom-vec-raw_vec_llvm.bc      # unsat ~2s
./build-18/verify --expect=unsat --entry=test_retain \
  build-18/src/rust-jobs/tinyvec-arrayvec/tinyvec-arrayvec_llvm.bc          # unsat ~210s
```

## The three that stay blacklisted

None are LLVM-18 regressions; none are addressed by this session's ExtraWide
fixes. Each needs a *test change*, not a SeaHorn change — parked in
loose-ends/parked.md.

[OBS 2026-07-23] `cfg-simplify_unsat_test` → "no assertion was found". The
assertion `sentinel >= (v as u32)*(v as u32)` is only reached when `v == 0` (the
`for i in 0..v { return; }` loop returns early otherwise), so it reduces to
`sentinel_u32 >= 0` — always true. `-O3` folds it away. Original author marked it
"(expected)". → src/rust-jobs/cfg-simplify/lib.rs:22

[OBS 2026-07-23] `smallvec-allocation_test_with_capacity_unsat_test` → spurious
`sat`. `cap` is unbounded (`assume!(cap >= 8)` only); the cex picks
`cap ≈ 2.6e18`, so `with_capacity(cap)` drives a huge/overflowing allocation size
into the *static* allocator model (`--sea-opsem-allocator=static`), which returns
a fixed concrete address (cex showed `0xbfffffd0`) regardless of requested size →
`v.capacity() == cap` is refuted spuriously. Not an ExtraWide bug of the kind
fixed this session. → src/rust-jobs/smallvec-allocation/lib.rs::test_with_capacity

[OBS 2026-07-23] `smallvec-allocation_test_insert_from_slice_unsat_test` →
spurious `sat`; "failed" on dev14 too. Same class of unbounded-allocation /
static-allocator modeling limitation.

[HYP 2026-07-23] Bounding the symbolic inputs in the two smallvec tests (e.g.
`assume!(cap <= 16)`) would make the modeled allocation representable and flip
them to `unsat` — untested, and it changes what the test verifies, so it's a
deliberate test edit, not a silent whitelist.

## Why this matters

Confirms the port fixes buy real coverage back (2 jobs), and cleanly separates
"SeaHorn/port problem" (none remain here) from "test-design problem" (the
remaining 3). The remaining work is small Rust-test edits, not solver work.

## See also

- ../../durable/dev14-to-dev18-port-record.md
- ../../loose-ends/parked.md
