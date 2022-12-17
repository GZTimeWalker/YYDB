FROM docker/dev-environments-default:stable-1

RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list && \
    sed -i 's/http:/https:/g' /etc/apt/sources.list

RUN apt update && apt upgrade -y && apt install --no-install-recommends -y \
    build-essential nano wget curl git vim cmake bat \
    zsh-syntax-highlighting zsh-autosuggestions fd-find \
    bzip2 openssl perl xz-utils zstd \
    libssl-dev zlib1g-dev libncurses-dev pkg-config libudev-dev

# set up rust env
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN set -eux; \
    url="https://sh.rustup.rs"; \
    curl "$url" | sh -s -- -y --no-modify-path --profile minimal --default-toolchain nightly \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup component add rustfmt clippy; \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN groupadd -r mysql && useradd -r -g mysql mysql

RUN apt-get update && apt-get install -y --no-install-recommends gnupg dirmngr

RUN set -eux; \
# gpg: key 3A79BD29: public key "MySQL Release Engineering <mysql-build@oss.oracle.com>" imported
    key='859BE8D7C586F538430B19C2467B942D3A79BD29'; \
    export GNUPGHOME="$(mktemp -d)"; \
    gpg --batch --keyserver keyserver.ubuntu.com --recv-keys "$key"; \
    mkdir -p /etc/apt/keyrings; \
    gpg --batch --export "$key" > /etc/apt/keyrings/mysql.gpg; \
    gpgconf --kill all; \
    rm -rf "$GNUPGHOME"

RUN echo 'deb [ signed-by=/etc/apt/keyrings/mysql.gpg ] http://mirrors.ustc.edu.cn/mysql-repo/apt/debian/ bullseye mysql-8.0' > /etc/apt/sources.list.d/mysql.list && \
    echo 'deb-src [ signed-by=/etc/apt/keyrings/mysql.gpg ] http://mirrors.ustc.edu.cn/mysql-repo/apt/debian/ bullseye mysql-8.0' >> /etc/apt/sources.list.d/mysql.list

WORKDIR /
COPY .devcontainer/config.toml $CARGO_HOME/config.toml
COPY .devcontainer/.zshrc /root/.zshrc

RUN useradd -m -s /bin/zsh -G sudo -u 1024 -U dev && \
    echo "dev ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers && \
    chsh -s /bin/zsh root && \
    cp /root/.zshrc /home/dev/.zshrc && \
    mkdir /YYDB

WORKDIR /usr/local/src
RUN apt update && apt source mysql-community && \
    rm *.tar.* *.dsc && mv mysql-* mysql-8.0 && \
    chown -R dev:dev mysql-8.0 && \
    mkdir -p /usr/local/mysql

ENV MYSQL_SOURCE_DIR=/usr/local/src/mysql-8.0

USER dev

CMD ["/bin/zsh"]
