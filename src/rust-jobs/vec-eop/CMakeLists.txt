# Error-on-panic (EoP) twin of ../vec, which is abort-on-panic (AoP).
# CUSTOM_PANIC_NO_STD builds std without panic_immediate_abort so a panic
# reaches the #[panic_handler]; Cargo.toml turns that handler into
# verifier_error(). Together these make panics (overflow, bounds, alloc
# failure) verification errors instead of pruned paths.
c_rust_llvm(vec-eop vec-eop.c CUSTOM_PANIC_NO_STD)

sea_add_unsat_test(vec-eop)
