# gk-host-xenial: the image of the Ubuntu 16.04 distribution column, whose gcc-5 builds with -fstack-protector-strong by default and, unlike 16.10 and later, without PIE (spec 04.3).
#
# The same recipe as the Debian hosts, with what xenial lacks left out: it has no zstd, which none of the kernels this column runs on use. The base is pinned by digest. Xenial is still on archive.ubuntu.com, where its updates no longer change.

FROM ubuntu:16.04@sha256:1f1a2d56de1d604801a9671f301190704c25d604a416f59e03c04f5c6ffee0d6

RUN apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc libc6-dev make perl bison flex bc python3 \
      libelf-dev libssl-dev openssl cpio xz-utils gzip bzip2 kmod \
      rsync file ca-certificates \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /work
