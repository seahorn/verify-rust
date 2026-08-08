# Error-on-panic (EoP) twin of ../vec-filter, which is abort-on-panic (AoP).
# CUSTOM_PANIC_NO_STD builds std without panic_immediate_abort so a panic
# reaches the #[panic_handler]; Cargo.toml turns that handler into
# verifier_error(). Together these make panics (overflow, bounds, alloc
# failure) verification errors instead of pruned paths.
c_rust_llvm(vec-filter-eop vec-filter-eop.c CUSTOM_PANIC_NO_STD)

# EoP turns the i32 overflow in `values.iter().filter(..).sum()` into a
# reachable verifier_error: x + y + z over three unconstrained i32 can overflow.
# The AoP twin proves `(result & 1) == 0` only over non-overflowing runs, so it
# reports unsat. Expect sat here until the harness bounds its inputs.
sea_add_sat_test(vec-filter-eop)
