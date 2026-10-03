# gk-host-woody: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-sarge, for the 2.4 era. The distribution's gcc is HOSTCC, bin86 assembles the real mode boot code of 2.0 and before, and modutils gives `make modules_install` the depmod it runs. debian/eol has woody for i386 only, and the bundles are static amd64 programs that run in it all the same. The base is pinned by digest. Woody's apt predates signed archives and its sources.list already points at archive.debian.org. Its apt rejects the Check-Valid-Until option, so it is left out, and its sed has no -i, so the commented lines are dropped with grep. Woody has no timeout command, so the shell stand-in of sarge takes its place.

FROM --platform=linux/386 debian/eol:woody@sha256:a196c9897d10409c0616491ed0e5f1282d9fa708fddd53422c7b554f9003c528

RUN grep -v '^# deb ' /etc/apt/sources.list > /tmp/sources.list && mv /tmp/sources.list /etc/apt/sources.list \
 && apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --force-yes \
      gcc libc6-dev make perl bin86 modutils file gzip bzip2 \
 && rm -rf /var/lib/apt/lists/*

COPY timeout /usr/local/bin/timeout
ENV LANG=C
WORKDIR /work
