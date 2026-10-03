# gk-forge-woody: builds the static GCC and binutils bundles for GCC 2.95.3, 3.0 and 3.1 (spec 04.4).
#
# Woody is i386 only, and its GCC is 2.95.4. The bundles it makes are i386 programs, which run in every host, 32 or 64 bit. Woody's bash has no pipefail and its tar can neither guess a compression nor set member times, so build.sh unpacks with an explicit decompressor and leaves the installed tree for gk to pack outside the container. Every source tarball comes in from the gk cache and the build never touches the network. apt reads archive.debian.org, and is told to install what it cannot authenticate. Woody's apt is older than the Valid-Until check that the newer forges turn off, and its sed has no -i.

FROM --platform=linux/386 debian/eol:woody@sha256:a196c9897d10409c0616491ed0e5f1282d9fa708fddd53422c7b554f9003c528

RUN grep -v '^# deb ' /etc/apt/sources.list > /tmp/sources.list && mv /tmp/sources.list /etc/apt/sources.list \
 && apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --force-yes \
      gcc libc6-dev make \
      bison flex texinfo gawk m4 perl \
      bzip2 gzip file patch diff \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C
WORKDIR /forge
