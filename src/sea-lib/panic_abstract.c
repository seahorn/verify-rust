// Panic abstraction stub, llvm-link --override'd into a job's bitcode by
// c_rust_llvm when the ABSTRACT_PANIC / ABSTRACT_PANIC_ERROR token is set.
//
// std jobs (e.g. borsh) link the full Rust unwinding + message-formatting
// runtime through `rust_begin_unwind`, because their dependencies use
// catch_unwind / set_hook. `panic_immediate_abort` only rewrites the panic!
// macro and cannot remove that. Overriding `rust_begin_unwind` here makes the
// whole panic/fmt subgraph dead so GlobalDCE strips it, and gives the same two
// panic semantics that no_std jobs already get via error_handle.rs:
//
//   default (ABSTRACT_PANIC)        -> abort-on-panic: assume the panic path is
//                                      infeasible (prunes panicking executions).
//   ABSTRACT_PANIC_ERROR            -> error-on-panic: report reaching a panic
//                                      as a verification error (panic-freedom).
//
// The verifier primitives are provided by SeaHorn and are already present in
// every job module.

void __VERIFIER_assume(int);
void __VERIFIER_error(void);

// Signature matches std's `rust_begin_unwind(&PanicInfo)`; noreturn like std's.
__attribute__((noreturn)) void rust_begin_unwind(void *info) {
#ifdef ABSTRACT_PANIC_ERROR
  __VERIFIER_error();
#else
  __VERIFIER_assume(0);
#endif
  __builtin_unreachable();
}
