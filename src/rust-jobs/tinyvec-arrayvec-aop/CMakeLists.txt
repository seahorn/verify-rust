# Abort on panic (no error)
c_rust_llvm(tinyvec-arrayvec-aop tinyvec-arrayvec-aop.c)

configure_file(sea.yaml sea.yaml @ONLY)
sea_discover_unsat_tests(tinyvec-arrayvec-aop)
