# gk-forge-slink: builds the static GCC and binutils bundles for GCC 2.7.2.3 (spec 04.4).
#
# Slink is i386 only, and its GCC is 2.7.2.3 itself. Its dpkg cannot run maintainer scripts under docker, where it fails to ignore SIGQUIT, so apt only downloads the packages and each one is unpacked with dpkg-deb. Slink has no getconf, so build.sh reads the word size from the ELF header of /bin/sh. Every source tarball comes in from the gk cache and the build never touches the network. apt reads archive.debian.org, which slink's apt cannot authenticate.

FROM --platform=linux/386 debian/eol:slink@sha256:8a6513d46960af71f6e534b294a31fdd09522220f4ee4df1629720d2c1d0d529

RUN grep -v '^# deb ' /etc/apt/sources.list > /tmp/sources.list && mv /tmp/sources.list /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -d -y gcc libc6-dev binutils make bison flex perl file gzip bzip2 patch \
 && for deb in /var/cache/apt/archives/*.deb; do dpkg-deb -x "$deb" / || exit 1; done \
 && rm -rf /var/cache/apt/archives/*.deb /var/lib/apt/lists/*

ENV LANG=C
WORKDIR /forge
