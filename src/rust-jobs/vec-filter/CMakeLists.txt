c_rust_llvm(vec-filter vec-filter.c CUSTOM_PANIC_NO_STD)

# EoP (CUSTOM_PANIC_NO_STD + verifier/panic_error) makes panics verification
# errors instead of pruned paths, which surfaces a reachable panic here that the
# previous abort-on-panic build assumed away. Expect sat until the harness is
# fixed or the panic is shown unreachable.
sea_add_sat_test(vec-filter)
