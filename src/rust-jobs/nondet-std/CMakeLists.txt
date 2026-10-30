c_rust_llvm(nondet-std nondet-std.c)

configure_file(sea.yaml sea.yaml @ONLY)

# one verification run per test_* (unsat) / testfail_* (sat) function
sea_discover_tests(nondet-std)
