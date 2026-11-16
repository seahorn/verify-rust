c_rust_llvm(borsh borsh.c ABSTRACT_PANIC)

configure_file(sea.yaml sea.yaml @ONLY)
sea_discover_tests(borsh)
