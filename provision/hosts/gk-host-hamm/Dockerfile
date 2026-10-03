# syntax=docker/dockerfile:1.7
# gk-host-hamm: the host side of a kernel build for the kernels its era covers in eras.toml (spec 05.2).
#
# The same recipe as gk-host-slink, for the 1.3 to 2.0 era: the distribution's gcc as HOSTCC, make, bin86 for the real mode boot code, and the modutils the base already has. Hamm has dpkg but no apt, so each package is fetched by the build itself from archive.debian.org, held to the SHA-256 written here, and unpacked with dpkg-deb, without the maintainer scripts, which none of these packages needs. The list is the closure of gcc, libc6-dev, make, bin86, file and bzip2 over what the base has, as of hamm's last Packages file, and libc5-altdev, whose libc5 headers under /usr/i486-linuxlibc1 are the ones the decompressor of 1.2 was written against, where the glibc headers of libc6-dev clash with it, and ash, which 1.0 is built with because its Makefiles run `for` over lists that can be empty, which the bash of hamm refuses. Hamm has no timeout command, so the shell stand-in of sarge takes its place.

FROM --platform=linux/386 debian/eol:hamm@sha256:68c75c6a1373739df8f4b862ea6c7f7dba0978047d584339ef22cbe37e67b4e0

ADD --checksum=sha256:59431885ab3fd9ecf9a8394e299a710d963c09597c48e0336fbe5d35f0123912 https://archive.debian.org/debian/dists/hamm/main/binary-i386/interpreters/cpp_2.7.2.3-4.8.deb /debs/
ADD --checksum=sha256:80630c64a879afe5bc78240f048b442978e1400769911b358c1b5b46d1ef8ba2 https://archive.debian.org/debian/dists/hamm/main/binary-i386/devel/binutils_2.9.1-0.2.deb /debs/
ADD --checksum=sha256:89f6af604dcde45cd170e142ca84f891dbe68dec6fcac87f58b5f034cced390c https://archive.debian.org/debian/dists/hamm/main/binary-i386/devel/gcc_2.7.2.3-4.8.deb /debs/
ADD --checksum=sha256:b037cb9181eb7043f960754683ffc231837510d1d1d3ee8f900a2fd881a58322 https://archive.debian.org/debian/dists/hamm/main/binary-i386/devel/libc6-dev_2.0.7t-1.deb /debs/
ADD --checksum=sha256:61b10c812ce74e7e76b0ffe990f5f28dfcad9f548620fb7849670631f326121b https://archive.debian.org/debian/dists/hamm/main/binary-i386/shells/ash_0.3.4-2.deb /debs/
ADD --checksum=sha256:13173299478ec723e5a3721f464016c99e7295b12c776e33168e8c9572e26291 https://archive.debian.org/debian/dists/hamm/main/binary-i386/oldlibs/libc5-altdev_5.4.38-1.1.deb /debs/
ADD --checksum=sha256:e085029e98b4f33b07da4a7cfefee6a7f0355aebef98eb41db603bfed55c4ca4 https://archive.debian.org/debian/dists/hamm/main/binary-i386/devel/make_3.76.1-8.deb /debs/
ADD --checksum=sha256:f63434b8810be02387d98a58ef84f68eb2b6ecca3dc2775ec5c2c17d70163e14 https://archive.debian.org/debian/dists/hamm/main/binary-i386/devel/bin86_0.13.0-4.deb /debs/
ADD --checksum=sha256:3f681145627a1304d097f7ce55f46f301dacf5021c301fdd2a92098a416ca56e https://archive.debian.org/debian/dists/hamm/main/binary-i386/utils/file_3.24-4.deb /debs/
ADD --checksum=sha256:3c88d6f0ebd757d1a16e5bc7dff09f5a5ecca055b411cd767961b29aa349816f https://archive.debian.org/debian/dists/hamm/main/binary-i386/utils/bzip2_0.1pl2-5.deb /debs/
RUN for deb in /debs/*.deb; do dpkg-deb -x "$deb" / || exit 1; done && rm -rf /debs

COPY timeout /usr/local/bin/timeout
ENV LANG=C
WORKDIR /work
