# gk-host-hardy: the image of the Ubuntu 8.04 distribution column, whose gcc-4.2 builds with -fstack-protector by default unless -ffreestanding or -nostdlib is given (spec 04.3).
#
# Hardy's image on Docker Hub is in the old manifest format current Docker does not pull, so the root file system is made with debootstrap from old-releases.ubuntu.com, where hardy has not changed, and copied into an empty image. wget is kept to IPv4, since a build network may have no IPv6 route and debootstrap's wget then waits forever. The archive keys have expired, so the packages are not authenticated. The initscripts postinst mounts file systems, which a build may not do, so it is replaced with an empty one and the configuration finished by hand. Then the same tools as the Debian hosts go in, gcc-4.2 among them.

FROM debian:bookworm@sha256:f37a335e82bca302e955fa39f9dfe28f1be618f016f8a2b56318e5a5111afc26 AS bootstrap

RUN apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends debootstrap \
 && rm -rf /var/lib/apt/lists/* \
 && echo 'inet4_only = on' >> /etc/wgetrc
RUN debootstrap --no-check-gpg --variant=minbase --arch=amd64 hardy /r http://old-releases.ubuntu.com/ubuntu || true \
 && printf '#!/bin/sh\nexit 0\n' > /r/var/lib/dpkg/info/initscripts.postinst \
 && chroot /r dpkg --configure -a \
 && cp /etc/resolv.conf /r/etc/resolv.conf \
 && chroot /r apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive chroot /r apt-get install -y --force-yes \
      gcc gcc-4.2 libc6-dev make perl bc cpio gzip bzip2 module-init-tools file \
 && chroot /r apt-get clean \
 && rm -rf /r/var/lib/apt/lists/* /r/etc/resolv.conf /r/debootstrap

FROM scratch
COPY --from=bootstrap /r /
ENV LANG=C PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
WORKDIR /work
CMD ["/bin/bash"]
