# Moving from SeaHorn dev14 to dev18

Status: **builds, does not pass.** The LLVM 14 baseline is captured (208/213, red
set == the CI blacklist exactly) and the LLVM 18 switch is made — verify-rust
compiles clean against dev18, 435/435 targets. But **25 tests that were green under
dev14 are red under dev18**, in three distinct classes. See
[Result against dev18](#result-against-dev18-25-new-failures).

The headline for anyone picking this up: *every problem encountered getting it to
**build** was a Rust/C toolchain problem, not a SeaHorn one. Every problem that
remains is a **verification** problem.*

## Why this is not a one-line change

The instinct is that pointing `SEAHORN_ROOT` at a dev18 build is the whole job. It
is not. This project's contract with SeaHorn is **LLVM bitcode**, and bitcode is
version-coupled in both directions:

```
rustc's bundled LLVM  ──emits──►  .bc  ◄──reads──  SeaHorn's LLVM
                clang ──emits──►  .bc
```

All three must agree on the LLVM major version. SeaHorn dev18 is built against
LLVM 18, so *everything upstream of it has to move to 18 at the same time* — you
cannot feed LLVM 14 bitcode to a dev18 `seaopt`. That is why the Rust toolchain is
part of an "LLVM" migration at all, which is the non-obvious bit.

Concretely, the version is pinned in **five** places that must move together:

| Where | What | LLVM 14 value |
|---|---|---|
| `rust-toolchain.toml` | Rust channel | `nightly-2022-08-01` |
| `CMakeLists.txt:17` | `Rust_TOOLCHAIN` (Corrosion) | `nightly-2022-08-01` |
| `CMakeLists.txt:29-39` | `llvm-nm`, `llvm-link`, `llvm-objcopy`, `llvm-dis`, `lld` | `*-14` |
| `Cargo.toml:19-20,32-33` | rustc linker flags (both `*-seahorn` profiles) | `clang-14`, `lld-14` |
| `CMakePresets.json:13-14`, `.github/workflows/main.yml` | C/C++ compiler | `clang-14` |

Miss one and you get a confusing mid-build failure rather than a clean "wrong
version" error.

## Picking the Rust nightly

The constraint is *"the nightly whose bundled LLVM matches SeaHorn's"*, so pick by
LLVM version, not by date. Rust used LLVM 18 from 1.78 through 1.81, then moved to
LLVM 19 partway through the 1.82 nightly cycle (mid-August 2024). So a July 2024
nightly is comfortably inside the LLVM 18 window.

**Verified on this machine** — never trust the table, just ask rustc:

```bash
$ rustc +nightly-2024-07-01 --version --verbose | grep -E "release|LLVM"
release: 1.81.0-nightly
LLVM version: 18.1.7
```

For comparison, the current pin:

```bash
$ rustc +nightly-2022-08-01 --version --verbose | grep -E "release|LLVM"
release: 1.64.0-nightly
LLVM version: 14.0.6
```

So: **`nightly-2024-07-01` is the dev18 counterpart of `nightly-2022-08-01`.**

### On 18.1.7 vs 18.1.3
rustc 1.81's LLVM is 18.1.7; noble's `clang-18` is 18.1.3. This is fine — bitcode
compatibility is a function of the LLVM **major** version, and patch releases do
not change the format. Do not burn time trying to make these match exactly.

### Cargo renamed lib targets underneath us (`add-lib` vs `add_lib`)

The first real casualty of the toolchain jump, and a good illustration of why the
risk lives in Rust rather than in SeaHorn. Symptom at configure time:

```
CMake Error at .../Corrosion.cmake:1242 (set_property):
  set_property could not find TARGET add-lib.  Perhaps it has not yet been created.
Call Stack:
  cmake/SeaHornBuild.cmake:28 (corrosion_set_cargo_flags)
```

Ask cargo directly and the cause is plain — the same workspace, two toolchains:

| | cargo 1.64 (`nightly-2022-08-01`) | cargo 1.81 (`nightly-2024-07-01`) |
|---|---|---|
| `workspace_members[0]` | `add-lib 0.1.0 (path+file:///…)` | `path+file:///…#add-lib@0.1.0` |
| lib target name | `add-lib` | `add_lib` |

Two independent changes: the package-ID format (harmless here — Corrosion compares
`workspace_members` against `packages[].id`, and both come from the same metadata,
so they stay consistent), and **the lib target name, which cargo now normalises by
replacing `-` with `_`**. That one matters, because Corrosion names its CMake
targets after the *cargo lib target*, not the package.

`c_rust_llvm` was using one name for both roles:

```cmake
set(RUST_LIB_TARGET ${TARGET}-lib)
corrosion_import_crate(... CRATES ${RUST_LIB_TARGET} ...)   # package name  -> needs `add-lib`
corrosion_set_cargo_flags(${RUST_LIB_TARGET} ...)           # CMake target  -> now `add_lib`
```

Under cargo 1.64 both were `add-lib` and the conflation was invisible. Fixed by
splitting the two roles in `cmake/SeaHornBuild.cmake`:

```cmake
set(RUST_LIB_PACKAGE ${TARGET}-lib)                          # cargo package
string(REPLACE "-" "_" RUST_LIB_TARGET ${RUST_LIB_PACKAGE})  # CMake target
```

**Bumping Corrosion does not fix this** — tested v0.5.2, which creates `add_lib`
too. The name comes from cargo's metadata, not from Corrosion's naming policy. So
this fix is needed regardless of the Corrosion decision.

`corrosion_import_crate`'s `IMPORTED_CRATES` output would let you *ask* Corrosion
what it created rather than deriving the name — more robust in principle, but it
hard-errors when `CORROSION_NATIVE_TOOLING` is ON, so it trades one fragility for
another. The dash→underscore rule is documented cargo behaviour and stable.

### ahash `=0.8.6` cannot build on any 2024 nightly

The second casualty, and a nice example of a pin that is load-bearing in one
direction only. Symptom:

```
error[E0635]: unknown feature `stdsimd`
  --> .../ahash-0.8.6/src/lib.rs:99:42
99 | #![cfg_attr(feature = "stdsimd", feature(stdsimd))]
error: could not compile `ahash` (lib) due to 1 previous error
```

Confusingly, `stdsimd` is **not a declared cargo feature** of ahash 0.8.6 — it is
absent from `[features]` entirely. The culprit is its `build.rs`:

```rust
if let Some(true) = version_check::supports_feature("stdsimd") {
    println!("cargo:rustc-cfg=feature=\"stdsimd\"");
}
```

It probes *"am I on a nightly?"* and, if so, force-enables the cfg. The probe
assumes every nightly has the `stdsimd` gate. Rust **removed** that gate in early
2024, so on any 2024 nightly ahash 0.8.6 turns on a feature gate that no longer
exists and fails to compile. ahash dropped the probe in 0.8.7. There is no
workaround at 0.8.6 — the version is simply incompatible with the target
toolchain.

**Why the pin was there.** `git log -L 20,20:src/rust-jobs/borsh/Cargo.toml`
shows `ahash = "=0.8.6"` was added in `d1ca2d4` ("Update ahash version",
Jan 2024) — the same month ahash 0.8.7 shipped. It was added to hold ahash *back*
so it kept working on `nightly-2022-08-01`. That is a **toolchain-compatibility
pin, not a verification-semantics pin**, so moving it forward is part of this
migration rather than a violation of its intent. Worth checking that history
before touching an exact pin: if it had been pinned to fix what the proof
verifies, the right move would have been different.

Bumped to **`=0.8.11`**, keeping the project's exact-pin style. The `Cargo.lock`
diff is exactly two lines.

**Do not jump straight to the latest (0.8.12) — it breaks the build in a way that
looks unrelated.** ahash 0.8.12 moved to `getrandom` 0.3, which drags in `wasip2`
→ `wit-bindgen 0.57.1`, and that crate requires `edition2024`:

```
error: failed to parse manifest at `.../wit-bindgen-0.57.1/Cargo.toml`
Caused by:
  feature `edition2024` is required
  ... not stabilized in this version of Cargo (1.81.0-nightly)
```

The trap is *where* this surfaces. The Rust libraries still build fine, because
`cargo rustc --lib` never touches a wasi-only dependency. It breaks **cbindgen**,
which shells out to `cargo metadata` — and metadata resolves the entire lockfile,
edition and all. So the symptom is a C compile error about a missing prototype
(cbindgen silently produced no header), three steps removed from the cause. If a
header comes out empty, run cbindgen by hand in the job directory and read its
stderr; it reports the real error.

ahash 0.8.7-0.8.11 stay on `getrandom` 0.2 and avoid the whole cascade. 0.8.11 is
the newest version compatible with cargo 1.81.

### Nightly feature gates that moved

Two of the six `#![feature(...)]` gates in the tree no longer exist on
`nightly-2024-07-01`. Both are project source, both fixed:

| Gate | Fate | Fix |
|---|---|---|
| `is_some_with` | **stabilised** as `Option::is_some_and` (Rust 1.70) | drop the gate; the stable signature takes `T` by value, not `&T`, so `\|num: &i32\| *num > 0` becomes `\|num: i32\| num > 0` |
| `core_panic` | **renamed** to `panic_internals` | dropped the gate — `panic-test` only uses `panic!()`, which needs no gate in `no_std`; the gate was for the commented-out `core::panicking::panic` call |
| `new_uninit`, `slice_ptr_get`, `alloc_error_handler`, `core_intrinsics` | still unstable, still fine | none |

On `core_panic` — renaming to `panic_internals` also works, but the live code
doesn't need any gate, and the renamed gate warns "internal to the compiler or
standard library" on every build. If someone uncomments the `core::panicking` line
they get a precise E0658 naming the feature, which is better than a dead gate.

**Testing gates: watch out for a false negative.** Probing a gate with a bare
`#![no_std]` crate wrongly reports library features as removed:

```bash
# WRONG: reports `new_uninit` as unknown, because it is a library feature of alloc
printf '#![no_std]\n#![feature(new_uninit)]\n' > t.rs && rustc --crate-type=rlib t.rs

# RIGHT: link the crate that declares the feature
printf '#![no_std]\n#![feature(new_uninit)]\nextern crate alloc;\n' > t.rs && rustc ...
```

Lang features live in the compiler; library features are declared with
`#[unstable(feature = "...")]` in core/alloc/std and are only visible if that crate
is linked. Do not "fix" a job on the strength of the first form.

### clang 16+ makes implicit function declarations an error

Not a SeaHorn or Rust change — a C change, which caught a latent bug:

```
result-unwrap_or_else.c:11:4: error: call to undeclared function 'entrypt';
  ISO C99 and later do not support implicit function declarations
```

`src/rust-jobs/result-unwrap_or_else/lib.rs` declared its entry point as bare
`fn entrypt()`, unlike every other job's `pub extern "C" fn entrypt()`. cbindgen
only emits declarations for `pub extern "C"` functions, so the generated
`inc/lib.h` for that job was **empty** and the C caller had no prototype.
`#[no_mangle]` still exported the symbol, so it linked, and clang-14 merely warned.
clang-16 promoted implicit declarations from warning to error, so it became fatal.

Fixed at the cause (`pub extern "C" fn entrypt()`, matching all 66 other jobs)
rather than by silencing the diagnostic with
`-Wno-implicit-function-declaration`. It was the only job with this shape.

### Cargo.lock is now toolchain-specific

A consequence worth stating plainly: after the ahash bump, `Cargo.lock` no longer
resolves for `nightly-2022-08-01`. The lockfile is shared between all jobs, so the
LLVM 14 configuration and the LLVM 18 configuration cannot both build from this
tree. **The baseline is not re-runnable after this point** — which is exactly why
it was captured first, and why the numbers are written down above rather than left
implicit in a build directory.

### Why the toolchain bump is the risky part
`CMakeLists.txt:17` carries the comment *"Our preferred toolchain for which fat lto
works"*, and the build leans on unstable machinery that is free to drift between a
2022 and a 2024 nightly:

- `cargo-features = ["profile-rustflags"]` (`Cargo.toml:1`) — unstable cargo gate
- `-Zbuild-std=panic_abort,std` and `-Zbuild-std-features=panic_immediate_abort`
  (`cmake/SeaHornBuild.cmake`)
- `-Zemit-thin-lto=no`, `-Clinker-plugin-lto` (`Cargo.toml`)

**Expect breakage here that has nothing to do with dev18.** This is the single
biggest reason the baseline below is worth having: without it, a red test after the
switch is ambiguous between "dev18 regressed something", "the 2024 nightly changed
codegen", and "this test was already red".

## What "green" means

`ctest` registers **213** tests. Not all are expected to pass: CI excludes a
blacklist per mode (`.github/workflows/`), and the default (no `VERIFY_FLAGS`) mode
excludes 5 via `blacklist.txt`:

```
cfg-simplify_unsat_test
custom-vec-raw_vec_unsat_test
smallvec-allocation_test_insert_from_slice_unsat_test
smallvec-allocation_test_with_capacity_unsat_test
tinyvec-arrayvec_test_retain_unsat_test
```

So default-mode green is **208 / 213**. There are separate blacklists for the `cex`
and `vac` matrix legs. Note also that no job calls `sea_disable_unsat_test` /
`sea_disable_sat_test` — the exclusion lives entirely in the CI blacklists, so a
plain local `ctest` will show those 5 as failures and that is *correct behaviour*,
not a broken checkout.

CI runs with `--timeout 2000`. ctest's default is 1500s, and several tinyvec tests
legitimately take 100-170s, so use the CI timeout when comparing runs or you will
manufacture timeouts that CI does not see.

## Baseline procedure (LLVM 14)

The point of the baseline is to record which tests are red *before* changing
anything, on this machine, with this checkout. Two SeaHorn trees are kept
side by side so the comparison is honest:

```
~/seahorn/seahorn          # dev18 (your working checkout)
~/seahorn/seahorn-dev14    # git worktree on dev14, for the baseline only
```

Created with `git worktree add ../seahorn-dev14 dev14`, which leaves the dev18
checkout untouched. See [building-on-ubuntu-24.04.md](building-on-ubuntu-24.04.md)
for the toolchain workarounds each build needs — none of them are optional on 24.04.

```bash
# SeaHorn dev14 (note: Clam is mandatory on dev14, see the other note)
cd ~/seahorn/seahorn-dev14 && mkdir build-rel && cd build-rel
cmake .. -GNinja \
  -DCMAKE_INSTALL_PREFIX=run -DCMAKE_BUILD_TYPE=RelWithDebInfo \
  -DCMAKE_C_COMPILER=clang-14 -DCMAKE_CXX_COMPILER=clang++-14 \
  -DCMAKE_C_FLAGS="--gcc-toolchain=$SHIM" -DCMAKE_CXX_FLAGS="--gcc-toolchain=$SHIM" \
  -DSEA_ENABLE_LLD=ON \
  -DZ3_ROOT=~/seahorn/seahorn/deps/z3-4.8.9 \
  -DYICES2_HOME=~/seahorn/seahorn/deps/yices-2.6.1
cmake --build . --target extra && cmake ..
cmake --build . --target crab  && cmake ..
cmake --build . --target install

# verify-rust against it
cd ~/seahorn/verify-rust && mkdir build-14 && cd build-14
cmake .. -GNinja \
  -DCMAKE_C_COMPILER=clang-14 -DCMAKE_CXX_COMPILER=clang++-14 \
  -DCMAKE_C_FLAGS="--gcc-toolchain=$SHIM" -DCMAKE_CXX_FLAGS="--gcc-toolchain=$SHIM" \
  -DRust_COMPILER=$(rustup which +nightly-2022-08-01 rustc) \
  -DRust_CARGO=$(rustup which +nightly-2022-08-01 cargo) \
  -DSEAHORN_ROOT=~/seahorn/seahorn-dev14/build-rel/run
ninja && ctest -j8 --timeout 2000
```

Prerequisites beyond the README: `sudo apt install clang-14 lld-14 llvm-14-dev
llvm-14-tools libpolly-14-dev`, `rustup install nightly-2022-08-01`,
`rustup component add rust-src --toolchain nightly-2022-08-01`, and
`cargo install --locked cbindgen --version ^0.26` (version per
`docker/verify-rust.Dockerfile`).

## Baseline result (LLVM 14 / SeaHorn dev14)

Recorded 2026-07-16 on Ubuntu 24.04, `ctest -j8` (ctest default 1500s timeout, not
CI's 2000s):

```
98% tests passed, 5 tests failed out of 213
```

| Test | Result | On `blacklist.txt`? |
|---|---|---|
| `cfg-simplify_unsat_test` | Failed | yes |
| `custom-vec-raw_vec_unsat_test` | Timeout | yes |
| `smallvec-allocation_test_insert_from_slice_unsat_test` | Failed | yes |
| `smallvec-allocation_test_with_capacity_unsat_test` | Failed | yes |
| `tinyvec-arrayvec_test_retain_unsat_test` | Timeout | yes |

The failure set is **exactly** the default-mode blacklist — no more, no less. So
the checkout reproduces CI's expectation (208/213) on this machine, and any *new*
red after the dev18 switch is attributable to the port rather than to pre-existing
rot or to the host environment. That is the whole value of doing this first.

(The two timeouts are blacklisted as `#(Timeout)` in the `cex` list too, so they are
slow by nature; they were not manufactured by using 1500s instead of CI's 2000s.)

## Current state

**Done**

- Rust `nightly-2022-08-01` (LLVM 14.0.6) and `nightly-2024-07-01` (LLVM 18.1.7)
  both installed with `rust-src`; the 18 counterpart is confirmed empirically.
- cbindgen 0.26.0.
- SeaHorn dev14 built and installed at `~/seahorn/seahorn-dev14/build-rel/run`.
- verify-rust builds clean against dev14 (435/435 targets).
- **Baseline captured: 208/213, failures == blacklist exactly.** See above.

- The five-place LLVM 18 switch, plus the four fixes described above
  (`SeaHornBuild.cmake` target naming, ahash, two feature gates, `entrypt`).
- verify-rust builds clean against dev18 from a **fresh** build directory —
  435/435 targets, zero errors, same target count as the LLVM 14 baseline.

**Not done**

- `ctest` against dev18 and the diff against the baseline (in flight).
- Decide whether to bump Corrosion (`CMakeLists.txt` `GIT_TAG v0.3.5`) so it works
  with rustup ≥ 1.28 without the `Rust_COMPILER`/`Rust_CARGO` override. Note v0.5.2
  was tried and does **not** fix the lib-target naming — that is cargo's doing, not
  Corrosion's — so this is purely about the rustup parsing.
- CI and `docker/verify-rust.Dockerfile` still target LLVM 14
  (`FROM seahorn/seahorn-llvm14:nightly`, `-DCMAKE_C_COMPILER=clang-14`). They need
  a dev18 base image before CI can pass; deliberately left alone for now, since
  flipping the compiler without the image would only break CI differently.
- The `default-jammy` CMake preset is now a misnomer: it names Ubuntu 22.04 but
  carries clang-18, which jammy does not ship.

## The dev18 `run/` tree is not self-contained (libz3 at runtime)

Symptom: **every** test fails instantly (213/213 in ~2 seconds). That timing is the
tell — real verification failures take seconds to minutes each, so a whole suite
dying in two seconds is the harness, not the solver.

```
seapp: error while loading shared libraries: libz3.so.4.8: cannot open shared object file
```

Compare the `RUNPATH` of the two installs:

```bash
readelf -d <seahorn>/build-rel/run/bin/seapp | grep -i runpath
```

| Build | RUNPATH |
|---|---|
| `seahorn-dev14` (built here, with `-DZ3_ROOT=`/`-DYICES2_HOME=`) | `$ORIGIN/../lib:/usr/lib/llvm-14/lib:…/deps/z3-4.8.9/lib:…/deps/yices-2.6.1/lib` |
| `seahorn` dev18 (pre-existing build) | `$ORIGIN/../lib:/usr/lib/llvm-18/lib` |

Neither install copies `libz3.so.4.8` into `run/lib` — the dev14 build merely
embeds the deps directories in its RUNPATH because those paths were passed at
configure time. The dev18 build did not, so its `seapp`/`seahorn` cannot find z3 or
yices at runtime.

Workaround when running against dev18:

```bash
export LD_LIBRARY_PATH=~/seahorn/seahorn/deps/z3-4.8.9/lib:~/seahorn/seahorn/deps/yices-2.6.1/lib
```

This keeps the comparison honest: both configurations load the *same*
`deps/z3-4.8.9` and `deps/yices-2.6.1` binaries, so solver version is not a
variable between baseline and dev18. The durable fix is to reconfigure the dev18
build with `-DZ3_ROOT=` / `-DYICES2_HOME=` so the RUNPATH is embedded, or to ship
the solver libs in `run/lib`.

## Driving `verify` by hand (outside ctest)

Debugging a single failure through ctest is slow and hides the output. The `verify`
driver takes a `.bc` directly, and every job's module is already built:

```bash
export LD_LIBRARY_PATH=~/seahorn/seahorn/deps/z3-4.8.9/lib:~/seahorn/seahorn/deps/yices-2.6.1/lib
BC=./build-18/src/rust-jobs/sea-vec/sea-vec_llvm.bc

./build-18/verify --expect=unsat --entry=test_zst      $BC   # just the verdict
./build-18/verify --cex          --entry=test_zst      $BC   # counterexample trace
./build-18/verify --vac          --entry=test_zst      $BC   # is it vacuous?
```

Two things to know before you trust a by-hand run:

- **`--entry` matters, and a wrong name fails silently.** A job's module holds
  every `#[no_mangle]` entry in `lib.rs`. Passing an entry that does not exist does
  **not** produce an error — it hangs. I burned a debugging round on
  `--entry=test_zst_len`, an entry I had invented, and read the hang as a dev18
  blow-up. Get the list from the source, not from memory:

  ```bash
  grep -n -A3 '#\[no_mangle\]' src/rust-jobs/<job>/lib.rs | grep 'fn '
  ```

- **The baseline times are already on disk** — no need to re-run dev14 to get them:

  ```bash
  grep <name> build-14/Testing/Temporary/CTestCostData.txt   # e.g. sea-vec_test_zst_unsat_test 1 1.20267
  ```

  Every `sea-vec` entry is ~1.1-1.25s on dev14. Check this *before* choosing a
  timeout: if a test's baseline is one second, a two-minute hang is never about the
  timeout, and treating it as one sends you down the wrong path.
- **The by-hand command is not necessarily the ctest command.** ctest adds flags
  (see `sea_discover_unsat_tests` in `cmake/SeaHornBuild.cmake`). If a by-hand run
  disagrees with the suite, diff the two command lines before believing either —
  `ctest -R <name> -V` prints what it actually ran.

Run with no arguments to get the flag list; it is a small argparse CLI.

## Result against dev18: 25 new failures

**The port builds, but it is not green.** Run: `ctest -j16 --timeout 600`, excluding
the two baseline tests already known to time out (`custom-vec-raw_vec_unsat_test`,
`tinyvec-arrayvec_test_retain_unsat_test`), so 211 of 213 ran.

```
87% tests passed, 28 tests failed out of 211
```

| | dev14 baseline | dev18 |
|---|---|---|
| red | 5 (== `blacklist.txt` exactly) | 28 |
| of which pre-existing | — | 3 |
| **new** | — | **25** |

The 25 new failures fall into three classes, and they are *not* equally serious.

### What about thin LTO? (it is not the cause)

Worth ruling out explicitly, because `CMakeLists.txt:17` pins the toolchain with the
comment *"Our preferred toolchain for which fat lto works"*, which makes LTO the
obvious suspect when a toolchain bump breaks verification.

The pipeline is:

| Stage | Flag | Where |
|---|---|---|
| rustc emits **fat** LTO bitcode | `-Clinker-plugin-lto`, `-Zemit-thin-lto=no` | `Cargo.toml:18,21,31,34` |
| clang emits bitcode | `-flto` | `SeaHornBuild.cmake:70,87` |
| lld merges + embeds it | `-Wl,--plugin-opt=-lto-embed-bitcode=post-merge-pre-opt` | `SeaHornBuild.cmake:89` |

That embedded post-merge module is exactly what SeaHorn verifies.

**Evidence that LTO is still working on the 2024 nightly:** the merged
`option-or_llvm.ll` contains `entrypt`'s Rust body with Rust debug info, and
build-14's contains a `define` (not `declare`) of the Rust `Into::into`. Both are
properly fat-merged modules. If `-Clinker-plugin-lto` had silently stopped taking
effect, the staticlib would be native objects and no Rust IR would reach the
embedded module at all. Since `-Clinker-plugin-lto` and `-Zemit-thin-lto=no` live
in the same `rustflags` list, the list is demonstrably applied. cargo also errors on
unknown `cargo-features`, and rustc errors on unknown `-Z` flags, so
`profile-rustflags` and `-Zemit-thin-lto=no` are both still accepted.

**Measured, not inferred.** `notes/lto-flag-probe.sh` compiles a trivial staticlib
under each flag combination and classifies the emitted bitcode by looking for a
ThinLTO `GLOBALVAL_SUMMARY_BLOCK` (present = thin, absent = fat):

| flags | rustc 1.81 (our pin, LLVM 18.1.7) | rustc 1.99 (LLVM 22.1.8) |
|---|---|---|
| `-O` alone | NATIVE | NATIVE |
| `-Clinker-plugin-lto` | THIN | THIN |
| `-Clinker-plugin-lto -Zemit-thin-lto=no` | **FAT** | **ERROR — option removed** |
| `-Clinker-plugin-lto -Clto=fat` | **THIN** | **FAT** |
| `-Clinker-plugin-lto -Clto=thin` | THIN | THIN |

So on our pinned 1.81 the flag is **doing real work**: it is the only thing turning
thin-prepared bitcode into fat, and `-Clto=fat` does *not* substitute for it. The
build is genuinely getting the fat whole-program module SeaHorn needs, which rules
thin LTO out as a cause of the 25 failures.

**But the flag is gone upstream.** On current nightly `-Zemit-thin-lto` is removed
outright (`error: unknown unstable option`), and `-Clto=fat` alongside
`-Clinker-plugin-lto` now yields fat bitcode on its own (rust-lang/rust#136840).

Consequence for any future toolchain bump: moving past ~1.81 **requires** swapping
`-Zemit-thin-lto=no` for `-Clto=fat` in both `*-seahorn` profiles in `Cargo.toml`.
Miss it and rustc hard-errors on the removed flag — noisy, so it cannot pass
silently. Do **not** make that swap while pinned to 1.81: there `-Clto=fat` silently
yields THIN bitcode, which would quietly change what SeaHorn verifies. The two
changes must move together with the pin.

### Measuring fat vs thin: two traps

Both cost real time here, and both produce confident-looking wrong answers:

1. **A rustc staticlib bundles hundreds of native `.o` members** (std,
   compiler_builtins) even under `-Clinker-plugin-lto` — typically ~170-330 ELF
   objects against 1-2 bitcode ones. Inspecting the first `.o` you find reports
   native ELF and tells you nothing. Check *every* member; skip non-`BC` magic;
   judge on the bitcode ones only.
2. **Never reuse an output path or extraction directory across probes.** If a
   later rustc rejects its flags and a stale artifact from an earlier run is still
   on disk, you will measure the old artifact and conclude a removed flag still
   works. This exact bug reported `-Zemit-thin-lto=no` as FAT on rustc 1.99, where
   it does not exist at all.

### The real mechanism: an identity function stopped being opaque

Diffing the post-merge IR for `option-or` between `build-14/` and `build-18/` gives
the whole story, and it is narrower than "LLVM 18 optimises harder":

```llvm
; build-14 — assertion reaches SeaHorn with real content
%5 = select i1 %4, i32 %1, i32 %2                  ; val1.or(val2).unwrap()
%7 = icmp eq i32 %5, %1                            ; result == v
%8 = call i1 @"_ZN..core..convert..Into..4into.."(i1 %7)   ; NOT inlined -> opaque
call void @__VERIFIER_assert(i1 %8)

; build-18 — nothing left to prove
call void @__VERIFIER_assert(i1 true)
```

`Into::into` is the **identity function** (`i1 noundef returned zeroext %0`). In
build-14 it is defined in the module but not inlined, and that opaque call is the
only thing preventing LLVM from folding `icmp eq (select %4, %1, %2), %1` to `true`
inside the branch where `%4` holds. In build-18 it is inlined, the fold happens, and
the assertion evaporates.

So these proofs were **only ever working by accident** — one inliner improvement away
from going vacuous. dev18 is what finally triggered it; it did not cause it. The same
would have happened on LLVM 14 had the inliner been slightly more eager.

This reframes Class 2 as well: fully-inlined bodies are also a plausible mechanism
for the query blow-ups. Diff the IR before theorising — the technique is cheap:

```bash
diff <(sed -n '/define.*@entrypt/,/^}/p' build-14/src/rust-jobs/<job>/<job>_llvm.ll) \
     <(sed -n '/define.*@entrypt/,/^}/p' build-18/src/rust-jobs/<job>/<job>_llvm.ll)
```

### Class 1 — assertion discharged by the front end (11 tests)

`option-or`, `option-unwrap-or`, `result-iter_mut`, `serde`,
`tinyvec-arrayvec-aop_test_{clear,extend_from_slice,from_array_empty,from_array_len,new,set_len,try_from_array_len}`

```
WARNING: no assertion was found so either program does not have assertions
         or front-end discharged them.
```

SeaHorn finds **no assertion to prove**, never prints `unsat`, so `--expect=unsat`
fails. These properties are true by construction (e.g. `option-or` asserts
`val1.or(val2).unwrap() == v` for even `v`), and the LLVM 18 optimiser is simply
strong enough to fold them away where LLVM 14 was not.

Note what this means: the tests **fail loudly rather than pass vacuously**, which is
the good outcome. But the underlying issue is that these proofs no longer exercise
SeaHorn at all. Silencing them by relaxing the expectation would be the wrong fix —
it would convert a visible problem into an invisible one. The job needs its input
made opaque to the optimiser (that is what `verifier::any!` is for) so the property
survives to BMC.

### Full-suite validation after the fixes (2026-07-19)

Ran all 213 via `ctest -j30 --timeout 400` (no `-E`), `LD_LIBRARY_PATH` set, with
all fixes applied (provenance in sea_vec/custom-vec, `devirt-functions: sea-dsa`):
**190 passed, 14 failed, 9 timeout.** The 400s cap is *stricter* than CI's 2000s,
yet failures dropped from the original dev18 run's 28 to 23 (14+9).

**No regressions.** The 14 fast failures are 3 blacklisted (cfg-simplify,
smallvec-allocation_test_{with_capacity,insert_from_slice}) + 11 Class 1
"no assertion found" (option-or, option-unwrap-or, result-iter_mut, serde, and 7
tinyvec-arrayvec-aop_test_*). Those 11 are exactly the original Class 1 set. Spot-
checked `tinyvec-arrayvec-aop_test_new`: identical "no assertion found" under both
`devirt=types` and `devirt=sea-dsa`, so the devirt change caused none of them.

**What the fixes resolved:** sea-vec_test_zst (Class 3, provenance) now unsat;
copy_stack_buffer 6.5s, custom-print 53s, borsh_test_fixed_sized_arrays 8.9s
(devirt) all pass; tinyvec-arrayvec_test_retain (309s) and
smallvec_test_insert_from_slice (392s) pass under the 400s cap.

**The 9 remaining timeouts are solver-bound** (not devirt, not frontend):
borsh_test_string, borsh-{enum,option,primitives,structs}, custom-vec_test_deref,
custom-vec_test_insert, sea-vec_test_macros, tinyvec-arrayvec-aop_test_retain.
Caveat: a few of these might pass under CI's 2000s cap — re-run with 2000s to
separate "genuinely unbounded" from "slow but finite" before deep-diving each.

Watch-outs for the eventual commit:
- The suite ran against the **build-dir** `sea.yaml` copy (synced by hand). A clean
  rebuild re-copies from `src/`, which is already fixed — fine.
- 400s is not CI's 2000s; the 9 timeouts are the honest residual at 400s, and 2 of
  the passers (tinyvec-retain, smallvec-insert) only just squeaked under.

### Class 2 — blow-up / timeout (13 tests)

`borsh_test_fixed_sized_arrays`, `borsh_test_string`, `borsh-enum`, `borsh-option`,
`borsh-primitives`, `borsh-structs`, `copy_stack_buffer`, `custom-print`,
`custom-vec_test_deref`, `custom-vec_test_insert`, `sea-vec_test_macros`,
`smallvec_test_insert_from_slice`, `tinyvec-arrayvec-aop_test_retain`

These are **not** artifacts of the 600s limit — the baseline times make that plain:

| test | dev14 | dev18 |
|---|---|---|
| `sea-vec_test_macros` | 1.21s | >605s |
| `copy_stack_buffer` | 4.05s | >611s |
| `custom-print` | 4.59s | >612s |
| `borsh-structs` | 10.07s | >611s |
| `custom-vec_test_deref` | 12.86s | >612s |
| `smallvec_test_insert_from_slice` | 21.52s | >601s |

A 1.21s proof taking >605s is a 500× blow-up, not a tight timeout.

#### Root cause (for `copy_stack_buffer`, and the std-heavy jobs): opaque-pointer devirt blow-up

**The time is not in the solver — it is in `seapp`'s preprocessing.** For
`copy_stack_buffer` the dev14 BMC is trivial (`bmc.dag_sz 19`, solve 0.00s); the
~4s "baseline" is almost entirely the LLVM/seapp pipeline. Under dev18 the pipeline
never finishes. Sampling the running process shows it pinned in the **first `seapp`
`.pp` stage** (`--horn-inline-all --devirt-functions --devirt-functions-method=types`),
not in `seaopt` or `seahorn`.

**It reproduces on the dev14 bitcode too** (dev18 `seapp` on the LLVM-14 module also
hangs), so unlike Class 3 the bitcode is *not* the variable — it is dev18's LLVM.
The reason: LLVM 18 loads any bitcode as **opaque pointers** (typed pointers are
gone), so this is independent of how the module was compiled.

**Mechanism, bisected pass-by-pass on the 54 MB fat module:**

| seapp passes | time |
|---|---|
| `--horn-inline-all` alone | 1.6s (shrinks 54 MB → 637 KB) |
| `--devirt-functions=types` alone | 2.8s |
| `--horn-inline-all` + `--externalize-addr-taken` | 1.6s |
| **`--horn-inline-all` + `--devirt-functions=types`** | **hangs (>150s)** |
| `--horn-inline-all` on the devirt'd module | **hangs** |
| `--horn-inline-all` + `--devirt-functions=sea-dsa` | **4.5s** |

The explosive pair is **inline-all × type-based devirt**. Why it detonates under
opaque pointers: `--devirt-functions-method=types` resolves each indirect call to
*every function of matching signature*. With opaque pointers, signatures collapse —
in this module **299 functions share `i1 (ptr, ptr)`** and 284 share `void (ptr)`
(measured). Under LLVM 14 those had distinct pointee types and each indirect call
matched a handful. So devirt now turns each indirect call into a dispatch over ~299
targets, and `inline-all` then inlines all 299 recursively → combinatorial blow-up.

**Fix (in this repo): `src/sea.yaml` line `devirt-functions: 'types'` → `'sea-dsa'`**,
in all three configs (`sea.yaml`, `sea_cex_base.yaml`, `sea_pcond.yaml`). `sea-dsa`
"discards callees with inconsistent types" via points-to analysis, so it does not
over-match. The existing comment on that very line already says *"sea-dsa is better
but less stable"* — `types` was the LLVM-14 compromise; opaque pointers make it
untenable. Switching the pp stage alone drops 4.5s vs hang. NOTE: this changes the
devirt precision for **every** job, so re-run the whole suite and confirm no
verdict changes before committing — `sea-dsa` is more precise (a subset of `types`
matches), so it should only help, but it must be validated, not assumed.

**End-to-end confirmed on `copy_stack_buffer`:** full pipeline with
`--devirt-functions=sea-dsa` returns **`unsat` in 3.0s** (correct — matches the
dev14 baseline verdict) versus `types` hanging >600s. So it is not merely faster
preprocessing; the whole verification completes with the right answer.

**Class 2 has AT LEAST TWO distinct causes — the devirt fix does not cover all of
it.** Rechecking the std-heavy jobs with `sea-dsa` active:

| job | with sea-dsa | bottleneck stage | verdict |
|---|---|---|---|
| `copy_stack_buffer` | >600s → **3.1s** | was `seapp` devirt | fixed |
| `custom-print` | >612s → **31.6s** | was `seapp` devirt | fixed |
| `borsh` | still **TIMEOUT >250s** | not confirmed | **NOT fixed** |
| `custom-vec` | still slow — >140s in **`seahorn`** (BMC solver), `seapp` finished | — | **NOT fixed** — solver-bound |

So the devirt/opaque-pointer story is real and fixes the jobs whose bottleneck was
the `seapp` frontend (`copy_stack_buffer`, `custom-print`), but:

- **`custom-vec` is solver-bound**, not devirt-bound: its `seapp` completes and the
  time is in `seahorn`'s BMC. Different mechanism entirely — likely the LLVM 18
  codegen making the SMT query harder (candidate: same checked-arithmetic /
  pointer-representation changes seen in Class 3, forcing case splits). Unsolved.
- **`borsh` still times out** even with `sea-dsa`. Cause not yet localised — could
  be solver-bound like custom-vec, or the standing ahash 0.8.6 → 0.8.11 suspicion.
  Sample which stage it is pinned in before theorising.

Do not report Class 2 as solved. The honest status: the frontend devirt blow-up is
found and fixed (helps ≥2 jobs, and is correct hygiene for all); a **separate
solver-side blow-up** remains for at least `custom-vec` and `borsh` and is
unexplained. `sea-vec_test_macros` (347s with the provenance fix, little std) is
probably in this second bucket too.

#### The solver-side blow-up (`custom-vec_test_deref`): same-size formula, z3 can't solve it

Characterised `custom-vec_test_deref` (nondeterministic `original`/`num_pops`, loop
bound 10; push→grow→pop→deref-to-slice→index). It is **solver-bound**: sampling the
process shows it pinned in `seahorn` (BMC), not seapp/seaopt.

**It is the LLVM 18 bitcode, not SeaHorn — proven by the 2×2.** Both `.cut.o.bc`
modules (dev14-pipeline and dev18-pipeline, kept via `sea --save-temps`) were run
through the **same dev18 seahorn with the same full flags**:

| dev18-seahorn on... | dag_sz | solve |
|---|---|---|
| dev14-pipeline bitcode | 3551 | **unsat, 3.3s** |
| dev18-pipeline bitcode | 8941 | **timeout** |

Same solver, same flags, only the bitcode differs — so LLVM 18 codegen (rustc 1.81
+ dev18 seaopt) produces a **~2.5× bigger VC DAG** that z3 cannot solve in time.
Like Class 3, this is a **codegen issue, not a SeaHorn limitation** — which means it
is potentially fixable at the Rust/codegen level rather than being a solver wall.

**CORRECTION — two of my own metrics here were artifacts; do not trust them:**
- **`circ_sz` is NOT the signal.** An earlier note claimed a "72× circuit blow-up
  (6 → 434)". That 434 was measured with an **incomplete seahorn flag set** (missing
  `--horn-bmc-coi` and the other pipeline flags). With the *full* flags, `circ_sz`
  is **6 for both** dev14 and dev18. The circuit does not blow up. Retracted.
- **`i128` is NOT the cause.** dev18 shows 10 `i128` tokens vs dev14's 0, but they
  are all in **debug metadata and the datalayout string** — `0` real i128
  instructions. Retracted.

Both mistakes came from grepping/measuring without the full context (pipeline flags;
`grep i128` catching metadata). Lesson reinforced: verify a metric end-to-end before
building a story on it.

**What IS solid:** the VC DAG is ~2.5× bigger from the dev18 bitcode (3551 → 8941,
robust across vcgen flags — base dev18 dag is already 6745), and that extra size is
what z3 chokes on. VC-gen itself is instant; the cost is entirely z3 on the bigger
DAG. Reducing the loop bound 10→4 does not help (dag_sz unchanged — the loop is cut,
not unrolled, so iteration count is not the driver).

**Not yet identified: which instructions inflate the DAG 2.5×.** Op-counts on the
two `.cut.o.bc` modules are inconclusive — dev18 has *fewer* IR lines (4373 vs 5922),
fewer stores (21 vs 45), fewer overflow intrinsics (2 vs 18), but *more* ptrtoint
(19 vs 8). Fewer instructions yet a bigger VC means the expansion is per-instruction
in vcgen, not a raw instruction-count difference.

**Ruled out: shadow-memory disambiguation is NOT coarser on dev18.** Hypothesis was
that LLVM 18 opaque pointers make sea-dsa less precise → fewer distinct memory
regions → everything funnels through shared arrays → bigger VC. Dumped the
shadow-mem-instrumented IR (`seahorn ... --oll=out.ll`, no `--horn-solve`) for both
and compared:

| | shadow.mem.def points | load/store | distinct cells touched |
|---|---|---|---|
| dev14 cut | 8 | 6 / 38 | 2 ({0,24}) |
| dev18 cut | 8 | 7 / 35 | 3 ({0,1,25}) |

Identical number of shadow-memory definition points (8), comparable load/store
counts, and dev18 touches *more* distinct cells, not fewer. No sign of extra node
collapse. So sea-dsa is not disambiguating worse on dev18, and shadow memory is not
the source of the 2.5× VC growth. (The `--oll` dump does contain the expected
`shadow.mem.{load,store,init,arg.init,global.init}` calls, each tagged with
`!shadow.mem`/`!shadow.mem.def` metadata — useful for any future memory-model
analysis.)

**Leading concrete lead: alignment ASSUMPTIONS on symbolic bases (not comparisons).**
The dev18 `.cut.o.bc` has 19 `ptrtoint` (vs dev14's 8), all in `@main` (test_deref
inlined). Classifying what they feed: dev18 has **12 feeding `and i64 %x, 3`**
(dev14: 1). Each is the alignment-check idiom, and it lands as an *assumption*:

```llvm
%218 = and i64 %217, 3                ; (ptrtoint malloc'd_ptr) & 3
%219 = icmp eq i64 %218, 0            ; 4-byte (i32) aligned?
call void @verifier.assume(i1 %219)  ; ASSUME aligned
%220 = call i1 @sea.is_dereferenceable(ptr %216, i64 4)
```

**Why this makes it slow but NOT wrong** (this is the key point, and it answers the
natural objection "wouldn't ptrtoint give `sat`, not a timeout?"):
- These `ptrtoint` feed **assumptions**, not pointer *comparisons*. The Class 3
  `sat` came from imprecise pointer *equality* (componentwise `ptrEq`). Imprecision
  in a comparison can flip `unsat`→`sat`. An **assumption only restricts the search
  space** — it can never turn `unsat` into `sat`. So this cannot cause a wrong
  verdict; it can only slow the solve. Hence timeout, not `sat`.
- It is nonetheless **hard**: `assume((base+offset) & 3 == 0)` over a *symbolic*
  allocation base forces the bitvector solver to track the low bits of ~12 distinct
  symbolic addresses and propagate them through all the address arithmetic and
  dereferenceability checks. Low-bit / modular constraints on symbolic bitvectors
  are a classic bitvector-solving blow-up — precise but expensive.

**ABLATION RESULT: REFUTED.** Neutralised all 12 alignment assumes in the dev18
`.cut.o.bc` (rewrote `assume(i1 %aligncheck)` → `assume(i1 true)`, tracing
`and,3`→`icmp eq 0`→`assume`) and re-ran:

| module | dag_sz | solve |
|---|---|---|
| dev14 | 3551 | 3.3s |
| dev18 original | 8941 | hang |
| dev18, 12 align-assumes removed | **8902** | **still hangs (>120s)** |

Removing them changed dag_sz by **−39** (8941→8902) and did **not** speed up the
solve. So the alignment assumptions are **not** the cause — this hypothesis is dead.
The mechanism-story above (precise-but-hard alignment reasoning) was plausible and
correctly explained why it is `timeout`-not-`sat`, but the ablation shows it is not
what actually drives the cost. The ~5400 extra DAG nodes come from elsewhere.

**Tally of dead hypotheses for the custom-vec solver blow-up (all tested & refuted):**
`circ_sz` (measurement artifact), `i128` (debug metadata), shadow-mem coarsening
(disambiguation identical), alignment assumes (ablation, above). The one robust,
repeatedly-confirmed fact is: **dev18 bitcode → 8941 dag_sz → z3 hangs; dev14 bitcode
→ 3551 → 3.3s, same seahorn.** WHAT produces the 2.5× DAG is still unknown.

**Change of method (stop guessing):** rather than nominate another construct, the
disciplined next step is to *bisect the DAG's origin* — the extra ~5400 nodes trace
to specific instructions/basic-blocks in the dev18 module. Diff the two `.cut.o.bc`
reachable from `@main` structurally (block-by-block, or instrument vcgen to attribute
DAG nodes to source instructions) and let the data name the construct. Four guesses
have now failed; the module contains the answer and should be read, not predicted.

#### FOUND (via VC dump, not guessing): byte-granular memory access from unprovable alignment

SeaHorn can dump the VC as SMT-LIB — no tool needed: `seahorn ... --log=bmc.simplify`
prints `z3_to_smtlib(vc)` at encode time (before solve). Dumped both modules and
diffed the SMT operator distribution (dev14 → dev18):

| smt term | dev14 | dev18 | ratio |
|---|---|---|---|
| `(_ extract 63 3)` | 1006 | 3715 | 3.7× |
| `ite` | 1408 | 4010 | 2.8× |
| `concat` | 978 | 2256 | 2.3× |
| `bvadd` | 2188 | 5358 | 2.4× |

`(_ extract 63 3)` = take bits [63:3] of an address = **byte-address → 8-byte-word
index** for the lambda memory array. The discriminating detail is what it wraps:

```
dev18:  ((_ extract 63 3) (bvadd #x…1 a!34))   ; addr+1
        ((_ extract 63 3) (bvadd #x…2 a!34))   ; addr+2   } one i32 read as
        ((_ extract 63 3) (bvadd #x…3 a!34))   ; addr+3   } 4 separate BYTES
```

Count of extracts over a `+1/+2/+3`-byte address: **dev14 = 0, dev18 = 414.** So
dev18 reads/writes each `i32` element (custom-vec is `CustomVec<i32>`) **byte by
byte**, then reassembles with `concat`/`ite`; dev14 does a single **word-aligned**
access. Byte-splitting every element multiplies extract/concat/ite → the 2.5× VC.

**Mechanism:** the lambda memory model accesses byte-granularly when it cannot prove
the address is 4-aligned. dev18's addresses are complex symbolic `base+offset`
expressions (the LLVM 18 pointer arithmetic, more `ptrtoint`), whose alignment the
model can't establish statically; dev14's were provably aligned. Byte access is
**precise** (correct — stays `unsat`, matching the observed timeout-not-`sat`) but
far larger and harder for z3.

This also explains why the **alignment-assume ablation failed**: those `assume(ptr &
3 == 0)` constraints don't change the memory model's *static* byte-vs-word decision —
the model inspects the address *expression*, not the assumptions. Adding an assume
doesn't make the address provably-aligned to the access-lowering logic.

**Verdict is `unsat`, confirmed — it is slow, not wrong.** Given enough wall-clock,
dev18 `test_deref` returns **`unsat` in 169s** (original) / 346s (alignment-assumes
ablated). So the ctest "timeout" was the 400s cap plus CPU contention under the
30-way parallel suite, not true non-termination. This is complexity (bigger VC),
never imprecision — exactly consistent with the byte-granular-access mechanism and
with "assumptions/precise-encoding can't flip `unsat`→`sat`".

**CONFIRMED by the alignment flag.** SeaHorn's lambda memory model (8-byte word)
byte-splits an access when it cannot *statically* prove the address is word-aligned;
`--horn-opsem-ignore-alignment` forces word-alignment. On the dev18 module:

| dev18 run | dag_sz | byte-split extracts | solve |
|---|---|---|---|
| default (respect alignment) | 8941 | 414 | hang / 169s |
| `--horn-opsem-ignore-alignment` | **754** | **0** | **unsat, 1.1s** |

12× smaller VC, byte-splits gone, instant solve. So byte-granular access from
alignment handling is definitively the cause — not a correlation.

**Why dev18 and not dev14 (the address expressions, not the instruction `align`):**
- Not the IR `align` attribute — dev14 and dev18 have similar distributions (both
  ~65 `align 1` ops; i32 accesses `align 4` in both). The split is decided at the
  SMT level from the *address expression's* provable alignment, not the annotation.
- The byte-split extracts are all over *symbolic* address bases (0 over concrete
  `#x…`). dev14's bases: `sea.sp0_0` (stack) only, 842 total. dev18's: `sea.sp0_0`
  (2796, ~3.4×) **plus a new `a!34`** cluster (555).
- `a!34` is `(bvadd (ite (= a!23 a!25) a!29 a!33) …)` — a **conditional pointer**
  (an `ite`/phi selecting between addresses). A control-flow-dependent address like
  this can be proven neither aligned nor constant, so every byte access over it
  stays explicit and multiplies `extract`/`concat`/`ite`.
- Root: LLVM 18's branchier codegen (conditional/`select`-heavy pointer computation,
  the same split-comparison tendency seen in Class 3) yields these `ite`-chain
  pointer expressions where dev14 kept the pointer simpler/foldable. The memory
  model then can't prove them word-aligned → byte-splits → precise-but-huge VC.

Everything stays sound (`unsat`) throughout — this is size/complexity, never the
imprecision that would give `sat`.

**The dominant byte-split source is the STACK, not the heap conditional pointer.**
Of the 414 byte-split extracts, 555-ish are the heap `a!34` cluster but **2796 are
over `sea.sp0_0`** — the symbolic stack-pointer base — vs dev14's 816. With
`--horn-explicit-sp0=false` (the config default) sp0 is symbolic, so *stack*
addresses `sp0+offset` can't be proven word-aligned and byte-split. This corrects
the earlier emphasis: the `grow()` conditional pointer (`a!34`) is real but
secondary; the bulk is symbolic-stack access, and dev18 has ~3.4× more of it.

**Two levers, both confirmed on custom-vec `test_deref` (hang → fast):**

| flag (added to pipeline) | dag_sz | byte-splits | solve |
|---|---|---|---|
| default | 8941 | 414 | hang / 169s |
| `--horn-opsem-ignore-alignment` | 754 | 0 | unsat 1.1s |
| `--horn-explicit-sp0=true` | 1192 | 0 | unsat 2.5s |

`--horn-explicit-sp0=true` is the **cleaner** lever: it makes the stack base a
concrete (aligned) address instead of blanket-assuming every access is aligned. That
targets the actual cause (symbolic stack) rather than over-approximating all memory,
so it is less likely to mask a real misalignment bug — though it should still be
sanity-checked for soundness on these jobs (concrete sp0 fixes one stack layout;
fine for proofs that don't depend on absolute stack addresses, which these don't).

**Swept `--horn-explicit-sp0=true` across the 9 solver-bound timeouts. Result:**

| job | with explicit-sp0=true | was |
|---|---|---|
| `sea-vec_test_macros` | **unsat 1s** | >605s |
| `custom-vec_test_insert` | **unsat 1s** | timeout |
| `custom-vec_test_deref` | **unsat 2.5s** | hang/169s |
| `tinyvec-arrayvec-aop_test_retain` | still slow (>250s) | ~309s — NOT fixed |
| borsh ×5 | not solver-bound — see below | timeout |

So `explicit-sp0=true` rescues the **sea-vec / custom-vec cluster** (3 jobs, dramatic:
605s→1s) — symbolic-stack byte-splitting was their shared cause. It does **not** fix:
- **`tinyvec_test_retain`**: still ~250s+ in the solver → a *different* solver cost,
  not stack byte-splitting. Unexplained.
- **the five borsh jobs**: these are **frontend-bound**, not solver-bound. A *solver*
  flag cannot help them. Diagnosed below.

#### borsh: NOT equally bad in dev14 — it's the devirt again (even sea-dsa)

**dev14 borsh-enum ran in 5.24s; dev18 times out (>400s) — an ~80× regression, not
"always slow".** The cause is the same opaque-pointer devirt problem as
copy_stack_buffer, except sea-dsa (the fix that tamed copy_stack_buffer) is *not
enough* for borsh.

The tell is the `inline-all` (pp) stage output — inline-all inlines everything into a
handful of functions, then whether DCE/devirt keeps the module small decides
everything downstream:

| | after pp | verdict/time |
|---|---|---|
| dev14 (devirt=types, LLVM 14 typed ptrs) | 5.6 MB / **61k lines**, 2 fns | unsat, 4s |
| dev18 (devirt=sea-dsa) | 114 MB / **1.29M lines**, 7 fns | timeout |
| dev18 inline-all **alone** (no devirt) | **50k lines**, 23 fns | — |
| dev18 inline-all + sea-dsa devirt | **1.34M lines**, 61 fns | — |

So inline-all *alone* on dev18 borsh is 50k lines (as small as dev14) — **the 27×
explosion is entirely the devirt.** borsh has ~207 indirect calls; under LLVM 14's
typed pointers, `devirt=types` resolved each to a few precise candidates (small);
under LLVM 18 opaque pointers, even `sea-dsa` over-approximates the candidate sets,
and inline-all then expands every candidate → 1.34M lines → O3/unroll crawl.

This means: **`devirt: sea-dsa` fixed the *small* devirt-bound jobs (copy_stack_buffer,
custom-print) but not borsh** — borsh has enough indirect calls that even sea-dsa's
residual imprecision explodes. Same root (opaque-pointer devirt), different severity.

**`devirt: none` makes borsh-enum 4s / unsat / assertion-checked — but it is
genuinely UNSOUND, not merely "unverified".** Confirmed from source how SeaHorn's
bv2 opsem handles an unresolved indirect call (`BvOpSem2.cc::visitIndirectCall`):

```cpp
if (CB.getType()->isVoidTy()) { /* "Interpreting indirect call as noop" */ return; }
setValue(CB, Expr());   // non-void: havoc the RETURN value only
```

It havocs the **return value** (that *is* a sound over-approximation) but does
**nothing to memory** — no store, no shadow update (neither `visitIndirectCall` nor
`visitExternalCall` contains any memory op; `horn-bv2-enable-external-calls` is off
by default). So it assumes the callee **writes nothing to memory**. That is an
*under*-approximation of the callee's side effects, not an over-approximation: it
drops executions where the callee mutates memory. If an assertion violation needs
that write, the model can't see it → **false `unsat`.** This is exactly why the
devirt help says "needed for soundness" — only resolving the call to the real
function gets its memory effects modeled.

The common intuition "havoc over-approximates, so `unsat` is sound" holds *only* for
a havoc that over-approximates ALL effects incl. memory; SeaHorn's does not. For
borsh it bites directly: serialization's indirect calls (`Write::write` into the
buffer, per-field serialize methods) exist to mutate memory, so with devirt off a
round-trip serialize→deserialize→assert-equal proof can pass vacuously. dev14 avoided
the whole dilemma: typed-pointer `devirt=types` was both precise *and* small, so the
real callees were resolved and their memory effects modeled.

Real fix directions (none adopted): a devirt precise enough under opaque pointers
(SeaHorn-side improvement); or reduce borsh's indirect calls / stub the irrelevant
std; or accept `devirt: none` after a per-job soundness check; or just raise borsh's
timeout (the pipeline is slow-but-finite — the original `.ul.bc` did complete).

This is the **third distinct Class 2 cause**: (1) devirt over-match, small jobs
[fixed: sea-dsa], (2) symbolic-stack byte-split [fixed: explicit-sp0], (3) devirt
over-match, borsh — sea-dsa insufficient [open]. Plus tinyvec_retain's separate
solver cost [open]. Note (1) and (3) are the *same* mechanism at different scales.

#### Feasibility measurement: type-recovery (à la promote-memcpy) will NOT fix borsh

Question: can we recover pointer types under opaque pointers (the `PromoteMemcpy.cc`
`recoverStructType` trick — read a GEP's `getSourceElementType` on the same pointer)
to make devirt precise and stop the borsh explosion? Measured on borsh-enum's fat
module (read-only IR analysis + existing `--log=devirt`, no SeaHorn rebuild). **Answer:
no** — the arg-type-recovery lever cannot help borsh. Evidence:

- borsh has **207 indirect calls; ~185 (90%) are trait-object vtable dispatch** — the
  fn-pointer is `load ptr, … !invariant.load` from a vtable slot (Rust `dyn Trait`).
- The candidate sets are genuinely polymorphic method families. E.g. one `core::fmt`
  callsite `%119 = call i1 %111(ptr,ptr)` resolves to **26 candidates — every
  `Debug`/`Display::fmt` impl in the program** (u8, u32, String, &T, *mut T, …).
- Those 26 candidates **all share the exact signature `i1(ptr,ptr)`**, and both args
  (`self`, `Formatter`) are **type-erased `ptr`** at the callsite (`self` is
  `load ptr, ptr …`). So: no arg pointee type is recoverable at the callsite, and
  even if it were, it is identical across all candidates. A pointee-type-consistency
  filter operates *within* a signature bucket — but here the whole candidate set is
  one signature bucket. It has nothing to discriminate on.
- Why promote-memcpy's trick doesn't transfer: memcpy has a GEP on the *same* pointer
  in the *same* function revealing the struct type — a local, syntactic recovery.
  Trait-object dispatch deliberately erases `self`'s concrete type at the callsite;
  recovering it would require an inter-procedural trace back through fat-pointer
  construction, far beyond the local GEP-scan trick.

#### THE FIX for borsh: abstract Rust panic entries (validated end-to-end)

The 26-candidate fmt/trait explosion is almost entirely the **std panic + message-
formatting machinery** (borsh uses `std`, so panics go through `rust_begin_unwind` →
`std::panicking::*` → format the message via `dyn Debug`/`Display` and box the payload
via `dyn BoxMeUp`). Abstracting the panic entry points to `assume(false)` (panic-as-
abort) — or `__VERIFIER_error()` (panic-as-error) — makes that whole subgraph dead, and
GlobalDCE removes it *before* devirt can explode it.

Measured on borsh-enum's fat module (stub 9 panic/unwind roots — `rust_begin_unwind`,
`std::panicking::{default_hook,rust_panic_with_hook,set_hook,begin_panic_handler
closures}`, `core::panicking::const_panic_fmt` — with `call void @__VERIFIER_assume(i32
0); unreachable`, then run the normal pipeline):

| | un-stubbed | panic-abstracted |
|---|---|---|
| defined functions | 2583 | **96** (after GlobalDCE) |
| `core::fmt` functions | 203 | 11 |
| indirect calls | 207 | 27 |
| vtable dispatch (`!invariant.load`) | 315 | 27 |
| pp module | 1.34M lines (→ O3/unroll timeout) | **31k lines, 0s** |
| **end-to-end** | **timeout** | **`unsat`, ~1s** (frontend 1s + solve 0s) |

**Soundness:** `assume(false)` at a panic prunes panicking executions — sound for
proving a property that holds on non-panicking paths (a panic aborts before any
downstream assert, so pruning masks nothing). `__VERIFIER_error()` instead reports
reaching a panic as a bug (checks panic-freedom). The mode should be a flag
(abort-vs-error), exactly as proposed. The `verifier::vassert!` property assertions
are untouched (50 assert/error markers survive; entrypt intact) → correct `unsat`.

**Gotcha:** stub with `assume(false)`, NOT a bare `unreachable` — SeaHorn's BMC
core-dumps on a reached `unreachable`; `__VERIFIER_assume(i32 0)` is clean.

#### IMPLEMENTED as a verify-rust job config (2026-07-20)

The panic abstraction is now a build-time job option, not a SeaHorn pass:
- `src/sea-lib/panic_abstract.c` defines `rust_begin_unwind(ptr)` → `__VERIFIER_assume(0)`
  (default) or `__VERIFIER_error()` (`-DABSTRACT_PANIC_ERROR`); built to bitcode by
  `src/sea-lib/CMakeLists.txt`.
- `c_rust_llvm` (`cmake/SeaHornBuild.cmake`) gains opt-in tokens `ABSTRACT_PANIC` /
  `ABSTRACT_PANIC_ERROR`. When set, after the `--dump-section` extraction it runs
  `llvm-link-18 <bc> --override=<stub>.bc -o <bc>`, replacing `rust_begin_unwind`;
  SeaHorn's GlobalDCE then strips the dead panic/fmt runtime. No token → byte-identical
  to before (no_std jobs keep their own `#[panic_handler]` — verified the token is
  opt-in precisely because `sea-vec` etc. define their own `rust_begin_unwind`).
- Added `ABSTRACT_PANIC` to the 5 borsh jobs.

**Result (validated): 4 of 5 borsh timeouts fixed.** Via ctest:
`borsh_test_fixed_sized_arrays`, `borsh_test_string`, `borsh-enum`, `borsh-option`,
`borsh-structs` all **pass `unsat` in <1.1s** (were >400s timeouts). The stripped modules
go fmt 203→8-11, panic-hook 77→0.

**`borsh-primitives` now returns `sat`** (was timeout) — NOT resolved, flagged:
- It round-trips every primitive (`bool`..`u128`/`i128`) via
  `try_to_vec().unwrap()`/`try_from_slice().unwrap()` and asserts `x == decoded`.
- Could not cleanly isolate whether the `sat` is (a) a pre-existing dev18 codegen
  precision issue exposed now that it is fast enough to reach a verdict (plausible —
  dev18 has known wide-int/pointer precision issues, cf. Class 3 and custom-vec; and
  `u128`/`i128` round-trips are prime suspects), or (b) an abstraction artifact
  (override+DCE removing an initializer → undef read → spurious mismatch). Every
  isolation path hit a tooling limit (dev14 bc times out with full panic; cross-version
  `llvm-link --override` breaks on DI metadata).
- **Soundness note:** `assume(false)` pruning can only *remove* behaviors, so it cannot
  turn `unsat`→`sat`; a `sat` is either real or a false positive (safe direction — never
  hides a bug). So the abstraction stays sound; borsh-primitives is a precision/false-
  positive regression at worst. Net for borsh: 4 newly passing, 1 still failing (now
  fast-sat, not timeout) — a strict improvement.

**ROOT-CAUSED (2026-07-20): borsh-primitives `sat` is a wide-integer imprecision, NOT
the panic abstraction.** Minimized `entrypt` to a single round-trip and bisected by type:

| single round-trip | verdict |
|---|---|
| `u32` (with ABSTRACT_PANIC) | **`unsat`** in 1s (correct) |
| `u64` | **`sat`** |
| `u128`, `i128` | timeout |

So the abstraction is fine (u32 round-trips correctly under it); the `sat` appears at the
**≥64-bit boundary** (u32 unsat, u64 sat, u128/i128 timeout).

**FULLY ROOT-CAUSED: it is the ExtraWide memory-manager bug — the SAME one as sea-vec
test_zst ptrEq.** From the u64 counterexample (`--cex` on a minimized single-u64
round-trip):

```
decoded = %_72 = ptrtoint ptr %_70          [borsh reconstructs the value via a pointer]
%_70    = struct(base = main@%_4_0 (=x), offset = 0xffffffffffffffff (= -1), ...)
%_72    = bvadd(0xffffffffffffffff, x)  =  x - 1      -> decoded = x-1 != x -> sat
```

The decoded u64 is reconstructed by `ptrtoint` of a fat pointer whose ExtraWide
`(base, offset)` is `(x, -1)`; ptrtoint yields `base + offset = x - 1`. The `-1` offset
comes from a `getelementptr i8, ptr, i64 -1` (a one-past-end rewind in the borsh/std slice
decode under LLVM 18 — the same shape as sea-vec `next_back`'s `end - 1`). So the value
`x` is (wrongly) flowing through the fat-pointer representation and picking up the slice's
-1 offset.

Confirmed ExtraWide-specific by the same differential that pinned the ptrEq bug:

| memory manager | single u64 | full borsh-primitives |
|---|---|---|
| `--horn-bv2-extra-widemem` (our config) | **sat** | sat |
| `--horn-bv2-widemem` | **unsat** | **unsat** |

Since borsh `u64` encode/decode is a bijection, the `sat` **cannot be real** — it is
SeaHorn imprecision (spurious, safe direction). It is NOT caused by the panic abstraction
(u32 round-trips unsat under it); the abstraction merely made the job fast enough to reach
the verdict.

**Same manager family as sea-vec test_zst, but a DISTINCT proximate sub-bug.** Both are
`BvOpSem2ExtraWideMemMgr` leaking the `(base,offset)` split under LLVM 18's `gep i8 -1`,
but via different functions:
- sea-vec test_zst → **`ptrEq`** (componentwise `base==base && offset==offset`).
  **FIXED** in seahorn `d7277faa` (compare `getAddressable`, i.e. `base+offset`, mirroring
  `ptrNe`). Both regression lit tests pass; no regression across 123 pointer-heavy
  verify-rust tests. This also makes the sea-vec Rust `wrapping_byte_*` provenance
  workaround redundant at the SeaHorn level (kept anyway — exposed-provenance is a real
  Rust defect worth fixing regardless).
- borsh-primitives → **`ptrtoint`** of a fat pointer whose `(base,offset)` is `(x,-1)`.
  `ptrtoint` already uses `getAddressable` (so it correctly yields `base+offset = x-1`);
  the bug is UPSTREAM. The ptrEq fix does NOT address this (verified: still `sat`).
  **Traced precisely** (cex on a minimized single-u64 round-trip):
  - `x` is the nondet input (`%_1072 = nondet.sea_isize`).
  - u64 decode goes through `core::slice::copy_from_slice` (borsh `from_le_bytes` copies
    the 8 bytes, then reinterprets). The decoded value is produced by
    `%_1101 = load ptr, ptr %_1090` — an **opaque `load ptr`** of a cell holding x's
    bytes — which the ExtraWide model reconstructs as `struct(base=x, offset=-1)`.
    `%_1148 = ptrtoint %_1146(=%_1101) = bvadd(-1, x) = x-1`, then `icmp eq x, x-1` = false.
  - So the bug is in ExtraWide's reconstruction of a **pointer-typed load of a cell that
    was written as integer data**: it fabricates a spurious `-1` offset. A correct model
    would treat it as `inttoptr(data) = (base=data, offset=0)`.
  - **FIXED** in seahorn `8d3980f6`. Root cause pinned to one line:
    `ExtraWideMemManagerCore::storeIntToMem` wrote raw memory but returned the offset/size
    shadow memories *unchanged*, so a plain integer store left a **stale offset** from a
    prior pointer store to the same location; a later `load ptr` reconstructed
    `(base=data, offset=stale)` → `ptrtoint = data+stale`. Fix: zero the offset shadow on
    the store (storing an int means the cell holds data → reload is `inttoptr(data)=(data,0)`).
    Restricted to **pointer-sized** stores to avoid bloating memory-heavy VCs (the wider
    version timed out 2 near-cap smallvec/tinyvec tests; the narrow guard does not).
    Validated: borsh-primitives sat→unsat; full suite **198 pass / 14 fail / 1 timeout**
    (was 197/15/1), no new failures/timeouts.
  - A minimal standalone SeaHorn lit test could not be reliably constructed — LLVM folds
    synthetic store-ptr/store-int/load-ptr reproductions (DSE removes the dead pointer
    store), and the bug needs borsh's real aliasing. borsh-primitives is the regression.

Workaround for borsh-primitives: `--horn-bv2-widemem` makes the full test pass `unsat`
(global precision change, not flipped in-config). Spurious `sat` throughout (safe
direction); not caused by the panic abstraction. The panic-abstraction work is correct
and complete.

### CMake incremental test-discovery bug — FIXED

Symptom: after rebuilding a single job's `_llvm.bc`, `ctest` dies with "Maximum recursion
depth of 1000 exceeded" in `<job>/CTestTestfile.cmake:7 (include)`.

Root cause: `sea_discover_tests`/`sea_discover_unsat_tests` (and the extract scripts) wrote
discovered tests **into `CTestTestfile.cmake` — the filename CMake reserves and
auto-generates per build subdir**. Because they also set that same path as the directory's
`TEST_INCLUDE_FILES`, CMake's own generated `CTestTestfile.cmake` emitted
`include("<this dir>/CTestTestfile.cmake")` — a self-include → infinite recursion. The
discovery custom-command overwrote it with real `add_test`s, so it *usually* worked; but any
reconfigure/partial rebuild that regenerated the CMake stub without re-running discovery left
the self-including version.

Fix (standard `gtest_discover_tests` pattern): write discovered tests to a **dedicated file
`<TARGET>_ctest_tests.cmake`** and point `TEST_INCLUDE_FILES` at THAT. CMake's
`CTestTestfile.cmake` then does `include("<TARGET>_ctest_tests.cmake")` — a different file,
no recursion, and it survives reconfigures. Also made the extract scripts `file(WRITE ... "")`
the file unconditionally (truncate-first) so reruns don't accumulate and the file always
exists for `include()`. Changed: `cmake/SeaHornBuild.cmake` (both discover functions),
`cmake/extract_tests.cmake.in`, `cmake/extract_unsat_tests.cmake.in`. Regression-tested:
rebuilding a single job's `.bc` no longer corrupts the test file; ctest enumerates and runs.

**Full-suite validation (2026-07-20, ctest -j30 --timeout 400, all fixes applied):
197 passed, 15 failed, 1 timeout** — vs the pre-panic-abstraction baseline of
193/14/6. Net change is exactly as designed:
- **4 borsh timeouts → pass** (borsh-enum, borsh-option, borsh-structs, borsh_test_string;
  borsh_test_fixed_sized_arrays was already passing). Timeouts 6→1.
- **borsh-primitives: timeout → fail** (fast `sat` in 3.6s) — the +1 failure (14→15),
  the known unresolved case above.
- **1 remaining timeout**: `tinyvec-arrayvec-aop_test_retain` (the separate solver-cost
  issue, unrelated to panic — not targeted).
- **No other change.** The other 14 failures are the exact pre-existing set (3 blacklist +
  11 Class 1 "no assertion found"); the opt-in design left every non-borsh job untouched.
So: +4 pass, −5 timeout, +1 fail — a strict improvement, with borsh-primitives the only
loose end.

**This supersedes the vtable-resolver plan for borsh.** The general vtable resolver
(`sunny-marinating-eclipse.md`) remains a valid *general* capability, but borsh's
explosion is panic-machinery bloat, and abstracting panic is a far cheaper, sound,
general fix (any std Rust job whose bloat is panic/format machinery benefits).

**Implementation:** a small `AbstractPanic` seapp pass (replace panic-entry bodies with
`__VERIFIER_assume(i32 0)` [abort] or `__VERIFIER_error()` [error], gated by a mode
flag), run FIRST in the pp pipeline (before devirt/inline) so GlobalDCE cascades. Or a
verify-rust-side llvm-link override providing the stubs. TODO: validate it fixes the
other four borsh timeouts (same std machinery → expected) and re-run the full suite.

**CORRECTION (a build-config fix does NOT work — and a methodology warning).** I first
thought the fix was a build-config one: verify-rust's default `-Zbuild-std-features=
panic_immediate_abort` should strip panic message formatting, and borsh looked like it
was linking the wrong std due to shared-target-dir aliasing. **That was wrong**, and it
came from trusting `nm` on the crate `.a` — **`nm` does not read LLVM-bitcode internal
symbols** (the crate libs are linker-plugin-lto bitcode), so it reported 0 fmt/panic
where `llvm-nm` shows 554 fmt / 5 panic-hook defs. USE `llvm-nm`, NOT `nm`, ON THESE
`.a` FILES. Corrected findings:
- An *isolated* clean `panic_immediate_abort` build of borsh-enum still has 554 fmt /
  5 panic-hook defs — identical to the shared build. No aliasing; immediate_abort is
  applied and simply does not strip borsh's machinery.
- Why: borsh references `rust_panic_with_hook`, `set_hook`, `__rust_start_panic` and
  has 14 `catch_unwind`/`_Unwind_` refs (custom-vec: zero). Something in borsh's deps
  (`std::io`, the `borsh` crate, or `ahash`) uses `catch_unwind`/the panic-hook API,
  which forces the full unwinding+hook runtime to link. `panic_immediate_abort` only
  changes the `panic!` macro; it cannot remove an explicitly-used unwinding runtime.
So the panic-entry **stub** (assume-false/error) is the fix that works regardless of
why the runtime is linked — as a verify-rust build step (llvm-link `--override` of a
verifier-lib module defining the panic entries) it stays a job config, not a SeaHorn
pass, per the requirement.

#### (Superseded) arg-type filter feasibility — kept for the record

**The go/no-go gate came back NO-GO for the arg-type filter (Lever A).** The only
real discriminator for these calls is *which vtable* the fn-pointer was loaded from —
a sea-dsa points-to precision problem, not arg-type recovery. And even perfect vtable
tracking may not shrink `core::fmt` dispatch much, because it is genuinely polymorphic
(a `Formatter` really can format any type). ABI attributes carry only weak size hints
here (14/180 calls have `sret`, 43 have `dereferenceable(N)`; 83 have only `align 1`
erased pointers).

**What would actually help borsh** (not arg-type recovery):
- **Cap devirt+inline fan-out**: if a callsite has > N candidates, do NOT devirt-and-
  inline it (leave the indirect call, or devirt without forcing inline). The explosion
  is `inline-all × many-candidates`; borsh's fmt/trait dispatch is legitimately
  many-candidate, so inlining all 26×207 is the wrong move regardless of types.
- **Vtable-provenance precision in sea-dsa** (keep distinct vtable loads in distinct
  nodes) — helps the non-`fmt` dispatches, but hard and partial.
- **Just raise borsh's timeout** — the pipeline is slow-but-finite.

Plan file (feasibility): `~/.claude/plans/sunny-marinating-eclipse.md`. The two levers
designed there (A: devirt type filter; B: sea-dsa store-type recovery) remain valid
*in general* and would help direct-fn-pointer indirect calls with typed args — just
not borsh's trait-object-dominated workload.

**APPLIED and VALIDATED.** `horn-explicit-sp0: true` set in `src/sea.yaml` (+ cex/pcond).
Full suite (ctest -j30 --timeout 400, all 213):

| | passed | failed | timeout |
|---|---|---|---|
| before (sp0=false) | 190 | 14 | 9 |
| after (sp0=true) | **193** | 14 | **6** |

+3 passes (custom-vec_test_deref, custom-vec_test_insert, sea-vec_test_macros), **no
regressions** (failed set unchanged: same 3 blacklist + 11 Class 1 "no assertion").
6 remaining timeouts = 5 borsh (frontend-bound) + tinyvec-arrayvec-aop_test_retain.
So `explicit-sp0: true` is a keeper.

**Build-tree gotcha hit during this validation (worth knowing):** rebuilding
individual `<job>_llvm.bc` targets for the multi-entry jobs (those using
`sea_discover_tests`) left their `build-*/…/CTestTestfile.cmake` **self-including**
(`include(<self>)` → "Maximum recursion depth 1000") because CMake regenerated the
file with a `TEST_INCLUDE_FILES` self-reference and the discovery post-build step
didn't re-run to overwrite it with the actual `add_test`s. 7 jobs were affected
(borsh, sea-vec, custom-vec, smallvec, smallvec-allocation, tinyvec-arrayvec,
tinyvec-arrayvec-aop). Fix: a full `cmake --build .` (runs the discovery steps and
rewrites the files). If ctest dies with a recursion error after you've rebuilt a
single job target, this is why.

#### Why the dev18 toolchain and not dev14 — as far as it can be answered

The conditional pointer that becomes `a!34` is **inherent to the source**, identical
in both builds (`custom-vec` is a hand-written Nomicon Vec, not std): `RawVec` starts
`ptr: NonNull::dangling()` and `grow()` does `new_ptr = if cap==0 { alloc } else {
realloc(old_ptr) }`. So it is **not** a std-library or source difference — both
toolchains compile the same conditional.

**"Rust version vs LLVM version" is not cleanly separable in these two builds:** rustc
1.64's codegen backend *is* LLVM 14; rustc 1.81's *is* LLVM 18. The dev14→dev18 jump
moves both together; you cannot hold one fixed. So the dichotomy in the question is
mostly a false one for this comparison.

What differs is how the identical conditional is *lowered*. Observable IR signature:
dev18 favors `select` (5 vs 1) with fewer pointer `phi` (4 vs 12) — i.e. more
if-conversion (branch+phi → `select`). A `select`ed pointer is a conditional *value*
→ an `ite`-address in the VC (which SeaHorn can't prove aligned → byte-split); a
`phi` pointer is resolved per path in BMC and stays simple. That matches LLVM's known
trend toward more aggressive if-conversion in newer versions — but this is attribution
**by signature, not proof** (small counts; not bisected).

**The experiment that would actually separate rustc-frontend from LLVM-version**
(NOT done — needs extra toolchains): bisect intermediate rustc nightlies across the
LLVM bumps (~1.65→LLVM15, ~1.70→LLVM16, ~1.73→LLVM17, ~1.78→LLVM18) and find which
toolchain first shows the byte-split blowup. If it flips exactly at an LLVM bump →
LLVM version; if at a non-bump rustc release → a rustc MIR/frontend change. Until then
the honest statement is: "the LLVM-18-era toolchain lowers the same conditional-pointer
source into addresses SeaHorn's static alignment analysis can't see through."

Reusable recipe (this is sound, unlike the metrics above): `sea --save-temps
--temp-dir=D` produces `D/*.cut.o.bc` (the exact seahorn VC input); run `seahorn`
on it **minus `--horn-solve`** but **with the full pipeline flags** to get dag_sz in
<1s, and swap that file between dev14/dev18 to isolate bitcode-vs-solver. Using the
full flag set is essential — a partial set gives misleading sizes (see the circ_sz
retraction).

### Class 3 — different verification result (1 test)

`sea-vec_test_zst_unsat_test` returns **`sat`** where the baseline proved `unsat`.
It is not a timeout and not a vacuity artifact. Both sides reproduce in about a
second, which makes this the cheapest failure to iterate on:

```bash
export LD_LIBRARY_PATH=~/seahorn/seahorn/deps/z3-4.8.9/lib:~/seahorn/seahorn/deps/yices-2.6.1/lib
for b in 14 18; do
  ./build-$b/verify --expect=unsat --entry=test_zst \
    ./build-$b/src/rust-jobs/sea-vec/sea-vec_llvm.bc
done
# build-14: unsat, exit 0, 1.18s   (matches recorded baseline of 1.20s)
# build-18: sat,   exit 1, 0.92s
```

dev18 is not slower and does not struggle — it confidently reaches the opposite
answer. Findings from diffing the IR:

**The failing assertion, from the counterexample** (`./build-18/verify --cex
--entry=test_zst .../sea-vec_llvm.bc`):

```
tail call void @verifier.assert(i1 false)  [src/rust-jobs/sea-vec/lib.rs:249]
store i64 10, ptr %..DROP_COUNT..          [src/rust-jobs/sea-vec/lib.rs:228]
```

`lib.rs:249` is `verifier::vassert!(iter.next() == None);` — SeaHorn finds a path
where an **exhausted `IntoIter<ZST>` returns `Some`**.

**`assert(i1 false)` is not a mis-fold.** It is the natural codegen of
`assert(x == None)`: on the branch where `next()` returns `Some`, the assertion is
false. LLVM 18 splits the comparison into branches; LLVM 14 computed one boolean
and asserted it. Semantically equivalent — do not read the constant as LLVM
"proving the test wrong".

**`IntoIter<ZST>::next` is semantically identical in both builds.** Diffed
directly; modulo opaque pointers, both are `return start != end` with the address
bumped by 1 per step:

```llvm
; build-14                        ; build-18
%8  = icmp eq ptr %4, %7          %6 = icmp ne ptr %3, %5
%18 = xor i1 %8, true             ret i1 %6
ret i1 %18
```

So **the divergence is upstream of `next`** — in how `start`/`end` are established
(`into_iter`, `insert`, `remove`), or in how SeaHorn reasons about the synthetic
ZST pointer arithmetic (`ptrtoint` → `llvm.uadd.with.overflow.i64` → `inttoptr`).
For a ZST there is no real storage: `start`/`end` are fake addresses used as
counters, and exhaustion is pointer equality. That is exactly the kind of thing a
memory model can lose precision on.

**Hypotheses tested and killed** (recorded so nobody re-runs them):

| hypothesis | verdict |
|---|---|
| more `undef` reads confuse SeaHorn | **no** — build-14 has *more*: 287 vs 162 |
| `test_zst`'s body got more poison/undef | **no** — 138 vs 139 undef, 0 poison in both |
| `pop`/`next` newly un-inlined and havoc'd | **no** — both defined, called 3× in both |
| `into_iter` sets up `start`/`end` differently | **no** — identical field-for-field (below) |

**`into_iter` is not the divergence either.** Finding the right monomorphisation
takes care: there are three `SeaVec::into_iter` instances per module and the name
hashes differ between builds, so match them by tracing the call out of `test_zst`:

```bash
awk '/^define.*@test_zst\(/,/^}/' build-14/src/rust-jobs/sea-vec/sea-vec_llvm.ll \
  | grep -o 'into_iter17h[0-9a-f]*E' | sort -u
# build-14: hb59d52e37c4cb85b (line 7761)   build-18: hed4a2ba52c6fdd84 (line 8066)
```

Confirm it is the ZST one via the types: build-14 has `%5 = type {}` (the ZST) and
the SeaVec is `%6 = type { { i8*, i64 }, i64 }` — i.e. `{{buf, cap}, len}`, so
`len` is at **byte 16** and `cap` at **byte 8**. That decoder ring is what makes
the two comparable, since build-14 uses typed GEPs and build-18 byte offsets:

| step | build-14 | build-18 |
|---|---|---|
| load `buf` | offset 0 | offset 0 |
| load `len` | gep `0,1` → byte 16 | gep i8 16 |
| `end = buf + len` | `ptrtoint` → `uadd.with.overflow.i64` → `inttoptr` | identical |
| load `cap` | gep `0,0,1` → byte 8 | gep i8 8 |
| store iterator | `{buf, cap, buf, end}` | `{buf, cap, buf, end}` |

A raw `diff` of the two bodies looks enormous, but it is ~95% opaque-pointer
syntax (`%6* %1` → `ptr %1`, bitcasts gone, typed GEPs → `getelementptr i8`).
Strip the `llvm.dbg` intrinsics and decode the offsets before concluding anything.

**The one real difference:** build-14 calls
`core::intrinsics::is_aligned_and_not_null(buf)` and `llvm.trap()`s on failure;
build-18 dropped that check entirely, carrying the fact only as `!nonnull`/
`!noundef` metadata on the load. (The panic path also changed from
`core::panicking::panic` with a string to `panic_const_add_overflow` — cosmetic.)

This is worth a look but is **not** an obvious culprit, and I have not tested it:
the check only traps, and losing it makes build-18 *less* constrained rather than
differently behaved. It would only matter if SeaHorn honours an explicit branch but
ignores `!nonnull` metadata — plausible, but it is unclear why `buf` being non-null
would be needed to prove `start == end` after `len` steps. Test it before believing
it.

**Superseded lead (kept so it is not re-chased):** module-wide `poison` jumps
**184 → 1528 (8×)** in build-18, while
`test_zst`'s own body is unchanged — i.e. the poison is in the *callees*. The cex
trace shows `= ptr poison` and `self = ptr poison` inside `drop`. LLVM 18 uses
poison where LLVM 14 used undef/concrete values; if SeaHorn's memory model treats
those pointers as unconstrained, it cannot prove `start == end` at exhaustion, and
`sat` follows. **This was wrong** — the poison is a red herring, and the real cause
is pointer-comparison imprecision (see "Root cause" below). The 8× poison jump is
real but incidental; it cost a full round of investigation.

**The drops are identical too.** All four ZST drops (`ZST`, `IntoIter<ZST>`,
`SeaVec<ZST>`, `Drain<ZST>`) diff clean modulo opaque pointers. Note `Drain`'s drop
is reached from `drop(drain)` at lib.rs:263 — *after* the failing assert at 249 —
so it cannot explain this failure regardless. The two that matter for line 249,
`drop_in_place<ZST>` and `drop_in_place<IntoIter<ZST>>`, are identical.

`next_back` (lib.rs:247, the call right before the failing assert) is identical as
well.

### The experiment that matters: SeaHorn is not the variable

`build-14` vs `build-18` confounds **two** changes — the bitcode *and* the SeaHorn
binary. Newer LLVM reads older bitcode, so dev18 can be run against the dev14
module to separate them:

| SeaHorn | bitcode | verdict |
|---|---|---|
| dev14 | build-14 | `unsat` |
| dev14 | build-18 | error 3 (cannot read LLVM 18 bitcode — expected, not a finding) |
| **dev18** | **build-14** | **`unsat`** |
| dev18 | build-18 | `sat` |

```bash
./build-18/verify --expect=unsat --entry=test_zst \
  ./build-14/src/rust-jobs/sea-vec/sea-vec_llvm.bc     # -> unsat, exit 0
```

**dev18 proves the property `unsat` given the LLVM 14 bitcode.** So SeaHorn dev18 is
exonerated for this test, and the cause is in the *bitcode* — i.e. the rustc
1.81 / LLVM 18 codegen, consistent with this migration's one-line lesson. Run this
2×2 first for any future Class 3 disagreement; it is two commands and it halves the
search space.

### Root cause: fat pointers compared by representation, not address

**The cex trace answers it outright — read the trace before diffing anything.**
Four hand-diffs of the IR (`test_zst`, `next`, `next_back`, `into_iter`, the drops)
all came back "identical" and cost hours. `--cex` prints the concrete values and
names the exact instruction, in one run:

```bash
./build-18/verify --cex --entry=test_zst ./build-18/src/rust-jobs/sea-vec/sea-vec_llvm.bc
```

The failing comparison, with SeaHorn's own values attached:

```
%.not47 = icmp eq ptr %_63, %_72   ->  false     [src/lib/seavec/src/sea_vec.rs:280]
%_63 = getelementptr i8, ptr %_36, i64 -1        [sea_vec.rs:311]   end
  %_63 struct(4:bv(64), -1:bv(64), 0:bv(64))
%_72 = inttoptr i64 %_71 to ptr                  [sea_vec.rs:285]   start
  %_72 struct(3:bv(64),  0:bv(64), 0:bv(64))
```

SeaHorn pointers are `struct(base, offset, size)` — confirm the format elsewhere in
the same trace: `self = struct(0xbf6fff40, 0, 32)` and `struct(0xbf6fff40, 16, 32)`
are the `IntoIter` (32 bytes) and its `start` field at offset 16.

**These are not distinct allocations.** `base`/`offset` is a *decomposition of one
address*, not an object identity, and nothing here is allocated. Real objects show a
stack base and nonzero size (`0xbf6fff40`, size 32). The ZST pointers have **size 0**
and bases `1, 2, 3, 4` — the literal counter values, starting from
`NonNull::dangling()` = 1 for align 1. The whole ZST iterator is pointer arithmetic
on fake addresses that are never dereferenced. The full sequence from the trace:

| SSA | triple | origin |
|---|---|---|
| `%_7`, `%_25` | `struct(1, 0, 0)` | `dangling()` -> `start` = 1 |
| `%_36` | `struct(4, 0, 0)` | `into_iter`: `inttoptr(1+3)` -> `end` = 4 |
| `%_49` | `struct(2, 0, 0)` | `next`: `inttoptr(1+1)` |
| `%_63` | `struct(4, **-1**, 0)` | `next_back`: `GEP(%_36, -1)` -> addr 3 |
| `%_72` | `struct(3, 0, 0)` | `next`: `inttoptr(2+1)` -> addr 3 |

So `end` = base 4, offset -1 = **address 3**. `start` = base 3, offset 0 =
**address 3**. *The two pointers denote the same address* — the iterator really is
exhausted and the assertion at lib.rs:249 is true. But `icmp eq` compares the
`(base, offset)` representation, 4 != 3, so it answers "not equal", `next()`
returns `Some`, and the assert fails.

The arithmetic (ZST align 1 -> `NonNull::dangling()` = 1, len 3 -> `end` = 4):

| lib.rs | op | start | end |
|---|---|---|---|
| 244 | `into_iter` | base 1 | base 4 |
| 245 | `next`: `start += 1` | 2 | 4 |
| 247 | `next_back`: `end -= 1` | 2 | **base 4, offset -1** |
| 248 | `next`: `start += 1` | **base 3, offset 0** | base 4, offset -1 |
| 249 | `start == end`? | addr 3 | addr 3 -> reported unequal |

**Why only under LLVM 18.** The two counters take different routes to the same
address, because of the checked-arithmetic canonicalisation:

- `start += 1` (`sea_vec.rs:285`) keeps `uadd.with.overflow` — the trace still shows
  `extractvalue { i64, i1 } %_67, 0`. The intrinsic **blocks** the
  `inttoptr(ptrtoint(p) + c)` -> `getelementptr` fold, so `start` stays an
  `inttoptr`: base becomes the literal value, offset 0.
- `end -= 1` (`sea_vec.rs:311`) lost its intrinsic to LLVM 18's `icmp` + plain
  `add i64 %x, -1` form, which **unblocks** that fold. `end` becomes
  `getelementptr i8, %base, -1`: base preserved, offset -1.

Under LLVM 14 both sides kept their `*.with.overflow` intrinsics, both stayed
`inttoptr`, and both had offset 0 — so comparing representations happened to agree
with comparing addresses. The proof was relying on that coincidence.

**How strong is the claim that `ptrEq` is "buggy"? Weaker than stated below.**
Read this first — the sections that follow overstate it.

The defensible claim: the `.ll` tests are legal LLVM IR, LLVM's `icmp eq` on
pointers is *address* comparison, and SeaHorn answers wrongly on them. Narrowly,
that is a bug.

The counter-argument, which is strong: if the model maintains the invariant that
distinct bases never yield equal addresses (allocations disjoint *and*
non-adjacent, offsets within `[0, size]`), then
`base1+off1 == base2+off2` **iff** `base1==base2 && off1==off2`. Componentwise is
then not merely correct but *cheaper* — two narrow equalities instead of an
addition — i.e. a deliberate encoding choice. The only thing that breaks the
invariant here is `inttoptr` minting a base out of an address, plus a GEP walking
`offset` to -1 on a size-0 object (outside `[0, size]`).

Attempts to break `ptrEq` on **provenance-clean** code (one-past-the-end, where
`a+1` and `b` may legitimately share an address with different provenance) **all
failed**: LLVM folds the tautology before SeaHorn sees it ("no assertion was found
... front-end discharged them" — the Class 1 failure mode again). So there is **no
evidence** `ptrEq` misbehaves on provenance-preserving IR. The `ptrEq`/`ptrNe`
inconsistency below is likewise a source-level fact with **no runtime witness** —
latent, not demonstrated.

This suggests the principled SeaHorn fix may not be `ptrEq` at all, but modelling
`inttoptr` as a wildcard pointer of unknown provenance instead of synthesising a
base from the integer and letting a structural comparison read meaning into it.

**Direction of the bug: on `inttoptr`-derived pointers it is wrong BOTH ways.** The
first read of this was that it is a false alarm only ("errs toward reporting a bug
that is not there — the safe direction"). That is wrong. Asserting the *negation*
shows the other direction:

| assertion | truth | widemem | extra-widemem |
|---|---|---|---|
| `assert(p == q)` | holds | `unsat` (correct) | `sat` — false alarm |
| `assert(p != q)` | **fails** | `sat` (correct) | `unsat` — **misses a real violation** |

So `extra-widemem` also **proves disequalities that do not hold**, which silently
drops genuine bugs. This is a soundness hole, not just imprecision. Both directions
are pinned by tests (below).

The negated case doubles as the **vacuity control**: `widemem` returning `sat` there
proves the path is reachable, so `widemem`'s `unsat` on the equality is a real proof
and not a vacuous one.

**Note the trace is of the post-`seaopt`/`seapp` module, not the input `.bc`.** That
is why hand-diffing the `.ll` kept showing "identical": `next_back` is `inttoptr` in
build-18's input bitcode and only becomes a `getelementptr` inside SeaHorn's own
pipeline. Diffing input bitcode cannot see this class of divergence.

### Fixing it

**The bug is one function**, `ExtraWideMemManagerCore<T>::ptrEq` in
`seahorn/lib/seahorn/BvOpSem2ExtraWideMemMgr.cc:65-75`:

```cpp
// NOTE: we consider two pointers to be same if their address (base and ofset)
//       is the same. Size is ignored. This is done to have parity with memset
//       like operations that zero out main memory but do not touch shadow memory.
return mk<AND>(m_main.ptrEq(p1.getBase(), p2.getBase()),
               m_offset.ptrEq(p1.getOffset(), p2.getOffset()));
```

That is `base1 == base2 && offset1 == offset2` — **strictly stronger** than
`base1 + offset1 == base2 + offset2`. The comment conflates "the address" with its
decomposition; that holds only while every pointer to an address carries the same
decomposition, and breaks as soon as two spellings exist (`inttoptr` -> `(3,0)`
vs. `GEP` -> `(4,-1)`).

The sibling managers compare a single raw value and are unaffected — the contrast
is the clearest statement of the bug:

```cpp
RawMemManagerCore::ptrEq  ->  doEq(p1.toExpr(), p2.toExpr())          // correct
WideMemManager::ptrEq     ->  m_main.ptrEq(p1.getRaw(), p2.getRaw())  // correct
ExtraWideMemManagerCore   ->  base AND offset, componentwise          // the bug
```

Note this affects `ptrEq` only; `ptrNe` and the ordering predicates deserve the same
audit — they were not checked.

**`ptrEq` contradicts its own sibling `ptrNe`** — in the same class, ~670 lines
apart:

```cpp
Expr ExtraWideMemManagerCore<T>::ptrNe(PtrTy p1, PtrTy p2) const {        // line 740
  return m_main.ptrNe(getAddressable(p1), getAddressable(p2));            // base+offset
}
Expr ExtraWideMemManagerCore<T>::getAddressable(PtrTy p) const {
  ...
  return m_ctx.alu().doAdd(p.getBase(), p.getOffset(), ptrSizeInBits());  // base + offset
}
```

`ptrNe` resolves the effective address via `getAddressable`; `ptrEq` never calls it.
So the two disagree about what pointer identity means, and **the fix is to mirror
`ptrNe` using the helper that already exists**:

```cpp
return m_main.ptrEq(getAddressable(p1), getAddressable(p2));
```

If both notions meet on the same pointers, SeaHorn can derive `!(p == q) && !(p != q)`
— a contradiction that makes paths spuriously infeasible, which is a second route
by which real bugs get pruned. Worth checking the ordering predicates
(`ptrUlt`/`ptrSle`/...) for the same split; they were not audited.

### The upstream cause is `sea_vec.rs` losing pointer provenance (fix verified)

The framing above — "a SeaHorn bug" — is only half the story, and the half that
matters least in practice. **`sea_vec.rs` should never have produced an `inttoptr`
here.** Every ZST path uses Rust's *exposed-provenance* int round trip, while the
non-ZST path one line away uses proper pointer arithmetic:

```rust
267:  end = ((slice.as_ptr() as usize) + slice.len()*mem::align_of::<T>()) as *const _  // ZST
271:  end = slice.as_ptr().add(slice.len())                                             // non-ZST, fine
285:  self.start = (self.start as usize + 1*mem::align_of::<T>()) as *const _           // ZST
311:  self.end   = (self.end   as usize - 1*mem::align_of::<T>()) as *const _           // ZST
```

`ptr as usize ... as *const _` discards provenance — the result has no link to the
original allocation. That is what makes rustc emit `inttoptr`, which is what makes
SeaHorn mint a fresh base, which is what makes the two decompositions diverge. The
codegen difference is exactly two instructions (verified on nightly-2024-07-01):

```llvm
; (p as usize + 1) as *const _        -> provenance lost
%_3 = ptrtoint ptr %p to i64
%_2 = add i64 %_3, 1
%_0 = inttoptr i64 %_2 to ptr

; p.wrapping_byte_add(1)              -> provenance preserved
%0 = getelementptr i8, ptr %p, i64 1
```

**Why the int cast is there at all** (it is not carelessness). Adding 1 to a ZST
pointer does not *generate* a `ptrtoint` — the obvious call generates *nothing*:

```llvm
; p.add(1) where T is a ZST
define ptr @zst_add(ptr readnone returned %p) { ret ptr %p }   ; a NO-OP
```

`.add(n)` is a GEP by `n * size_of::<T>()`, and `size_of::<ZST>() == 0`, so it moves
zero bytes and cannot advance the counter. The author needed *byte-level* movement,
which the typed pointer API will not give; the pre-strict-provenance idiom for that
was `(p as usize + 1) as *const _`, and that is an int round trip. So ZST-ness is
the *reason* the cast gets reached for, but `ptrtoint` is a property of the idiom,
not of ZST arithmetic. The three forms, verified on nightly-2024-07-01:

| source | LLVM | provenance |
|---|---|---|
| `p.add(1)` | `ret ptr %p` | n/a — no-op, useless here |
| `(p as usize + 1) as *const _` | `ptrtoint`/`add`/`inttoptr` | **lost** |
| `p.wrapping_byte_add(1)` | `getelementptr i8, ptr %p, i64 1` | preserved |

**Rust's own std gets this right** — `alloc/src/vec/into_iter.rs` on this toolchain:

```rust
203:  let ptr = if T::IS_ZST {
204:      if self.ptr.as_ptr() == self.end as *mut T { return None; }
207:      // `ptr` has to stay where it is to remain aligned, so we reduce the length by 1 by
208:      // reducing the `end`.
209:      self.end = self.end.wrapping_byte_sub(1);
225:  self.end.addr().wrapping_sub(self.ptr.as_ptr().addr())   // .addr(), not `as usize`
```

Two lessons from std, beyond the API choice. It uses `.addr()` (strict provenance)
rather than `as usize` for the length computation. And it keeps `ptr` **fixed**,
moving only `end` — both `next` and `next_back` do `end.wrapping_byte_sub(...)`.
`sea_vec.rs` instead moves `start` up and `end` down, which is what produced two
pointers derived by different routes. The `wrapping_byte_*` patch makes that
harmless (both become GEPs off one base), but std's shape avoids the class outright.

**Where this idiom came from, and where else it lives.** `sea_vec.rs` is derived from
the Rustonomicon's Vec chapter (`custom-vec/lib.rs:296` cites it directly), which
predates strict provenance.

Auditing the Nomicon-derived jobs, only two actually **create a pointer** via the
int round trip (the bug shape). Both are now fixed the same way as `sea_vec.rs`:

| job | ZST pointer-creation sites fixed |
|---|---|
| `custom-vec-final` | lib.rs:710 (end init), 730 (`next`), 758 (`next_back`), 838 (Drain) |
| `custom-vec-zst-alignment-issue` | lib.rs:116 (end init), 136 (`next`), 160 (`next_back`) |

After the fix both verify `unsat` (correct) at baseline speed — `custom-vec-final`
1.16s->0.30s, `custom-vec-zst-alignment-issue` 0.21s->0.25s — and the ZST
`next`/`next_back` bodies are `getelementptr`, no int round trip. (A residual
`inttoptr` still shows in those functions but it is **debug metadata only** —
`metadata ptr inttoptr (i64 1 to ptr)`, recording `NonNull::dangling()` = addr 1 —
not an instruction. Grep with `grep -v llvm.dbg` before counting, or it misleads.)

The other three custom-vec jobs need **no** change: `custom-vec`, `custom-vec-drain`,
`custom-vec-raw_vec` have no ZST branch at all and no int-cast pointer creation —
they advance with `.offset()` (already `getelementptr inbounds`). Their only `as
usize` use is the *length* subtraction `(end as usize - start as usize)`, which
yields an integer, never a compared pointer, so it cannot trigger this bug. (Same
as `sea_vec.rs` line 298, left untouched there too. Converting these to `.addr()`
would be provenance hygiene matching std, but is not required for correctness.)

This also **weakens the Class 2 correlation**: `custom-vec_test_deref` /
`custom-vec_test_insert` (both Class 2 blow-ups) live in the `custom-vec` job, which
never had this idiom — so the ZST-provenance bug is *not* their cause. Class 2
remains unexplained.

With provenance preserved, every pointer is a GEP off the same base, so `start` and
`end` are `(1, 2)` and `(1, 2)` — bases match, offsets match, and even the
componentwise `ptrEq` returns the right answer.

**Verified end to end.** Patching the three ZST sites to
`wrapping_byte_add`/`wrapping_byte_sub` and rebuilding:

| entry | dev14 baseline | dev18 before | dev18 + provenance fix |
|---|---|---|---|
| `test_zst` | 1.20s `unsat` | **`sat`** | **`unsat`, 1.0s** |
| `test_macros` | 1.21s `unsat` | **>605s timeout** | **`unsat`, 347s** |
| other 7 sea-vec entries | ~1.1-1.2s | pass | pass, ~0.9-1.7s, no regressions |

`inttoptr` count in `next`/`next_back` after the fix: **0**.

So the fix resolves Class 3 outright *and* converts the `test_macros` Class 2
timeout into a pass — but note 347s vs a 1.21s baseline is still a ~287x
regression. The blow-up is reduced, not explained. Do not call Class 2 solved.

### Fixing it

Ordered by what actually matters:

1. **Fix `sea_vec.rs` to preserve provenance** (verified above). This is not
   "editing the code under test to appease the verifier" — an earlier version of
   this note said that and was wrong. Exposed-provenance round trips are a genuine
   defect: they are what `strict_provenance` exists to remove, they inhibit
   optimisation, and they are UB-adjacent under Miri's strict-provenance checks.
   Fixing them makes the Rust better *and* the proof robust.
   **Is fixing the Rust *enough*, without touching SeaHorn?** Mostly, but not as a
   general policy. `inttoptr` is pervasive in code you do not control — counts in
   the verified bitcode:

   ```
   borsh-primitives 4952   copy_stack_buffer 4244   custom-vec-raw_vec 1612
   borsh            4914   custom-print      4227   smallvec-*        ~1550
   borsh-structs    4891   custom-vec        1599   sea-vec            231  <- post-fix, passes
   ```

   Those come from std/core/alloc, formatting, and hashing — not from this repo.
   *But* `sea-vec` still has 231 `inttoptr` after the fix and verifies fine, so
   presence alone is harmless: it only bites when two **differently-derived
   pointers to the same address get compared**, which is rare and is exactly what
   the Nomicon's start-up/end-down iterator does. So: fix the Rust first, and treat
   the SeaHorn change as insurance whose value is currently unquantified.

   **Untested lead for Class 2:** the jobs with the most `inttoptr` are almost
   exactly the Class 2 blow-up list — all five `borsh-*`, `copy_stack_buffer`,
   `custom-print`, `custom-vec` (8 of the top 10). If imprecise pointer reasoning
   forces case splits, that would explain the blow-ups. It is a **correlation, not
   a finding** — `custom-vec-loop-unroll` has 4236 and passes — but it is the first
   real lead on Class 2 and is cheap to test.

2. **Decide what SeaHorn should do about `inttoptr` at all.** Two candidates, and
   the second is probably the principled one:
   - Make `ptrEq` use `getAddressable`, mirroring `ptrNe`. Looks strictly better:
     identical answers on provenance-clean code (given the invariant), right
     answers on `inttoptr`-derived pointers, at the cost of one addition. Needs
     measuring, and the `memset` parity the original comment cites re-checked.
   - Or model `inttoptr` as a **wildcard pointer of unknown provenance** rather
     than synthesising `base` from the integer. `inttoptr` genuinely *has* no
     provenance; giving it a base and then comparing bases structurally reads
     meaning into a value that carries none. Fixing `ptrEq` treats the symptom of
     that modelling choice.

   Either way this is defence in depth, not the root cause — and per the caveat at
   the top of this section, there is no evidence of misbehaviour on
   provenance-clean IR, so do not oversell it as "SeaHorn is broken".
3. Audit the other jobs for the same `as usize ... as *const` idiom — anything
   doing ZST or byte-level pointer arithmetic is a candidate for the same failure.

Upstream this: it is a SeaHorn bug, reproducible in ~1s, and independent of this
repo. Any ZST iterator that advances from both ends can hit it.

### Regression tests (written, in the seahorn tree)

Two lit tests in `seahorn/test/opsem2/widemem/`, reduced to ~20 lines of IR with no
Rust involved:

| test | asserts | correct | widemem | extra-widemem |
|---|---|---|---|---|
| `ptr_eq_inttoptr_vs_gep.ll` | `p == q` | `unsat` | `unsat` | `sat` |
| `ptr_ne_inttoptr_vs_gep_missed_bug.ll` | `p != q` | `sat` | `sat` | `unsat` |

Both are **differential**: each runs the same IR under `--horn-bv2-widemem` (passes)
and `--horn-bv2-extra-widemem` (fails). The manager is the only variable, so the
`widemem` line doubles as proof that the test is well-formed and the property really
is provable.

They must be `.ll`, not `.c`: clang folds `gep(inttoptr(4), -1)` straight into
`inttoptr(3)`, which erases the very distinction under test. For the same reason the
values come from `nd_int()` + `assume` rather than literals.

Both carry `XFAIL: *` so the suite stays green while the bug is open. When `ptrEq` is
fixed, lit reports XPASS (which it treats as a failure) and the `XFAIL` lines should
be deleted. **Do not "fix" these by flipping CHECK to match the buggy output** — the
CHECK lines state the mathematically correct verdicts.

Caveat: `lit`/`filecheck` are not installed on this machine, so the RUN-line and
XFAIL plumbing is unverified — only the underlying verdicts were confirmed, by
invoking `sea` directly with the flags `%sea` expands to.

Check whether the other Class 2 sea-vec blow-ups share this cause before fixing: if
imprecise pointer equality is forcing extra case splits, it could explain
`sea-vec_test_macros` (1.21s -> >605s) too.

### Suggested order of attack

1. `sea-vec_test_zst` (`sat` vs `unsat`) — smallest, and the only possible soundness signal.
2. The borsh six — first rule out ahash 0.8.11 as the cause.
3. Class 1 — mechanical: make the inputs opaque so assertions survive `-O3`.
4. The remaining blow-ups — likely the deepest work.

## What this migration taught, in one line

Every single failure was in the **Rust/C toolchain jump** (2022→2024 nightly,
clang-14→18), not in SeaHorn. dev18's `seaopt`/`seapp` consumed LLVM 18 bitcode
without complaint from the first build attempt onward. If you are planning a
similar move, budget your time accordingly: the SeaHorn side is the easy half.
