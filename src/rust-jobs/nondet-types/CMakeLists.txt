c_rust_llvm(nondet-types nondet-types.c CUSTOM_PANIC_NO_STD)

configure_file(sea.yaml sea.yaml @ONLY)

sea_add_unsat_test(nondet-types)
