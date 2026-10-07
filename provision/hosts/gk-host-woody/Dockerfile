# syntax=docker/dockerfile:1.7

# gk-host-woody: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-sarge, for the 2.4 era. The distribution's gcc is HOSTCC, bin86 assembles the real mode boot code of 2.0 and before, and modutils gives `make modules_install` the depmod it runs. debian/eol has woody for i386 only, and the bundles are static amd64 programs that run in it all the same. The base is pinned by digest. Woody's apt predates signed archives and its sources.list already points at archive.debian.org. Its apt rejects the Check-Valid-Until option, so it is left out, and its sed has no -i, so the commented lines are dropped with grep. Woody has no timeout command, so the shell stand-in of sarge takes its place.
#
# Woody's modutils is 2.4.15, whose `depmod -F System.map` only takes a `__kstrtab_` symbol that nm typed `?`. binutils 2.12 typed the .kstrtab section that way, but the binutils that come with GCC 3.4 and later type it `R`, so depmod found no kernel symbols and failed every module with unresolved symbols. modutils 2.4.17 fixed that, so depmod is built from 2.4.27, the last modutils release, and replaces woody's. The rest of woody's modutils stays. It is built for i386 only. A depmod that also handles x86_64 modules picks one by `uname -m` when it runs, which in this container says x86_64, and then it refuses i386 modules.

FROM --platform=linux/386 debian/eol:woody@sha256:a196c9897d10409c0616491ed0e5f1282d9fa708fddd53422c7b554f9003c528

RUN grep -v '^# deb ' /etc/apt/sources.list > /tmp/sources.list && mv /tmp/sources.list /etc/apt/sources.list \
 && apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --force-yes \
      gcc libc6-dev make perl bin86 modutils file gzip bzip2 \
 && rm -rf /var/lib/apt/lists/*

ADD --checksum=sha256:c3e34860a54a0c578a09dfec1fe8521bb49bca9824c82eae618f810059c5cbdf https://mirrors.edge.kernel.org/pub/linux/utils/kernel/modutils/v2.4/modutils-2.4.27.tar.gz /tmp/

RUN cd /tmp && tar xzf modutils-2.4.27.tar.gz && cd modutils-2.4.27 \
 && ./configure --disable-kerneld --disable-common-i386 \
 && make -C util && make -C obj && make -C depmod \
 && cp depmod/depmod /sbin/depmod \
 && cd / && rm -rf /tmp/modutils-2.4.27 /tmp/modutils-2.4.27.tar.gz

COPY timeout /usr/local/bin/timeout
ENV LANG=C
WORKDIR /work
