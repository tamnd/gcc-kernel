# gk-host-eoan: the image of the Ubuntu 19.10 distribution column, whose gcc-9 builds with -fcf-protection and -fstack-clash-protection by default (spec 04.3).
#
# The same recipe as the Debian hosts. The base is pinned by digest. Eoan's archive moved to old-releases.ubuntu.com when it reached its end of life, so apt is pointed there.

FROM ubuntu:19.10@sha256:f332c4057e21ec71cc8b20b05328d476104a069bfa6882877e0920e8140edcf0

RUN sed -i -e 's|http://archive.ubuntu.com|http://old-releases.ubuntu.com|' -e 's|http://security.ubuntu.com|http://old-releases.ubuntu.com|' /etc/apt/sources.list \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc libc6-dev make perl bison flex bc python3 \
      libelf-dev libssl-dev openssl cpio xz-utils zstd gzip bzip2 kmod \
      rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /work
