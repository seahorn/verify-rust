c_rust_llvm(option-vec-map option-vec-map.c)
configure_file(sea.yaml sea.yaml @ONLY)
# Do not run in vacuity because of
# https://github.com/seahorn/seahorn/issues/572
sea_add_unsat_test_novac(option-vec-map)
