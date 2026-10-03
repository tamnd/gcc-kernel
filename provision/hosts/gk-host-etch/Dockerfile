# gk-host-etch: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-squeeze: the distribution's gcc as HOSTCC, make and the other tools kbuild runs, and nothing of the target side. The base is pinned by digest and apt reads archive.debian.org, where etch has not changed. Its archive keys have expired, so apt is told to install what it cannot authenticate, and the Valid-Until date is not checked. Etch's apt has no --no-install-recommends, and does not install recommends anyway. Etch has no timeout command, so a shell stand-in takes its place.

FROM debian/eol:etch@sha256:9fb6015b4c3f348fdfae6f6e7b22e5bf6f64de9cdcd45ed4c388300c48a47453

RUN sed -i -e '/^# deb /d' /etc/apt/sources.list \
 && apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Check-Valid-Until=false install -y --force-yes \
      gcc libc6-dev make perl bc cpio gzip bzip2 module-init-tools file \
 && rm -rf /var/lib/apt/lists/*

COPY timeout /usr/local/bin/timeout
ENV LANG=C
WORKDIR /work
