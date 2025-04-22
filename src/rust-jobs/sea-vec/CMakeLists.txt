c_rust_llvm(sea-vec sea-vec.c CUSTOM_PANIC_NO_STD)
configure_file(sea.yaml sea.yaml @ONLY)
sea_discover_tests(sea-vec)
