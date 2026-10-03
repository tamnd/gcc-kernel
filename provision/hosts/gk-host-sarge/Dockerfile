# gk-host-sarge: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-squeeze: the distribution's gcc as HOSTCC, make and the other tools kbuild runs, and nothing of the target side. debian/eol has sarge for i386 only. The bundles are static amd64 programs and run in it all the same, and kbuild only builds its host tools with this gcc. The base is pinned by digest and apt reads archive.debian.org. Sarge's apt predates signed archives, so there is nothing to authenticate. Sarge has no timeout command, so a shell stand-in takes its place.

FROM debian/eol:sarge@sha256:9bcda9a444c06d72061b7a71ec454034551d043abc2af2bfa5be583455865638

RUN sed -i -e '/^# deb /d' /etc/apt/sources.list \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --force-yes \
      gcc libc6-dev make perl cpio gzip bzip2 module-init-tools file \
 && rm -rf /var/lib/apt/lists/*

COPY timeout /usr/local/bin/timeout
ENV LANG=C
WORKDIR /work
