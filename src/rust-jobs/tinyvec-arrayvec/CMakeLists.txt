c_rust_llvm(tinyvec-arrayvec tinyvec-arrayvec.c CUSTOM_PANIC_NO_STD)

configure_file(sea.yaml sea.yaml @ONLY)
sea_discover_tests(tinyvec-arrayvec)
