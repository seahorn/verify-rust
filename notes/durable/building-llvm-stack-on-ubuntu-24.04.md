# Building the LLVM 14 stack on Ubuntu 24.04 (noble)

[FACT] Every build failure documented here is a version-skew problem between a
pinned-old component (LLVM 14) and a moving-newer distro, not a SeaHorn bug. The
fixes are stable environment reality, re-verified building on this host.
→ see the three numbered symptoms below (`-lstdc++`, `libpolly`, `z3`).

The `README.md` build instructions assume Ubuntu 22.04 (jammy), which is what the
CI Docker image (`seahorn/seahorn-llvm14:nightly`) is. On a 24.04 host everything
is still *installable* — `clang-14`, `lld-14`, `llvm-14-dev`, `libpolly-14-dev`
are all in `noble/universe` — but three things fail in ways that look unrelated to
LLVM. All three come from the same root cause: **LLVM 14 is old enough that the
rest of the distro has moved past the assumptions it was built with.**

## The one idea to take away

Every problem below is a version-skew problem between a pinned-old component and a
moving-newer one. When something here breaks in a way these notes don't cover, the
first question is *"which two components disagree about a version, and which one is
pinned?"* — not *"what flag do I add?"*

---

## 1. clang-14 cannot find `-lstdc++`

### Symptom
`cmake` dies at the compiler check, before compiling any project source:

```
/usr/bin/ld: cannot find -lstdc++: No such file or directory
clang: error: linker command failed with exit code 1
```

### Why
Clang discovers a GCC installation at runtime by scanning
`/usr/lib/gcc/x86_64-linux-gnu/` and **selecting the highest version number it
finds**. On noble that directory contains both `13` and `14`:

```
$ ls /usr/lib/gcc/x86_64-linux-gnu/
13  14
```

The `14` directory is created by `libgcc-14-dev` (pulled in as a transitive
dependency) and contains only C runtime objects — `crtbegin.o`, `crtend.o`, and
friends. There is no `libstdc++.so` in it, because the C++ half lives in
`libstdc++-14-dev`, which is not installed. So clang selects GCC 14, then looks
for libstdc++ under GCC 14, and finds nothing.

Confirm the selection with:

```bash
clang++-14 -v -x c++ /dev/null -fsyntax-only 2>&1 | grep -i "Selected GCC"
```

### The trap
The obvious fix — `apt install libstdc++-14-dev` — makes this error disappear and
replaces it with a worse one. libstdc++ 14's headers require **clang 18 or newer**
(they use the `__is_convertible` builtin, among others). You would be feeding
clang-14 headers it cannot parse. Do not do this.

### The fix
Point clang-14 at GCC 13 instead. `--gcc-install-dir=` would be the direct way to
say this, but **it does not exist before clang 16** — on clang-14 you get
`error: unsupported option '--gcc-install-dir='`.

What clang-14 does support is `--gcc-toolchain=<prefix>`, which changes *where* it
scans. It still picks the highest version under that prefix — so give it a prefix
where 13 is the only choice:

```bash
SHIM=~/.local/share/gcc13-shim
mkdir -p $SHIM/lib/gcc/x86_64-linux-gnu
ln -sfn /usr/lib/gcc/x86_64-linux-gnu/13 $SHIM/lib/gcc/x86_64-linux-gnu/13
```

Then pass it to every clang-14 build (both this project and SeaHorn itself):

```bash
-DCMAKE_C_FLAGS="--gcc-toolchain=$SHIM" -DCMAKE_CXX_FLAGS="--gcc-toolchain=$SHIM"
```

The shim is just symlinks — it costs nothing and touches nothing outside your home
directory. That is the point: it is a *view* of the system toolchain, not a change
to it.

---

## 2. `libPolly.a` missing

### Symptom
```
ninja: error: '/usr/lib/llvm-14/lib/libPolly.a', needed by 'bin/seaopt', missing
```

### Why
`llvm-14-dev` does not pull in Polly; `libpolly-14-dev` is a separate package.
SeaHorn's `seaopt` links it unconditionally.

### Fix
```bash
sudo apt install libpolly-14-dev
```

This is easy to miss if you already have `libpolly-18-dev` installed for a dev18
build — the `-18` package being present says nothing about `-14`.

### Generalising
Before assuming you have the whole LLVM 14 install, ask ninja what it wants and
check each file exists:

```bash
grep -ohE "/usr/lib/llvm-14/lib/lib[A-Za-z0-9_]+\.(a|so)" build.ninja |
  sort -u | while read f; do [ -f "$f" ] || echo "MISSING: $f"; done
```

---

## 3. Corrosion cannot find `rustc` ("Rust not installed or ~/.cargo/bin not added to path?")

### Symptom
`cmake` fails inside Corrosion even though `rustc` is plainly on `PATH` and works:

```
CMake Error at .../FindRust.cmake:21 (message):
  The rustc executable was not found. Rust not installed or ~/.cargo/bin not
  added to path?
```

### Why
This is a version skew between **Corrosion 0.3.5** (pinned in `CMakeLists.txt` via
`FetchContent`) and **rustup ≥ 1.28**, which changed the output format of
`rustup toolchain list --verbose`. Modern rustup marks the active toolchain:

