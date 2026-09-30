# gk-forge-modern: builds the static GCC and binutils bundles for GCC 9.5 to 16.2 (spec 04.4).
#
# It holds only what a GCC and binutils build needs. The source tarballs, GMP, MPFR, MPC and ISL included, come in from the gk cache already checked, so the build itself never touches the network. Static linking needs libc.a and libstdc++.a, which libc6-dev and libstdc++-14-dev carry.

FROM debian:trixie@sha256:9cc080028c43b27d2074d63a5f9caf7166d731494965616c1a6d2827a004585c

ARG SNAPSHOT=20260930T000000Z

RUN rm -f /etc/apt/sources.list.d/debian.sources \
 && echo "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/${SNAPSHOT} trixie main" > /etc/apt/sources.list \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc g++ libc6-dev libstdc++-14-dev make \
      bison flex texinfo gawk m4 perl python3 \
      xz-utils bzip2 gzip zstd file patch diffutils \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /forge
