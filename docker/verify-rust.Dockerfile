FROM seahorn/seahorn-llvm14:nightly

ENV SEAHORN=/home/usea/seahorn/bin/sea PATH="$PATH:/home/usea/seahorn/bin:/home/usea/bin:/home/usea/.cargo/bin"

## install required pacakges
USER root

## Install latest cmake
RUN apt -y remove --purge cmake
RUN apt -y update
RUN apt -y install wget python3-pip
RUN python3 -m pip install --upgrade pip
RUN apt -y install emacs

## Install rust
USER usea
WORKDIR /home/usea

# install rust 
RUN bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none"
RUN rustup install nightly-2022-08-01
RUN rustup +nightly-2022-08-01 component add rust-src
RUN cargo +nightly-2022-08-01 install --locked --force cbindgen --version ^0.26
RUN cargo install --locked kani-verifier --version ^0.43 ## latest version to work with Rust 1.64

# install cmake and point path to it
RUN wget https://github.com/Kitware/CMake/releases/download/v3.31.7/cmake-3.31.7-linux-x86_64.tar.gz
RUN tar -xzf cmake-3.31.7-linux-x86_64.tar.gz
ENV PATH="/home/usea/cmake-3.31.7-linux-x86_64/bin:$PATH"

# install reframe hpc test framework v4.8.0 and set path
RUN pip install ReFrame-HPC==4.8.0
ENV PATH="/home/usea/.local/bin:$PATH"

## import verify-rust
USER usea
WORKDIR /home/usea
#
# assume we are run inside verify-rust 
# copy the contents of the current directory to /home/usea/verify-rust
RUN mkdir verify-rust 
COPY --chown=usea:usea . verify-rust

# Run the reframe tests from outside the root dir to help with staging
RUN mkdir -p /tmp/benchmarks 
WORKDIR /tmp/benchmarks

### set default user and wait for someone to login and start running verification tasks
USER usea
