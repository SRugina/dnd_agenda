FROM gitpod/workspace-postgres

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y \
&& cargo install diesel