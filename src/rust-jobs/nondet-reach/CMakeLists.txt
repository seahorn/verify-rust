c_rust_llvm(nondet-reach nondet-reach.c CUSTOM_PANIC_NO_STD)

configure_file(sea.yaml sea.yaml @ONLY)

# one verification run per test_* (unsat) / testfail_* (sat) function
sea_discover_tests(nondet-reach)
