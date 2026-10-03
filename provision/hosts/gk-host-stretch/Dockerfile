# gk-host-stretch: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-buster: the distribution's gcc as HOSTCC, make and the other tools kbuild runs, and nothing of the target side. Only the base differs. The base is pinned by digest and apt reads the snapshot.debian.org lines the debian/eol image ships commented out. The snapshot is years past its Valid-Until date, so apt is told not to check it.

FROM debian/eol:stretch@sha256:87de8b3de9239bd73f9d73bd3816411e156705c7517dd0177b50136e97cdb43f

RUN sed -i -e '/^deb /d' -e 's/^# deb /deb /' /etc/apt/sources.list \
 && apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Check-Valid-Until=false install -y --no-install-recommends \
      gcc libc6-dev make perl bison flex bc python3 \
      libelf-dev libssl-dev openssl cpio xz-utils zstd gzip bzip2 kmod \
      rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /work
