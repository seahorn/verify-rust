c_rust_llvm(pointer_same_arg pointer_same_arg.c CUSTOM_PANIC_NO_STD)

# see source for why this is expected to fail
sea_add_sat_test(pointer_same_arg)
