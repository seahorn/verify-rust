# Kani 0.53.0 vs seabmc: full sweep of all 275 harnesses

First cross-tool comparison on dev18. Supersedes the committed benchmark CSVs,
which turn out to be unusable.

## Result

[OBS 2026-08-07] 275 `kani::proof` harnesses across 63 jobs, kani 0.53.0 against
the current seabmc ctest run (209 Passed / 3 Failed / 1 Timeout of 213).

| bucket | n | meaning |
|---|---|---|
| AGREE-pass | 147 | both verify |
| AGREE-fail | 3 | both reject (the `*-error` jobs) |
| kani-only | 80 | job runs a single `entrypt` ctest, so no per-harness seabmc verdict |
| kani fails, seabmc passes | 32 | see decomposition |
| incomplete | 13 | harness never reached (job-level run cut short) |

**[FACT] There is no case where seabmc fails and kani passes.** Every divergence
is in the same direction.

## The 32 decomposed — only 7 are verdicts

[FACT] 25 of the 32 are *my* resource limits, not tool disagreements:

- **15 TIMEOUT** — 150-300s per harness (`custom-vec` ×9, `smallvec`
  `test_clear`/`test_extend_from_slice`, `tinyvec-arrayvec{,-aop}/test_pop`,
  `borsh-option`, `smallvec-insert_many-fix`).
- **10 OOM** — my 14GB `ulimit -v` (`tinyvec-arrayvec{,-aop}` `test_drain`,
  `test_retain`, `testfail_drain_panic`; `smallvec` `test_insert`,
  `test_insert_from_slice`; `smallvec-bound4` `test_push`, `test_try_grow`;
  `smallvec-allocation-bound2/test_shrink_to_fit`; `borsh/test_string`).

[FACT] **A CBMC resource abort also prints `VERIFICATION:- FAILED`.** The log
says `Out of memory` then `CBMC failed with status 6`, and only then the FAILED
line. Parsing the verdict alone reports your own ulimit as a tool disagreement —
I did exactly that on the first pass. Always grep the body:

```bash
grep -E "^Out of memory$|CBMC failed with status" <log>   # => resource abort, not a CEX
```

## The 7 genuine failures

[OBS 2026-08-07] Four are one bug class — **the harness ends in a deliberate
panic but carries no `kani::should_panic`**:

| job | harness | kani check that fires |
|---|---|---|
| `smallvec-bound8` | `test_remove` | `assertion failed: index < len` (smallvec-1.13.2:1352) |
| `smallvec-bound8` | `test_swap_remove` | `attempt to subtract with overflow` (:1333) |
| `smallvec-allocation-bound8` | `test_remove` | same |
| `smallvec-allocation-bound8` | `test_swap_remove` | same |

Each ends:

```rust
// v is empty, so this should panic
v.remove(0);
// This assertion should not be reachable since the call to remove panics.
verifier::vassert!(false);
```

seabmc abstracts the panic (abort / `assume false`), so the path is pruned and
`vassert!(false)` is genuinely unreachable → real `unsat`. kani treats the same
panic as a failed check. Both are right about the code; the harness is annotated
for one tool only. These are `testfail_`-shaped tests wearing a `test_` name.
Verified in isolation at a 20GB cap with all unwinding assertions `SUCCESS`, so
neither OOM nor under-unwinding explains them.

[OBS 2026-08-07] The other three are **property classes seabmc does not check**:

- `vec-filter/entrypt` — `attempt to add with overflow`. `values.iter().filter(..).sum::<i32>()`
  can overflow; the harness only asserts `(result & 1) == 0`. A real defect kani
  finds and seabmc is not looking for.
- `smallvec-allocation/test_from_raw_parts` — `rust_dealloc` size/layout mismatch.
- `smallvec-allocation-bound4/test_try_grow` — `assertion failed: new_cap >= len`.

## The old CSVs are unusable

[SUPERSEDED] `benchmark/benchmark_postprocess{,_aop,_eop}.csv` recorded 15
disagreements. Both of the two that had an actual verdict (rather than a
timeout) **fail to reproduce**: `smallvec-allocation-bound2/test_insert_many` now
passes both tools (seabmc `unsat` non-vacuous in 483s; kani SUCCESSFUL in 45s),
and `testfail_insert_many_panic` panics as expected. The CSVs predate the dev18
port *and* the `testfail_` rename — they still use the old `test_*_panic` names.

## Resource profile is the real story

[FACT] CBMC needs 22-30GB RSS on single `tinyvec-arrayvec`/`smallvec` harnesses;
seabmc proves the same properties in seconds to minutes. That, not any modelling
gap, explains the historical timeout cluster on `drain`/`splice`/`retain`.

**Running this sweep with 8 workers OOM-killed the machine**, taking out tmux and
the agent — a single cbmc at 30GB × several workers exhausts 60GB and the kernel
picks unrelated victims. Cap it:

```bash
( ulimit -v 14000000; cargo-kani --harness <h> --manifest-path <job>/Cargo.toml )
```

## Reproduce

```bash
cargo +nightly-2024-07-01 install --locked kani-verifier --version 0.53.0 --root <dir>
<dir>/bin/cargo-kani setup
<dir>/bin/cargo-kani --harness <h> --manifest-path src/rust-jobs/<job>/Cargo.toml
```

[FACT] `--locked` is **required**: without it the dep `home-0.5.12` demands the
`edition2024` Cargo feature, unstable in `cargo 1.81.0-nightly (4ed7bee47
2024-06-25)`. Same trap as the repo's `ahash =0.8.11` pin.

[FACT] Kani 0.53.0 runs with `rust-toolchain.toml` present and **no** `rustup
override` — confirming the [HYP] in parked.md that the
`rm -f rust-toolchain.toml` workaround can go. Its bundled nightly is the repo's.

## See also

- ../../durable/kani-version-to-rust-toolchain-map.md
- ../../loose-ends/parked.md
