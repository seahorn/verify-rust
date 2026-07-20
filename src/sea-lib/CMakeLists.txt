add_library(sea-lib.LIB STATIC seahorn.c)
target_compile_definitions(sea-lib.LIB PRIVATE VACCHECK)
target_compile_options(sea-lib.LIB PUBLIC
	-flto
        -O1
        -Xclang -disable-llvm-optzns
        --target=x86_64-unknown-linux-gnu
)

# Panic-abstraction override stubs (bitcode), llvm-link --override'd into a
# job's module by c_rust_llvm when ABSTRACT_PANIC(_ERROR) is requested.
# Two variants: abort-on-panic (assume false) and error-on-panic (verifier error).
set(PANIC_ABSTRACT_SRC ${CMAKE_CURRENT_SOURCE_DIR}/panic_abstract.c)
set(PANIC_ABSTRACT_ABORT_BC ${CMAKE_CURRENT_BINARY_DIR}/panic_abstract_abort.bc
    CACHE INTERNAL "abort-on-panic override stub bitcode")
set(PANIC_ABSTRACT_ERROR_BC ${CMAKE_CURRENT_BINARY_DIR}/panic_abstract_error.bc
    CACHE INTERNAL "error-on-panic override stub bitcode")

add_custom_command(
    OUTPUT ${PANIC_ABSTRACT_ABORT_BC}
    COMMAND ${CMAKE_C_COMPILER} --target=x86_64-unknown-linux-gnu -flto -c
            ${PANIC_ABSTRACT_SRC} -o ${PANIC_ABSTRACT_ABORT_BC}
    DEPENDS ${PANIC_ABSTRACT_SRC}
    COMMENT "Building abort-on-panic override stub bitcode"
)
add_custom_command(
    OUTPUT ${PANIC_ABSTRACT_ERROR_BC}
    COMMAND ${CMAKE_C_COMPILER} --target=x86_64-unknown-linux-gnu -flto -c
            -DABSTRACT_PANIC_ERROR ${PANIC_ABSTRACT_SRC} -o ${PANIC_ABSTRACT_ERROR_BC}
    DEPENDS ${PANIC_ABSTRACT_SRC}
    COMMENT "Building error-on-panic override stub bitcode"
)
add_custom_target(panic-abstract-stubs ALL
    DEPENDS ${PANIC_ABSTRACT_ABORT_BC} ${PANIC_ABSTRACT_ERROR_BC})
