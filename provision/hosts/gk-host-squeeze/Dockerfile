# gk-host-squeeze: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-buster: the distribution's gcc as HOSTCC, make and the other tools kbuild runs, and nothing of the target side. Squeeze has no kmod, so module-init-tools stands in, and no libelf, which no kernel of its era asks for. The base is pinned by digest. Squeeze's apt cannot follow the relative redirects snapshot.debian.org answers with, so it reads archive.debian.org, where squeeze has not changed since it was archived in 2016. Its archive keys have expired, so apt is told to install what it cannot authenticate, and the Valid-Until date is not checked.

FROM debian/eol:squeeze@sha256:80ed3d6f933f102b35efaa1a9a7513e5ed9937c5a93c289dd6e2e1db51d6f331

RUN sed -i -e '/^# deb /d' /etc/apt/sources.list \
 && apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Check-Valid-Until=false install -y --force-yes --no-install-recommends \
      gcc libc6-dev make perl bison flex bc python \
      libssl-dev openssl cpio xz-utils gzip bzip2 module-init-tools \
      rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /work
