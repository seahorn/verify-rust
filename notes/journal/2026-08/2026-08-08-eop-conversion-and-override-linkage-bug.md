# Making EoP the default, and the override linkage bug it exposed

Follow-on from the kani sweep, which found an `i32` overflow in `vec-filter`
that seabmc reported `unsat` on. Root cause was the panic mode, not the solver.

## Why seabmc missed the overflow

[OBS 2026-08-08] seabmc was never blind to it. `vec-filter`'s bitcode contains
462 `with.overflow` intrinsics, including in `Iterator::sum`:

```llvm
%21 = call { i32, i1 } @llvm.sadd.with.overflow.i32(i32 %14, i32 %17)
      br i1 %22, label %23, label %24
23:   tail call void @llvm.trap()
      unreachable
```

The overflow branch dead-ends in `llvm.trap`/`unreachable`, SeaHorn prunes it,
and `(result & 1) == 0` is proved over non-overflowing runs only. That verdict
is correct for the property actually being checked.

[OBS 2026-08-08] Flipping `vec-filter` to EoP changes the branch target to
`panic_const_add_overflow` → `#[panic_handler]` → `verifier_error()`, and the
verdict to **`sat`**. Both halves are required; the Cargo feature alone does
nothing. → durable/panic-modes-aop-vs-eop.md

## The conversion

[OBS 2026-08-08] 20 no_std jobs converted to EoP; 13 more already declared
`panic_error` in Cargo.toml **without** the CMake token, so they had been
silently running AoP — they now get the token. → commit `072dcb5`

Three jobs surfaced reachable panics and are registered `sea_add_sat_test`,
each keeping a `<job>-aop` twin that preserves the original proof (the twins
reuse the base `lib.rs` via `[lib] path`, unlike the hand-duplicated
`tinyvec-arrayvec-aop`):

| job | finding |
|---|---|
| `vec-filter` | `i32` overflow in `.filter(..).sum()` — root-caused |
| `custom-vec-raw_vec` | reachable panic — **not** root-caused |
| `custom-vec-zst-alignment-issue` | reachable panic — **not** root-caused |

Keeping the `-aop` twin matters most for `custom-vec-raw_vec`, whose
52-assertion non-vacuous proof was whitelisted on 2026-07-23 and would
otherwise have been lost.

## The linkage bug

[OBS 2026-08-08] Routing the 12 **std** jobs through `ABSTRACT_PANIC_ERROR`
instead (they cannot use `panic_error`) made **8 of 12** report *"no assertion
was found"*. Not a caught bug — the job stopped verifying anything.

Cause: `llvm-link --override` renames the internalized `__VERIFIER_error` to
`__VERIFIER_error.N`, and SeaHorn identifies an assertion by the branch to
`__VERIFIER_error`. Full mechanism and the check commands are in
durable/panic-modes-aop-vs-eop.md.

Fix: `KEEP_EXTERNAL` (`used` + default visibility) on the four primitives in
`src/sea-lib/seahorn.c`. → commit `60a1010`. Afterwards all 12 std jobs verify
`unsat` with no renamed symbols, so std jobs can now be EoP.

[FACT] The same latent flaw sits under `ABSTRACT_PANIC` (abort), used by the
five borsh jobs in CI. It escapes only because those modules *declare* rather
than *define* `__VERIFIER_assume`. A job that defined it would have had its
**assumptions** silently dropped — harder to notice than a lost assertion,
because nothing reports it.

## Corrections

[SUPERSEDED] I said the definitions come from `sea-lib/src/bindings.rs`. They do
not — `bindings.rs` only *declares* them in `extern "C"` blocks. The definitions
are the empty stand-ins in `src/sea-lib/seahorn.c`.

[SUPERSEDED] I proposed, and reported as working, a fix where the stub *defines*
the primitives instead of declaring them. It does not work. I had grepped for
`@__VERIFIER_error\.2` and got 0; the symbol had been renamed to `.4`. Always
match `\.[0-9]+`, never a specific suffix.

[OPEN] The 12 std jobs reach `unsat` with the assertion visible, but I did not
check that their panic paths are **reachable** — EoP may be doing no real work
on them. A `--vac` pass would settle whether these are clean or merely
panic-free by construction.

[OPEN] Coverage is asymmetric: only 3 jobs have both modes; the other ~30 are
EoP-only. Twins for all of them roughly double their build cost.

[OPEN] `custom-vec` was reverted to AoP. Its `test_grow`/`test_push` genuinely
fail under EoP, but it uses `sea_discover_tests`, which infers sat/unsat purely
from a `testfail_` name prefix — there is no way to mark one discovered test as
sat without renaming the harness and changing what it claims.

## State

Suite **216 tests, 4 failures**, all pre-existing (3 blacklisted +
`tinyvec-arrayvec-aop_test_retain` at the local 400s cap).

## See also

- ../../durable/panic-modes-aop-vs-eop.md
- 2026-08-07-kani-vs-seabmc-sweep.md
