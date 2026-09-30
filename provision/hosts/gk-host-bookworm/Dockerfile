# gk-host-bookworm: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# It has the distribution's gcc as HOSTCC, make and the other tools kbuild runs, and nothing of the target side. The compiler under test is a static bundle mounted at /opt/gk/t and named by CROSS_COMPILE. The base is pinned by digest and apt reads a snapshot.debian.org date, so a rebuild gets the same packages.

FROM debian:bookworm@sha256:f37a335e82bca302e955fa39f9dfe28f1be618f016f8a2b56318e5a5111afc26

ARG SNAPSHOT=20260930T000000Z

RUN rm -f /etc/apt/sources.list.d/debian.sources \
 && echo "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/${SNAPSHOT} bookworm main" > /etc/apt/sources.list \
 && echo "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian-security/${SNAPSHOT} bookworm-security main" >> /etc/apt/sources.list \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc libc6-dev make perl bison flex bc python3 \
      libelf-dev libssl-dev openssl cpio xz-utils zstd gzip bzip2 kmod \
      rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /work
