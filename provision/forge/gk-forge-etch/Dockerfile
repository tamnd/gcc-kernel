# gk-forge-etch: builds the static GCC and binutils bundles for GCC 3.2 to 4.1 (spec 04.4).
#
# Etch has GCC 4.1 as gcc and GCC 3.4 as gcc-3.4. build.sh builds the 3.x releases with gcc-3.4, because GCC 4 rejects some of their C, and 4.0 and 4.1 with gcc. The spec names an i386 etch, but debian/eol has etch for amd64, and an amd64 forge makes bundles that run in every host like the rest. Etch has no xz, no zstd and a tar that cannot sort members, so build.sh leaves an uncompressed tarball with a sorted member list and gk compresses it outside the container. Every source tarball comes in from the gk cache and the build never touches the network. apt reads archive.debian.org, and since etch's archive keys have expired it is told to install what it cannot authenticate. Etch's apt has no --no-install-recommends, and does not install recommends anyway, and diffutils is still called diff there.

FROM debian/eol:etch@sha256:9fb6015b4c3f348fdfae6f6e7b22e5bf6f64de9cdcd45ed4c388300c48a47453

RUN sed -i -e '/^# deb /d' /etc/apt/sources.list \
 && apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Check-Valid-Until=false install -y --force-yes \
      gcc gcc-3.4 libc6-dev make \
      bison flex texinfo gawk m4 perl \
      bzip2 gzip file patch diff \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C
WORKDIR /forge