```
nightly-2022-08-01-x86_64-unknown-linux-gnu (active, default) /home/you/.rustup/toolchains/...
nightly-2024-07-01-x86_64-unknown-linux-gnu                   /home/you/.rustup/toolchains/...
```

Corrosion 0.3.5's regex only knows `(default)`, `(override)`, and
`(default) (override)` — it has never heard of `(active, default)`. The regex still
matches, but the marker it fails to recognise gets swallowed by the trailing
path capture. Corrosion ends up believing the toolchain path is literally
`"(active, default) /home/you/.rustup/toolchains/..."` and looks for rustc under
it. Hence "not found", despite rustc being right there.

Note the tell: **only the toolchain rustup marks active is corrupted.** Any
toolchain without a marker parses fine. The active one is whichever
`rust-toolchain.toml` pins, so this always hits the toolchain you actually want.

### Fix
Point Corrosion straight at the binaries — this is what CI already does
(`.github/workflows/main.yml`), so it is the project's established pattern, not a
local hack:

```bash
-DRust_COMPILER=$(rustup which rustc) -DRust_CARGO=$(rustup which cargo)
```

### The gotcha inside the gotcha
Use `Rust_COMPILER` / `Rust_CARGO`. **Do not** try to set `Rust_COMPILER_CACHED` /
`Rust_CARGO_CACHED` directly — it looks like it should work and silently does not.
`FindRust.cmake` contains:

```cmake
set(_Rust_USER_VARS Rust_COMPILER Rust_CARGO Rust_CARGO_TARGET Rust_CARGO_HOST_TARGET)
foreach(_VAR ${_Rust_USER_VARS})
    if (DEFINED "${_VAR}")
        set(${_VAR}_CACHED "${${_VAR}}" CACHE INTERNAL "...")
    else()
        unset(${_VAR}_CACHED CACHE)     # <-- wipes anything you passed with -D
    endif()
endforeach()
```

If you set only the `_CACHED` variable, the `else()` branch unsets it before
`find_program` runs, and you are back to the broken path. The user-facing variable
is the supported knob; the `_CACHED` one is derived from it.

### The real fix (not done)
Bump the Corrosion `GIT_TAG` in `CMakeLists.txt` to a release that understands
rustup 1.28+ output. That is a deliberate dependency upgrade with its own
compatibility surface, so it is tracked separately from any LLVM work.

---

## 4. SeaHorn `dev14` requires Clam; `dev18` does not

Worth knowing before you try to save build time by skipping it.

`dev18` has a `WITH_CLAM` cmake option and its build sets `WITH_CLAM=OFF`.
`dev14` has **no such option** — it auto-detects the `clam/` directory that the
`extra` target clones, and `lib/seahorn/BvOpSem2.cc` includes
`clam/ClamQueryAPI.hh` unconditionally. Delete `clam/` on dev14 and you get:

```
fatal error: 'clam/ClamQueryAPI.hh' file not found
```

So on dev14 you must also build crab:

```bash
cmake --build . --target extra && cmake ..
cmake --build . --target crab  && cmake ..
cmake --build . --target install
```

## Reusing prebuilt Z3 / Yices

Both branches want the same solver versions (`find_package(Z3 4.8.9)`,
`find_package(Yices2 2.6.0)`). If you already have a SeaHorn tree with populated
`deps/`, point a second build at them instead of rebuilding Z3 from source:

```bash
-DZ3_ROOT=/path/to/seahorn/deps/z3-4.8.9
-DYICES2_HOME=/path/to/seahorn/deps/yices-2.6.1
```

Those `deps/` entries are plain `bin/include/lib` trees, not build artifacts tied
to a branch.

## cbindgen headers go stale and ninja will not tell you

Worth knowing before you debug a "missing prototype" error that makes no sense.

`c_rust_llvm` in `cmake/SeaHornBuild.cmake` wires the generated header like this:

```cmake
add_custom_command(
    OUTPUT  ${CMAKE_CURRENT_BINARY_DIR}/inc/lib.h
    COMMAND cbindgen ...
    DEPENDS ${RUST_LIB_TARGET}     # <-- a TARGET, not the .rs sources
)
```

The dependency is on the Corrosion target, not on `lib.rs`. Editing a job's Rust
source therefore does **not** mark `inc/lib.h` out of date — cbindgen runs once and
the header sticks forever. `ninja` cheerfully reports `no work to do` while
compiling C against a header generated from an older version of the crate.

If you change a job's exported signatures, force it:

```bash
rm -f build/src/rust-jobs/<job>/inc/lib.h && ninja
```

And when validating something that matters (a toolchain move, a SeaHorn bump),
build from a fresh directory rather than trusting an incremental green.

**Debugging tip.** If a generated header comes out empty or missing declarations,
run cbindgen by hand in the job's source directory and read its stderr — the CMake
build swallows it:

```bash
cd src/rust-jobs/<job> && cbindgen --config ../../../cbindgen.toml
```

cbindgen shells out to `cargo metadata`, so it fails whenever anything in the
workspace lockfile fails to resolve — even a dependency your crate never compiles.
An empty header is the symptom; the cargo error is the cause.

## A harness note that cost real time

`cmake --build . 2>&1 | tail -40` **reports the exit code of `tail`, not the
build.** A failing build looks like a success. Either capture the status
explicitly or skip the pipe:

```bash
cmake --build . --target install > build.log 2>&1; echo "EXIT=$?"
```
