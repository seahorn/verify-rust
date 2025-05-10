cmake_minimum_required(VERSION 3.19)

project(C-Rust)

# ---------- Local CMake Modules ----------- #
set(CMAKE_MODULE_PATH ${CMAKE_SOURCE_DIR}/cmake)


# ---------- Install Corrosion ----------- #
include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.3.5 # fixed tag, update manually
)

# Our preferred toolchian for which fat lto works
set(Rust_TOOLCHAIN nightly-2022-08-01)

# Set any global configuration variables such as `Rust_TOOLCHAIN` before this line!
FetchContent_MakeAvailable(Corrosion)


include(CTest)
enable_testing()

include(SeaHornBuild)
find_program(AWK awk mawk gawk)
find_program(DEFAULT_LLVM_NM llvm-nm-14)
find_program(CPP_FILT c++filt)

set(SEAHORN_ROOT "" CACHE PATH "Path to SeaHorn installation")
set(SEA_LINK "llvm-link-14" CACHE STRING "Path to llvm-link")
set(SEA_OPT "${SEAHORN_ROOT}/bin/seaopt" CACHE STRING "Path to seaopt binary")
set(SEA_PP "${SEAHORN_ROOT}/bin/seapp" CACHE STRING "Path to seapp binary")

set(LLVM_OBJCOPY "llvm-objcopy-14" CACHE STRING "Path to llvm-objcopy")
set(LLVM_DIS "llvm-dis-14" CACHE STRING "Path to llvm-dis")
set(LLD "lld-14" CACHE STRING "Path to lld")
set(LLVM_NM ${DEFAULT_LLVM_NM} CACHE PATH "Path to llvm-nm")

set(LLVMIR_LINK ${SEA_LINK})
set(LLVMIR_OPT ${SEA_OPT})

set(CBINDGEN_TOML ${CMAKE_SOURCE_DIR}/cbindgen.toml)

configure_file(verify.py.in verify @ONLY)
set(VERIFY_CMD ${CMAKE_CURRENT_BINARY_DIR}/verify)

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/cmake/extract_tests.cmake.in extract_tests.cmake @ONLY)
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/cmake/extract_unsat_tests.cmake.in extract_unsat_tests.cmake @ONLY)

set(EXTRACT_TEST_CMD ${CMAKE_CURRENT_BINARY_DIR}/extract_tests.cmake)
set(EXTRACT_UNSAT_TEST_CMD ${CMAKE_CURRENT_BINARY_DIR}/extract_unsat_tests.cmake)

include_directories(BEFORE ${CMAKE_CURRENT_BINARY_DIR}/include)
include_directories(${SEAHORN_ROOT}/include)

add_subdirectory(src)

