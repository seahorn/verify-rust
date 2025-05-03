c_rust_llvm(custom-vec custom-vec.c)

configure_file(sea.yaml sea.yaml @ONLY)
sea_discover_tests(custom-vec)
