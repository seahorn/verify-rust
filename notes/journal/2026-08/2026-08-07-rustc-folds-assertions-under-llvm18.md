# The 11 "no assertion found" failures are folded by rustc, not by SeaHorn

The last standing class of dev18 failures (Class 1 in the port record) is fixed.
The cause was one layer earlier than the port record assumed.

## The finding

[OBS 2026-08-07] The assertions are constant-folded by **rustc's own LLVM 18**,
before SeaHorn is invoked at all. The post-merge *pre-opt* bitcode already
contains a literal `true`:

```llvm
define internal void @entrypt() {
  %1 = tail call noundef i32 @sea_nd_i32()
  %2 = and i32 %1, 1
  %3 = icmp eq i32 %2, 0
  br i1 %3, label %4, label %5
4:  tail call void @__VERIFIER_assert(i1 noundef zeroext true)   ; <- folded
5:  tail call void @__VERIFIER_assert(i1 noundef zeroext true)   ; <- folded
}
```

→ `build-18/src/rust-jobs/option-unwrap-or/option-unwrap-or_llvm.ll`, the
artifact of the "post merge pre opt" ninja step. Verify with:

```bash
cd build-18 && ninja src/rust-jobs/<job>/CMakeFiles/<job>.ir
sed -n "/define.*@entrypt/,/^}/p" build-18/src/rust-jobs/<job>/<job>_llvm.ll
```

**Corrects a wrong assumption**, not a wrong claim: the port record diagnosed the
symptom ("the LLVM 18 optimiser is simply strong enough to fold them away") but
left the impression that seahorn's `-O3` did it. It does not; the bitcode arrives
already folded. Anything done at the `sea` pipeline level could never have fixed
this. → durable/dev14-to-dev18-port-record.md:604 (Class 1)

## Why `verifier::any!` was not enough

[FACT] Every one of these tests already drew its input from `verifier::any!`.
The *input* was opaque; the *assertion* was not. Each test asserted against a
value the front end could correlate with the branch condition that produced it:

```rust
let v: i32 = verifier::any!();
let result = (if (v & 1) == 1 { None } else { Some(v) }).unwrap_or(0);
if (v & 1) == 0 { verifier::vassert!(result == v); }   // same (v & 1) twice
else            { verifier::vassert!(result == 0); }
```

rustc threads the two uses of `(v & 1)` and proves both arms. Nondeterministic
input does not imply an unprovable assertion.

## The fix pattern

[FACT] Compare against an expected value drawn from a *second* `verifier::any!`
and tied to the concrete value through `verifier::assume!`. rustc sees `assume`
only as an opaque `extern "C"` call (`@__VERIFIER_assume`) and cannot derive the
relation, so the assertion survives to BMC, where SeaHorn discharges it using
the assume. Semantically identical; opaque at compile time.

```rust
let expected: i32 = verifier::any!();
if (v & 1) == 0 { verifier::assume!(expected == v); }
else            { verifier::assume!(expected == 0); }
verifier::vassert!(result == expected);
```

This is the same technique loose-ends/parked.md recommends for `cfg-simplify`
("relate the sentinel to a second `verifier::any!()` under an assume").

## Results

[OBS 2026-08-07] All 11 fixed, in `option-or`, `option-unwrap-or`,
`result-iter_mut`, `serde`, and 7 tests in `tinyvec-arrayvec-aop`. → commit
`52f0369`.

| check | result |
|---|---|
| `--expect=unsat` | 11/11 `unsat` |
| `--vac` reachability | 11/11 `reach>0`, `vacuous=0` — **non-vacuous** |
| mutation test (4 sampled) | 4/4 mutants caught as `sat` |

Full local suite after the fix: **209 / 213** (`ctest -j30 --timeout 400`). The 4
remaining are the 3 blacklisted tests plus `tinyvec-arrayvec-aop_test_retain`,
which needs more than the local 400s cap and passes under CI's 2000s.

Reproduce one:

```bash
export LD_LIBRARY_PATH=\
/home/siddharth/seahorn/seahorn/deps/z3-4.8.9/lib:\
/home/siddharth/seahorn/seahorn/deps/yices-2.6.1/lib:\
/home/siddharth/seahorn/seahorn/build-rel/run/lib
./build-18/verify --expect=unsat --entry=entrypt \
  build-18/src/rust-jobs/option-unwrap-or/option-unwrap-or_llvm.bc   # unsat
./build-18/verify --vac --entry=entrypt \
  build-18/src/rust-jobs/option-unwrap-or/option-unwrap-or_llvm.bc   # Assertion passed, no Vacuity failed
```

## Two side findings

[OBS 2026-08-07] The `// Necessary to make seahorn work.` keep-alives in 4
`tinyvec-arrayvec-aop` tests (`assume!(x < u32::MAX/2); vassert!(x * 2 >= x)`)
were an earlier author's workaround for this same problem under LLVM 14. LLVM 18
folds them too, so they had stopped working. Removed — with the real assertions
now surviving they were dead code that looked load-bearing.

[OBS 2026-08-07] `tinyvec-arrayvec-aop::test_extend_from_slice` was **not** this
bug. It had no assertion at all and never called `extend_from_slice` — an
unfinished stub, so "no assertion found" was the correct verdict. Body written
out, mirroring the existing `testfail_extend_from_slice` sibling. This is the
one piece of authored test content in `52f0369` rather than a mechanical fix,
and it only checks the resulting length, not element contents.

## Why this matters

Class 1 is closed, and with it the last non-blacklisted dev18 failure class. More
usefully: it establishes that on this toolchain **an assertion can die before
SeaHorn ever sees it**, and that `verifier::any!` on the input is not sufficient
protection. Any test whose assertion is derivable from its construction is
exposed. The `--vac` + mutation-test pair is what distinguishes a real proof from
both failure modes (folded-away and vacuous).

## See also

- ../../durable/dev14-to-dev18-port-record.md (Class 1, line 604)
- 2026-07-24-vacuity-audit-of-all-unsat-tests.md (the other failure mode)
- ../../loose-ends/parked.md (cfg-simplify uses the same fix)
