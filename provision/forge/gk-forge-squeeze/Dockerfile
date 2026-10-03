# gk-forge-squeeze: builds the static GCC and binutils bundles for GCC 4.2 to 4.5 (spec 04.4).
#
# Those releases want a host compiler of their own time, and squeeze's GCC 4.4 builds all four. Squeeze has no zstd and its tar cannot sort members, so build.sh leaves an uncompressed tarball with a sorted member list and gk compresses it outside the container. As in the other forges, every source tarball comes in from the gk cache and the build never touches the network. apt reads archive.debian.org, and since squeeze's archive keys have expired it is told to install what it cannot authenticate.

FROM debian/eol:squeeze@sha256:80ed3d6f933f102b35efaa1a9a7513e5ed9937c5a93c289dd6e2e1db51d6f331

RUN sed -i -e '/^# deb /d' /etc/apt/sources.list \
 && apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Check-Valid-Until=false install -y --force-yes --no-install-recommends \
      gcc libc6-dev make \
      bison flex texinfo gawk m4 perl \
      xz-utils bzip2 gzip file patch diffutils \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C
WORKDIR /forge
