# gk-host-precise: the image of the Ubuntu 12.04 distribution column, whose gcc-4.4 and gcc-4.6 build with -fstack-protector by default unless -ffreestanding or -nostdlib is given (spec 04.3).
#
# The same recipe as the Debian hosts, with both GCCs: gcc is 4.6, the HOSTCC, and gcc-4.4 is the column. The base is pinned by digest. Precise's archive moved to old-releases.ubuntu.com when it reached its end of life, so apt is pointed there. Precise still has module-init-tools, not kmod.

FROM ubuntu:12.04@sha256:18305429afa14ea462f810146ba44d4363ae76e4c8dfc38288cf73aa07485005

RUN sed -i -e 's|http://archive.ubuntu.com|http://old-releases.ubuntu.com|' -e 's|http://security.ubuntu.com|http://old-releases.ubuntu.com|' /etc/apt/sources.list \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc gcc-4.4 libc6-dev make perl bison flex bc cpio xz-utils gzip bzip2 \
      module-init-tools rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C
WORKDIR /work
