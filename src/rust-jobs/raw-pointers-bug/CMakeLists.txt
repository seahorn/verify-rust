c_rust_llvm(raw-pointers-bug raw-pointers-bug.c CUSTOM_PANIC_NO_STD)

# assertion ignores aliasing; see lib.rs
sea_add_sat_test(raw-pointers-bug)
