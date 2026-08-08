c_rust_llvm(smallvec-insert_many-fix smallvec-insert_many-fix.c ABSTRACT_PANIC_ERROR)

configure_file(sea.yaml sea.yaml @ONLY)
sea_add_unsat_test(smallvec-insert_many-fix)
