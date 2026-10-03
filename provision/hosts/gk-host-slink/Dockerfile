# gk-host-slink: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-woody, for the 2.2 era: the distribution's gcc as HOSTCC, make, bin86 and modutils. Slink's dpkg cannot run maintainer scripts under docker, where it fails to ignore SIGQUIT, so apt only downloads the packages and each one is unpacked with dpkg-deb. None of these packages needs its scripts to work. The base is pinned by digest and apt reads archive.debian.org, which slink's apt cannot authenticate. Slink has no timeout command, so the shell stand-in of sarge takes its place.

FROM --platform=linux/386 debian/eol:slink@sha256:8a6513d46960af71f6e534b294a31fdd09522220f4ee4df1629720d2c1d0d529

RUN grep -v '^# deb ' /etc/apt/sources.list > /tmp/sources.list && mv /tmp/sources.list /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -d -y gcc libc6-dev make perl bin86 modutils file gzip bzip2 \
 && for deb in /var/cache/apt/archives/*.deb; do dpkg-deb -x "$deb" / || exit 1; done \
 && rm -rf /var/cache/apt/archives/*.deb /var/lib/apt/lists/*

COPY timeout /usr/local/bin/timeout
ENV LANG=C
WORKDIR /work
