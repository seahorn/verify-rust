# EoP (CUSTOM_PANIC_NO_STD + verifier/panic_error): a panic, including core's UB
# precondition checks, is a verification error, not a pruned path. The reachable
# panic EoP surfaced here is core's ptr::read alignment check on the original ZST
# iterator (testfail_alignment_bug); test_alignment_fixed applies std's fix.
c_rust_llvm(custom-vec-zst-alignment-issue custom-vec-zst-alignment-issue.c CUSTOM_PANIC_NO_STD)

# test_*     -> unsat (fixed ZST iterator)
# testfail_* -> sat   (original ZST iterator, misaligned read)
sea_discover_tests(custom-vec-zst-alignment-issue)
