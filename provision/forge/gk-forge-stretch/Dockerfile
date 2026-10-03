# gk-forge-stretch: the neighbour forge for GCC 5 to 7 (spec 04.4).
#
# gk-forge-bullseye builds 5.5 to 8.5 with GCC 10. A release that fails there is tried here before it is recorded as unbuildable, with stretch's GCC 6.3, which is of the releases' own time. Stretch has zstd and a tar that sorts members, so build.sh runs here as it does in the newer forges. As in the other forges, every source tarball comes in from the gk cache and the build never touches the network.

FROM debian/eol:stretch@sha256:87de8b3de9239bd73f9d73bd3816411e156705c7517dd0177b50136e97cdb43f

# apt reads the snapshot.debian.org lines the image ships commented out. The snapshot is years past its Valid-Until date, so apt is told not to check it.
RUN sed -i -e '/^deb /d' -e 's/^# deb /deb /' /etc/apt/sources.list \
 && apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Retries=5 update \
 && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Check-Valid-Until=false install -y --no-install-recommends \
      gcc g++ libc6-dev make \
      bison flex texinfo gawk m4 perl python3 \
      xz-utils bzip2 gzip zstd file patch diffutils \
 && rm -rf /var/lib/apt/lists/*

ENV LANG=C.UTF-8
WORKDIR /forge
