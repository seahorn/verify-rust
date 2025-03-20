FROM seahorn/seahorn-llvm14:nightly

ENV SEAHORN=/home/usea/seahorn/bin/sea PATH="$PATH:/home/usea/seahorn/bin:/home/usea/bin:/home/usea/.cargo/bin"

## install required pacakges
USER root

## Install latest cmake
RUN apt -y remove --purge cmake
RUN apt -y update
RUN apt -y install wget python3-pip
RUN python3 -m pip install --upgrade pip
RUN pip3 install cmake --upgrade
RUN apt -y install emacs

## Install rust
USER usea
WORKDIR /home/usea

RUN bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none"
RUN rustup install nightly-2022-08-01
RUN rustup +nightly-2022-08-01 component add rust-src
RUN cargo +nightly-2022-08-01 install --locked --force cbindgen --version ^0.26
RUN cargo install --locked kani-verifier --version ^0.43 ## latest version to work with Rust 1.64

## import c-rust
USER usea
WORKDIR /home/usea
#
## assume we are run inside c-rust 
RUN mkdir c-rust 
COPY --chown=usea:usea . c-rust

#
WORKDIR /home/usea/c-rust
#
RUN rm -Rf build && mkdir build && cd build && cmake -DCMAKE_C_COMPILER=clang-14 -DCMAKE_CXX_COMPILER=clang++-14 -DSEAHORN_ROOT=/home/usea/seahorn -DRust_COMPILER=$(rustup which rustc) -DRust_CARGO=$(rustup which cargo)  ../ -GNinja && cmake --build .

#
### set default user and wait for someone to login and start running verification tasks
USER usea
