c_rust_llvm(custom-vec-drain custom-vec-drain.c)

configure_file(sea.yaml sea.yaml @ONLY)
sea_add_unsat_test(custom-vec-drain)
