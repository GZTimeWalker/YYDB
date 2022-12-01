#!/bin/sh

set -eux;

apt update && apt install -y --no-install-recommends gnupg dirmngr dpkg-dev

# gpg: key 3A79BD29: public key "MySQL Release Engineering <mysql-build@oss.oracle.com>" imported
key='859BE8D7C586F538430B19C2467B942D3A79BD29'
export GNUPGHOME="$(mktemp -d)"
gpg --batch --keyserver keyserver.ubuntu.com --recv-keys "$key"
mkdir -p /etc/apt/keyrings
gpg --batch --export "$key" > /etc/apt/keyrings/mysql.gpg
gpgconf --kill all
rm -rf "$GNUPGHOME"

echo 'deb [ signed-by=/etc/apt/keyrings/mysql.gpg ] https://mirrors.ustc.edu.cn/mysql-repo/apt/debian/ bullseye mysql-8.0' > /etc/apt/sources.list.d/mysql.list
echo 'deb-src [ signed-by=/etc/apt/keyrings/mysql.gpg ] https://mirrors.ustc.edu.cn/mysql-repo/apt/debian/ bullseye mysql-8.0' >> /etc/apt/sources.list.d/mysql.list

cd /usr/local/src

apt update && apt source mysql-community
rm *.tar.* *.dsc &&
mv mysql-* mysql-8.0

export MYSQL_SOURCE_DIR=/usr/local/src/mysql-8.0
chown $USER:$USER -R $MYSQL_SOURCE_DIR
