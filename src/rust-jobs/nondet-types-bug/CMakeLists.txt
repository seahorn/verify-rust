c_rust_llvm(nondet-types-bug nondet-types-bug.c CUSTOM_PANIC_NO_STD)

configure_file(sea.yaml sea.yaml @ONLY)

# the assertion fails only if every generator can reach a specific value
sea_add_sat_test(nondet-types-bug)
