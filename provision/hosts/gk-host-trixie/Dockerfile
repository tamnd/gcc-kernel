# gk-host-trixie: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# It has the distribution's gcc as HOSTCC, make and the other tools kbuild runs, and nothing of the target side. The compiler under test is a static bundle mounted at /opt/gk/t and named by CROSS_COMPILE. The base is pinned by digest and apt reads a snapshot.debian.org date, so a rebuild gets the same packages.

FROM debian:trixie@sha256:9cc080028c43b27d2074d63a5f9caf7166d731494965616c1a6d2827a004585c

ARG SNAPSHOT=20260930T000000Z

RUN rm -f /etc/apt/sources.list.d/debian.sources \
 && echo "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/${SNAPSHOT} trixie main" > /etc/apt/sources.list \
 && echo "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian-security/${SNAPSHOT} trixie-security main" >> /etc/apt/sources.list \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc libc6-dev make perl bison flex bc python3 \
      libelf-dev libssl-dev openssl cpio xz-utils zstd gzip bzip2 kmod \
      rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /work
