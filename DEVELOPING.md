# Developing verify-rust: build, verify, commit

Every commit that touches a job must pass the four checks in
[Pre-commit checklist](#pre-commit-checklist). A passing `ctest` alone is not enough:
a test can pass because its assertion was never reached.

## Environment

All build and verify commands run **inside the `seahorn-minimal` container**. The
`/home/usea/...` paths do not exist on the Mac host.

| What | Value |
| --- | --- |
| Container | `seahorn-minimal` |
| Image | `ghcr.io/seahorn/seahorn-llvm18:nightly` (amd64, emulated on Apple Silicon) |
| Source mount | host `verify-rust/` → `/home/usea/verify-rust` |
| Build dir | `/home/usea/build` (container only) |
| Verify driver | `/home/usea/build/verify` |
| SeaHorn install | `/home/usea/seahorn` |

Edit files on the host. The mount makes changes visible in the container right away.
Run git on the host.

Start a session:

```bash
docker start -ai seahorn-minimal      # or: docker exec -it seahorn-minimal bash
source $HOME/.cargo/env
export CARGO_TARGET_DIR=/home/usea/cargo-target
```

Run one command from the host without opening a shell:

```bash
docker exec seahorn-minimal bash -lc 'source $HOME/.cargo/env; export CARGO_TARGET_DIR=/home/usea/cargo-target; <command>'
```

## Build

```bash
cmake /home/usea/build                                    # only after adding/removing a job or editing a CMakeLists.txt
cmake --build /home/usea/build --target <job>.ir -j2      # build one job
cmake --build /home/usea/build -j2                        # build everything (slow under emulation)
```

Names are easy to mix up:

| Thing | Name |
| --- | --- |
| Build target | `<job>.ir` |
| Bitcode that SeaHorn verifies | `/home/usea/build/src/rust-jobs/<job>/<job>_llvm.bc` |
| Readable IR (for inspection) | `/home/usea/build/src/rust-jobs/<job>/<job>_llvm.ll` |
| ctest names | `<job>_unsat_test`, `<job>_sat_test`, `<job>_<fn>_unsat_test` |

Build output is full of `resolver = "1"` and `unexpected cfg(kani)` warnings. They are
harmless; look at the last few lines for errors.

## Verify

### 1. Result: does it return what the test expects?

```bash
ctest --test-dir /home/usea/build -R '^<job>_' -V
```

`sea_add_unsat_test` expects `unsat` (no assertion can fail). `sea_add_sat_test`
expects `sat` (a counterexample exists). `sea_add_unsat_test_novac` is an unsat test that
is excluded from the vacuity CI leg. ctest "Passed" means the result matched.

### 2. Vacuity: was every assertion actually checked?

```bash
/home/usea/build/verify --vac /home/usea/build/src/rust-jobs/<job>/<job>_llvm.bc 2>&1 | grep -E 'Assertion|^(sat|unsat)'
```

Pass when:
- every `vassert!` line in `lib.rs` appears as `Assertion passed: .../lib.rs:<line>`;
- there is no `Assertion failed ... backedge!!` line.

What failure looks like:
- **A `vassert!` line is missing.** That assertion was never checked: it is unreachable, or
  rustc/LLVM folded it to `true`. The test is vacuous.
- **`backedge!!` on a loop line.** The loop runs more iterations than SeaHorn explores.
  Code after the loop is likely unreachable. See [Bounded loops](#bounded-loops).

Skip `--vac` for `sat` tests (they carry the `skipvac` label).

### 3. Mutation: can it fail?

Break the property on purpose and confirm `sat`, then revert. Back up first:

```bash
cp src/rust-jobs/<job>/lib.rs /tmp/lib.rs.bak    # on the host
# edit the assertion or the assumption so it is false, e.g. v*v -> v*v + 1
# rebuild <job>.ir, then:
/home/usea/build/verify /home/usea/build/src/rust-jobs/<job>/<job>_llvm.bc 2>&1 | grep -E '^(sat|unsat)'   # expect sat
cp /tmp/lib.rs.bak src/rust-jobs/<job>/lib.rs     # revert, then rebuild
```

If the mutant still gives `unsat`, the test is not checking what you think.

### Other useful runs

```bash
/home/usea/build/verify --dry-run <bc>    # print the sea command line without running it
/home/usea/build/verify --cex <bc>        # produce a counterexample for a sat result
```

## Pre-commit checklist

1. **Build:** `cmake --build /home/usea/build --target <job>.ir -j2` (re-run
   `cmake /home/usea/build` first if you added a job).
2. **Result:** `ctest --test-dir /home/usea/build -R '^<job>_' -V` gives the expected sat/unsat.
3. **Vacuity:** `verify --vac` shows every assertion passed and no `backedge!!`.
4. **Mutation:** a deliberately broken version gives `sat`. Revert, rebuild, re-run step 2
   on the final code.

Then commit on the host.

## Writing jobs that verify for real

### Adding a job

1. Scaffold with `scripts/add-job`, or copy an existing small job (e.g. `src/rust-jobs/add`).
2. Register it in `src/rust-jobs/CMakeLists.txt` and `scripts/rust-jobs.json`.
3. In the job's `CMakeLists.txt`, add `sea_add_unsat_test(<job>)` or `sea_add_sat_test(<job>)`.
4. Re-run `cmake /home/usea/build`, then the checklist.

Write jobs in pairs where possible: a correct one (`unsat`) and one with a wrong
assertion (`sat`), e.g. `raw-pointers` / `raw-pointers-bug`.

### Keep assertions alive past rustc

Rust jobs are compiled at `opt-level = 2` before SeaHorn sees them. If rustc can prove
an assertion, it deletes it and the test is vacuous. Assert against an opaque value
tied to the real one only through `assume!`, which rustc cannot see through
(pattern from commit 19ebae2):

```rust
// Folded away by rustc:
verifier::vassert!(sentinel >= (v as u32) * (v as u32));

// Survives:
let bound: u32 = verifier::any!();
verifier::assume!(bound == (v as u32) * (v as u32));
verifier::vassert!(sentinel >= bound);
```

### Bounded loops

SeaHorn cuts loops (`--horn-cut-loops`), so a loop with no constant bound gets about one
iteration. A `while !p.is_null()` over a 3-node list makes everything after it unreachable.
Give loops a constant trip count so LLVM unrolls them fully:

```rust
const MAX_NODES: usize = 4;
for _ in 0..MAX_NODES {
    if p.is_null() { break; }
    // ...
}
```

`--vac` reports `backedge!!` when a loop still needs more iterations.

## Git workflow

- Branch: `jamie/dev`. Remotes: `origin` = github.com/dwseoh/verify-rust (push here),
  `upstream` = github.com/seahorn/verify-rust (read only; PRs go here).
- One task per commit. Message style: `test(<job>): ...`, `fix(config): ...`.
- Before committing, check `git status`: stage only that task's files.
- CI skips tests listed in `.github/workflows/blacklist.txt`. When a fix makes a
  blacklisted test pass, remove its line in the same commit.
- Upstream CI has been red since 2026-08-09 (8 vec tests return `sat`, caused by a base
  image change). Compare against your local run, not CI, until that is resolved.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `/home/usea/build is not a directory` | Command ran on the Mac host | Run it inside `seahorn-minimal` |
| `docker ps` hangs | Docker Desktop daemon stuck | Quit and restart Docker Desktop |
| New job has no ctest | CMake not reconfigured | `cmake /home/usea/build` |
| ctest passes, `--vac` lists fewer assertions than `lib.rs` has | Assertion unreachable or folded | Opaque-bound pattern; bound loops |
| `Assertion failed ... backedge!!` | Loop not fully unrolled | Constant trip count |
| `verify` can't find the file | Used `<job>.ir` | Run on `<job>_llvm.bc` |
| An option in `sea.yaml` has no effect | `sea` silently drops unknown option names | Check the name in `seahorn/py/sea/commands.py`; `verify --dry-run` shows what is passed |
