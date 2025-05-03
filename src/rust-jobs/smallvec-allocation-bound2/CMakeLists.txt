c_rust_llvm(smallvec-allocation-bound2 smallvec-allocation-bound2.c CUSTOM_PANIC_NO_STD)

configure_file(sea.yaml sea.yaml @ONLY)
sea_discover_tests(smallvec-allocation-bound2)
