# gk-host-buster: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# It has the distribution's gcc as HOSTCC, make and the other tools kbuild runs, and nothing of the target side. The compiler under test is a static bundle mounted at /opt/gk/t and named by CROSS_COMPILE. The base is pinned by digest and apt reads the snapshot.debian.org lines the debian/eol image ships commented out, which name a fixed date, so a rebuild gets the same packages. The live lines point at archive.debian.org, which moves, and are dropped.

FROM debian/eol:buster@sha256:3207f48cbea2d8c398c1ecd43ba351c8cad5b2a3676cd365f5e144d51ff39650

RUN sed -i -e '/^deb /d' -e 's/^# deb /deb [check-valid-until=no] /' /etc/apt/sources.list \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc libc6-dev make perl bison flex bc python3 \
      libelf-dev libssl-dev openssl cpio xz-utils zstd gzip bzip2 kmod \
      rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /work
