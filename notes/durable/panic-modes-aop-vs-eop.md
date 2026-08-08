# Panic modes: abort-on-panic (AoP) vs error-on-panic (EoP)

[FACT] Every job verifies in one of two panic modes, and the choice decides
whether panic-class defects — arithmetic overflow, bounds violations, alloc
failure — are **checked** or **assumed away**.

- **AoP**: a panic lowers to `llvm.trap` / `unreachable`. SeaHorn prunes the
  path. Properties are proved only over non-panicking executions. Fast.
- **EoP**: a panic reaches a verifier error. Panic-freedom becomes part of what
  is verified. Strictly more checking, and slower.

**EoP is the standard; AoP is the named variant** (`<job>-aop`). → the comment in
`src/rust-jobs/tinyvec-arrayvec-aop/CMakeLists.txt`.

## Selecting the mode — two mechanisms, do not mix them

[FACT] **no_std jobs** need **both** halves. Neither works alone:

| half | where | effect |
|---|---|---|
| `CUSTOM_PANIC_NO_STD` | job `CMakeLists.txt`, on `c_rust_llvm(...)` | builds std with `-Zbuild-std-features=default` instead of `panic_immediate_abort`, so a panic reaches the `#[panic_handler]` |
| `features = [..., "panic_error"]` | job `Cargo.toml`, on the `verifier` dep | forwards to `sea/panic_error`; the handler calls `verifier_error()` → `src/sea-lib/src/error_handle.rs:21` |

The Cargo feature alone changes nothing: `panic_immediate_abort` lowers the
panic branch to `llvm.trap`/`unreachable` and the handler is never reached.
Verified by inspecting `<job>_llvm.ll` before and after.

[FACT] **std jobs cannot use that route at all** — `sea` rejects `std` and
`panic_error` as mutually exclusive (`error: Features "std" and "panic_error"
are mutually exclusive`). They use the bitcode-override token instead:
`ABSTRACT_PANIC` (abort) / `ABSTRACT_PANIC_ERROR` (error), which
`llvm-link --override`s `rust_begin_unwind` with `src/sea-lib/panic_abstract.c`.
→ `cmake/SeaHornBuild.cmake:100-110`

## The override requires the primitives to stay EXTERNAL

[FACT] `src/sea-lib/seahorn.c` defines `__VERIFIER_error`, `__VERIFIER_assume`,
`__SEA_assume`, `__VERIFIER_assert` as deliberately empty stand-ins (SeaHorn
strips their bodies). `panic_abstract.c` *references* two of them as external
symbols. If LTO internalizes the definitions, LLVM cannot hold an internal and
an external symbol of the same name, so `llvm-link` **renames the definition** to
`__VERIFIER_error.N`.

The consequence is silent and severe: `sassert` expands to
`(__VERIFIER_assert(X), (X)) || (__VERIFIER_error(), 0)` (`seahorn.h:80`), and
SeaHorn identifies the assertion by the **branch to `__VERIFIER_error`**, not by
the `__VERIFIER_assert` marker (which has an empty body). Rename the target and
SeaHorn reports **"no assertion was found"** while the assertion is still
present in the bitcode.

Hence `KEEP_EXTERNAL` (`used` + default visibility) on those four in
`seahorn.c`. Check it survived:

```bash
grep -E "^define.*__VERIFIER_(error|assume)\(" build-18/src/rust-jobs/<job>/<job>_llvm.ll
# want `define dso_local`, NOT `define internal`
grep -oE "@__VERIFIER_(error|assume)\.[0-9]+" build-18/src/rust-jobs/<job>/<job>_llvm.ll
# want no output; ANY numeric suffix means the override collided
```

[FACT] This is a property of the *linkage*, not of the stub. Three-way
`llvm-link` experiment on `option-or` (2026-08-08):

| job's `__VERIFIER_error` | stub | renamed | SeaHorn |
|---|---|---|---|
| `internal` | declares it | `.2` | no assertion found |
| `internal` | **defines** it | `.4` | no assertion found |
| **external** | declares it | none | `unsat` |

Making the stub define the symbols does **not** help — a second definition still
cannot merge with an internal one.

## Failure signatures

- `no assertion was found` right after adding a panic token → symbol rename;
  check for `.N` suffixes.
- A job declares `panic_error` in Cargo.toml but has no CMake token → it is
  silently running AoP. 13 jobs were in this state before 2026-08-08.

## See also

- ../journal/2026-08/2026-08-08-eop-conversion-and-override-linkage-bug.md
- ../journal/2026-08/2026-08-07-kani-vs-seabmc-sweep.md (kani checks panics by
  default; this is the main reason its results differ from seabmc's)
