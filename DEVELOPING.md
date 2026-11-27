# Developing verify-rust

This guide covers how to build a verification job, check that it verifies what you
intend, and add new jobs. For installing the toolchain and SeaHorn, see
[README.md](README.md).

A passing `ctest` is not enough on its own. A test can pass because its assertion was
never reached, or because the compiler removed it. The checks in
[Before submitting a change](#before-submitting-a-change) catch both.

In the commands below, `<build>` is your CMake build directory and `<job>` is the name of
a job under `src/rust-jobs/`.

## Build

```bash
cmake <build>                                    # reconfigure after adding/removing a job or editing a CMakeLists.txt
cmake --build <build> --target <job>.ir          # build one job
cmake --build <build>                            # build everything
```

| Thing | Name |
| --- | --- |
| Build target | `<job>.ir` |
| Bitcode that SeaHorn verifies | `<build>/src/rust-jobs/<job>/<job>_llvm.bc` |
| Readable IR (for inspection) | `<build>/src/rust-jobs/<job>/<job>_llvm.ll` |
| ctest names | `<job>_unsat_test`, `<job>_sat_test`, `<job>_<fn>_unsat_test` |

Cargo may print warnings such as `resolver = "1"` or `unexpected cfg(kani)`. They are
harmless; check the last lines of the output for real errors.

## Verify

### 1. Result: does it return what the test expects?

```bash
ctest --test-dir <build> -R '^<job>_' -V
```

`sea_add_unsat_test` expects `unsat` (no assertion can fail). `sea_add_sat_test` expects
`sat` (a counterexample exists). `sea_add_unsat_test_novac` is an unsat test excluded
from the vacuity check in CI. ctest reports "Passed" when the result matches.

### 2. Vacuity: was every assertion actually checked?

```bash
<build>/verify --vac <build>/src/rust-jobs/<job>/<job>_llvm.bc 2>&1 | grep -E 'Assertion|^(sat|unsat)'
```

The check passes when:
- every `vassert!` line in `lib.rs` appears as `Assertion passed: .../lib.rs:<line>`;
- there is no `Assertion failed ... backedge!!` line.

If it fails:
- **A `vassert!` line is missing.** That assertion was never checked: it is unreachable,
  or rustc/LLVM folded it to `true`. The test is vacuous. See
  [Keep assertions alive](#keep-assertions-alive).
- **`backedge!!` on a loop line.** The loop runs more iterations than SeaHorn explores, so
  code after it is likely unreachable. See [Bounded loops](#bounded-loops).

Vacuity checking is for `unsat` tests. Hand-registered `sat` tests (`sea_add_sat_test`)
carry the `skipvac` label so a vacuity run skips them.

### 3. Mutation: can it fail?

Break the property on purpose, for example by changing the assertion or the assumption
so it is false. Rebuild and confirm the result is `sat`:

```bash
<build>/verify <build>/src/rust-jobs/<job>/<job>_llvm.bc 2>&1 | grep -E '^(sat|unsat)'   # expect sat
```

Then revert the change and rebuild. If the broken version still gives `unsat`, the test
is not checking what you think it is.

### Other useful options

```bash
<build>/verify --dry-run <bc>    # print the sea command line without running it
<build>/verify --cex <bc>        # produce a counterexample for a sat result
```

## Before submitting a change

For each job you add or change:

1. **Build:** `cmake --build <build> --target <job>.ir` (reconfigure first if you added a job).
2. **Result:** `ctest --test-dir <build> -R '^<job>_' -V` gives the expected sat/unsat.
3. **Vacuity:** `verify --vac` shows every assertion passed and no `backedge!!`.
4. **Mutation:** a deliberately broken version gives `sat`. Revert it and re-run step 2
   on the final code.

## Writing jobs

### Adding a job

1. Run `scripts/add-job <job>`. It creates `src/rust-jobs/<job>/` with a
   `sea_add_unsat_test(<job>)`, and adds the job to `scripts/rust-jobs.json` and
   `src/rust-jobs/CMakeLists.txt` in sorted order. If you copy an existing job by hand
   instead, add it to both lists yourself.
2. For a job that should find a bug, change the test in the job's `CMakeLists.txt` to
   `sea_add_sat_test(<job>)`.
3. Reconfigure with `cmake <build>`, then run the checks above.

For nondeterministic inputs, use `verifier::any!()`. It works for integers, `bool`,
`char`, `Option`, tuples, arrays, and your own types with `#[derive(Arbitrary)]`;
`any_slice`/`any_str` give bounded slices and strings (see `src/rust-jobs/nondet-types`).

Where possible, write jobs in pairs: a correct one (`unsat`) and one with a wrong
assertion or a deliberate bug (`sat`), as the `*-error` jobs do for smallvec and tinyvec.

### Keep assertions alive

Rust jobs are compiled with optimization before SeaHorn sees them. If LLVM can prove an
assertion, it deletes it and the test becomes vacuous. Do not work around this with a new
`assume!`: an assumption constrains the program and must itself be justified. Instead,
pass the value through a volatile load, which LLVM cannot see through but which does not
constrain anything:

```rust
#[inline(always)]
fn opaque<T: Copy>(x: T) -> T {
    unsafe { core::ptr::read_volatile(&x) }
}

// LLVM can fold this away when it knows v == 0 on this path:
verifier::vassert!(sentinel >= (v as u32) * (v as u32));

// This survives, and SeaHorn still sees the real value:
verifier::vassert!(sentinel >= opaque((v as u32) * (v as u32)));
```

### Bounded loops

SeaHorn unrolls loops only up to a bound, so code after a longer loop becomes
unreachable. Set `bound:` in the job's `sea.yaml` to at least the loop's trip count
(for example `src/rust-jobs/smallvec-bound2/sea.yaml`). In harness code (not in the code under test),
you can also give a loop a constant trip count so LLVM unrolls it fully:

```rust
const MAX_NODES: usize = 4;
for _ in 0..MAX_NODES {
    if p.is_null() { break; }
    // ...
}
```

`--vac` reports `backedge!!` when a loop still needs more iterations.

## Contributing

- Keep one logical change per commit, with messages like `test(<job>): ...` or
  `fix(<area>): ...`.
- CI skips tests listed in `.github/workflows/blacklist.txt` (and `blacklist.vac.txt`,
  `blacklist.cex.txt` for the vacuity and counterexample runs). If your change makes a
  blacklisted test pass, remove its line in the same commit.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| New job has no ctest | CMake not reconfigured | `cmake <build>` |
| ctest passes, but `--vac` lists fewer assertions than `lib.rs` has | Assertion unreachable or folded | [Keep assertions alive](#keep-assertions-alive); [bound loops](#bounded-loops) |
| `Assertion failed ... backedge!!` | Loop not fully unrolled | Raise `bound:` in the job's `sea.yaml` |
| `verify` can't find the file | Passed the target name `<job>.ir` | Pass `<job>_llvm.bc` |
| ctest stops with `include could not find requested file: ..._ctest_tests.cmake` | Jobs with per-function tests are not built yet; their test lists are generated at build time | Build everything, or run one job's tests with `ctest --test-dir <build>/src/rust-jobs/<job>` |
| An option in `sea.yaml` has no effect | `sea` ignores unknown option names | Check the spelling; `verify --dry-run` shows what is passed |
