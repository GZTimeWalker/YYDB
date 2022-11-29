FROM docker/dev-environments-default:stable-1

RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list && \
    sed -i 's/http:/https:/g' /etc/apt/sources.list

RUN apt update && apt upgrade -y && apt install -y \
    build-essential nano wget curl git vim cmake bat \
    zsh-syntax-highlighting zsh-autosuggestions

# set up rust env
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.65.0

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='5cc9ffd1026e82e7fb2eec2121ad71f4b0f044e88bca39207b3f6b769aaa799c' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='48c5ecfd1409da93164af20cf4ac2c6f00688b15eb6ba65047f654060c844d85' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='e189948e396d47254103a49c987e7fb0e5dd8e34b200aa4481ecc4b8e41fb929' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='0e0be29c560ad958ba52fcf06b3ea04435cb3cd674fbe11ce7d954093b9504fd' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.25.1/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup component add rustfmt clippy; \
    rustup --version; \
    cargo --version; \
    rustc --version;

COPY .devcontainer/config.toml $CARGO_HOME/config.toml

COPY .devcontainer/.zshrc /root/.zshrc

RUN useradd -m -s /bin/zsh -G sudo -u 1024 -U dev && \
    echo "dev ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers && \
    chsh -s /bin/zsh root && \
    cp /root/.zshrc /home/dev/.zshrc && \
    mkdir /Y-Engine && ln -s /Y-Engine /com.docker.devenvironments.code

USER dev

CMD ["/bin/zsh"]
