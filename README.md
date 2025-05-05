# verify-rust

![os](https://img.shields.io/badge/os-linux-orange?logo=linux)
[![CI](https://github.com/seahorn/verify-rust/actions/workflows/main.yml/badge.svg)](https://github.com/seahorn/verify-rust/actions?query=workflow%3ACI)

Building this project will first require installations of Rust, cbindgen and LLVM 14

## Installing Rust with LLVM 14

To start, install rustc and rustup from the [rust web site](https://www.rust-lang.org/tools/install)

Next step is to set the rust version to one that used LLVM 14. It was first added in version 1.60 and last used in version 1.64.0. To do this for a version such as 1.64.0, use the following command:

```bash
rustup install <version>
rustup default <version>
```

To check that the update worked, run the following command to view the rustc version being used along with the version of LLVM being used:

```bash
rustc --version --verbose
```

The rust library contained in this project contains a `rust-toolchain.toml` to set the version to that needed for compatability with LLVM 14 and the required rust compiler flags. If building the project with additional crates, ensure to add a `rust-toolchain.toml` file in the crate directory setting the default toolchain channel to `nightly-2022-08-01`.

## Installing cbindgen

Using cargo, install cbindgen and add it to your path:

```bash
cargo install --force cbindgen
export PATH=$PATH:~/.cargo/bin
```

## Installing LLVM 14 Tools

First ensure that clang and lld are installed and based on LLVM version 14

```bash
sudo apt install clang-14 lld-14
```

To check installation, run

```bash
clang-14 --version
ld.lld-14 --version
```

## Building

As a prerequisite, follow [this guide](https://github.com/seahorn/seahorn/tree/main#developers-zone) to build SeaHorn locally.

1. Create a build directory

    ```bash
    mkdir build
    cd build
    ```

2. Configure with cmake

    ```bash
    cmake \
     -DCMAKE_C_COMPILER=clang-14 \
     -DCMAKE_CXX_COMPILER=clang++-14 \
     -DSEAHORN_ROOT=<SEAHORN_ROOT> \
     ../ -GNinja
    ```

    Where `SEAHORN_ROOT` is the directory containing your local seahorn build.

    Alternatively, the project can be configured using cmake presets. To do this, simply run the following command:

    ```bash
    cmake --preset <PRESET_NAME>
    ```

    Where `PRESET_NAME` is the preset you would like to use. The presets that are currently available are: `default-jammy`. These presets assume that `seahorn` is in your home directory.

    This will also allow the project to be configured and compiled within VS Code using the CMake Tools extension.

    If you would like to use different compilation settings or if you have `seahorn` in any other directory, you will need to make your own `CMakeUserPresets.json` file with your own presets.

3. Compile

    ```bash
    ninja
    ```

    or

    ```bash
    cmake --build .
    ```

## Running on Windows with Docker

In order to use the project on Windows, a Docker container is used to run the project. To set this up, do the following:

1. Install and setup the [Windows Subsystem for Linux (WSL)](https://docs.microsoft.com/en-us/windows/wsl/install-win10)
2. Install [Docker Desktop for Windows](https://docs.docker.com/docker-for-windows/install/)
3. Install the [Dev - Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension for VS Code
4. From a command prompt, enter WSL using:

    ```cmd
    wsl
    ```

5. Configure git inside of WSL

    ```bash
    git config --global user.name "Your Name"
    git config --global user.email "Your Email"
    ```

6. Setup an SSH key for GitHub by following [this guide](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/adding-a-new-ssh-key-to-your-github-account?platform=linux)

7. Clone the project into your home directory

    ```bash
    cd ~
    git clone git@github.com:thomashart17/c-rust.git
    ```

8. Build the Docker container

    From the root of the project, run:

    ```bash
    docker pull seahorn/seahorn-llvm14:nightly
    docker build -t c-rust -f docker/c-rust.Dockerfile .
    ```

9. Run the Docker container

    ```bash
    docker run -it c-rust
    ```

    After completing this step, you will be able to stop and start the docker container from the Docker Desktop application. Running this again will create a new container that will not contain any of the changes made in the previous container.

10. Open the project in VS Code

    Once the container is running, open VS Code and select the `Dev Containers: Attach to Running Container...` command from the command palette. Select the container that was just created and wait for the project to open.

## Custom Print Macros

To avoid increased runtime for jobs using print macros, custom print macros can overwrite the standard print macros and remove their functionality. These macros are defined in `sea` and can be imported as follows:

```rust
use sea::print;
use sea::println;
use sea::eprint;
use sea::eprintln;
```

## Build Process

### Generating C Headers From Rust Crate

Use CBindGen to generate C header files for Rust library. First install CBindGen

```bash
cargo install -force cbindgen
export PATH=$PATH:~/.cargo/bin
```

Next create a cbindgen.toml file in the crate's directory. For the test lib in this project,

```bash
cd /project-path/src/test-lib
echo -e "language = \"C\"\n\ninclude_guard = \"TEST_LIB_H_\"" > cbindgen.toml
```

Once configured, inside the same directory, run the cbindgen command

```bash
cbindgen --config cbindgen.toml --output ./inc/lib.h
```

This will generate a header file `lib.h` inside the crate's folder inside of a new `inc` folder.

### LTO

Link Time Optimization was based off the resource available [here](https://blog.llvm.org/2019/09/closing-gap-cross-language-lto-between.html).

Before running LTO between rust and C, clang and lld must be installed as described in **Installing LLVM 14 Tools**
Now to build the lto files

When building for LTO, certain flags are set for the rust compiler `rustc`, the LLVM C frontend and the LLVM linker.

#### Rustc flags

The flags needed for LTO with the rustc compiler are set in the `Cargo.toml` file and will be implemented by the cargo tool automatically. They are described here.

- `crate-type = static-lib` - Will generate a static system library as the output of crate compilation.
- `opt-level = 2` - Sets optimizer level 2
- `Clinker-plugin-lto` - Defers LTO optimization to the final linking stage. Passed as a profile `RUSTFLAG`.
- `Zemit-thin-lto=no` - Experimental flag to running thin LTO pass when using the LTO linker plugin. Passed as a profile `RUSTFLAG`.

#### C flags

The flags needed for LTO with the clang compiler are passed as compiler arguments. They are described here.

- `flto` - Defers LTO optimization to the final linking stage
- `O2` - Sets optimizer level 2

#### Compiler flags

The flags needed for LTO with the lld linker are passed as arguments. They are described here

- `flto` - Enables LTO
- `fuse-ld=lld-14` - Enables using the LLVM linker LLD of version 14
- `Wl,--plugin-opt=-lto-embed-bitcode=post-merge-pre-opt` - Embeds the post merge, pre optimized bitcode to the resultant binary file
- `O2` - Set optimizer level 2

### Outputting LLVM

Once the binary is generated, it is necessary to extract the bitcode from the file. This is done using the `llvm-objcopy-14` and `llvm-dis-14` tool

```bash
objcopy a.out --dump-section .llvmbc=llvm.bc
llvm-dis-14 llvm.bc
```
