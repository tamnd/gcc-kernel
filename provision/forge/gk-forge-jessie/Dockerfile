# gk-forge-jessie: builds the static GCC and binutils bundles for GCC 4.6 to 4.9 (spec 04.4).
#
# Those releases want a host compiler of their own time, and jessie's GCC 4.9 is the newest that builds all four. Jessie has no zstd and its tar cannot sort members, so build.sh leaves an uncompressed tarball with a sorted member list and gk compresses it outside the container. As in the other forges, every source tarball comes in from the gk cache and the build never touches the network.

FROM debian/eol:jessie@sha256:81cc60273fcd009665b917085e8edb4a44af01130e42a9349545bcaaf05236a2

# apt reads the snapshot.debian.org lines the image ships commented out. The snapshot is years past its Valid-Until date, so apt is told not to check it.
RUN sed -i -e '/^deb /d' -e 's/^# deb /deb /' /etc/apt/sources.list \
 && apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Check-Valid-Until=false install -y --no-install-recommends \
      gcc g++ libc6-dev make \
      bison flex texinfo gawk m4 perl \
      xz-utils bzip2 gzip file patch diffutils \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /forge
