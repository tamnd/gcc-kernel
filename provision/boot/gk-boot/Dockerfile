# gk-boot: the QEMU that boots every cell (spec 07.4).
#
# It holds QEMU and nothing else a cell needs. The kernel image and the initramfs are mounted read only, and the console comes back on stdout. Its digest is the qemu coordinate of every booted cell, so a QEMU upgrade is a new image, a new pin in hosts.toml and new cells, never a silent change. The base and the apt snapshot match gk-host-trixie.

FROM debian:trixie@sha256:9cc080028c43b27d2074d63a5f9caf7166d731494965616c1a6d2827a004585c

ARG SNAPSHOT=20260930T000000Z

RUN rm -f /etc/apt/sources.list.d/debian.sources \
 && echo "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/${SNAPSHOT} trixie main" > /etc/apt/sources.list \
 && echo "deb [check-valid-until=no] http://snapshot.debian.org/archive/debian-security/${SNAPSHOT} trixie-security main" >> /etc/apt/sources.list \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      qemu-system-x86 qemu-system-arm qemu-system-misc seabios \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /boot
