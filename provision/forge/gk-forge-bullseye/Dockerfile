# gk-forge-bullseye: builds the static GCC and binutils bundles for GCC 5.5 to 8.5 (spec 04.4).
#
# Those releases do not build with a GCC as new as trixie's, so they get bullseye's GCC 10. Bullseye is end of life, and its packages come from snapshot.debian.org. As in gk-forge-modern, every source tarball comes in from the gk cache and the build never touches the network.

FROM debian/eol:bullseye@sha256:ba7817dd622861e0f29ac5cd2dff6a9ba30937d6c72037a88e981593a51c5f64

# The image already has bullseye's last security updates installed, and the live security mirror no longer has all of their files, so apt reads the snapshot.debian.org lines the image ships commented out, taken on the day it was built.
RUN sed -i -e '/^deb /d' -e 's/^# deb /deb [check-valid-until=no] /' /etc/apt/sources.list \
 && rm -rf /etc/apt/sources.list.d/* \
 && apt-get -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      gcc g++ libc6-dev libstdc++-10-dev make \
      bison flex texinfo gawk m4 perl python3 \
      xz-utils bzip2 gzip zstd file patch diffutils \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /forge
