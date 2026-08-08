# Abort-on-panic (AoP) twin of ../custom-vec-zst-alignment-issue. No panic token => std is built with
# panic_immediate_abort, so panics become llvm.trap/unreachable and SeaHorn
# prunes those paths. This keeps the original proof of the explicit assertions,
# which the EoP base job can no longer report as unsat.
c_rust_llvm(custom-vec-zst-alignment-issue-aop custom-vec-zst-alignment-issue-aop.c)

sea_add_unsat_test(custom-vec-zst-alignment-issue-aop)
