# gk-host-jessie: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-buster: the distribution's gcc as HOSTCC, make and the other tools kbuild runs, and nothing of the target side. Jessie has no zstd, which no kernel of its era asks for. The base is pinned by digest and apt reads the snapshot.debian.org lines the debian/eol image ships commented out. The snapshot is years past its Valid-Until date, so apt is told not to check it.

FROM debian/eol:jessie@sha256:81cc60273fcd009665b917085e8edb4a44af01130e42a9349545bcaaf05236a2

RUN sed -i -e '/^deb /d' -e 's/^# deb /deb /' /etc/apt/sources.list \
 && apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Check-Valid-Until=false install -y --no-install-recommends \
      gcc libc6-dev make perl bison flex bc python3 \
      libelf-dev libssl-dev openssl cpio xz-utils gzip bzip2 kmod \
      rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /work
