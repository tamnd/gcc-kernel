#!/bin/bash
# Builds one static toolchain bundle inside a forge container (spec 04.4). `gk forge` runs it and passes everything in through the environment.
#
# The sources come from /src, read only, already checked by `gk fetch`: the binutils and GCC tarballs named by GK_BINUTILS_TAR and GK_GCC_TAR, and GMP, MPFR, MPC and ISL under /src/infrastructure. The bundle is written to /out as a tarball whose members are relative to the bundle root, so it can be unpacked at /opt/gk/t/<id> in any host. Nothing here reaches the network.
#
# The build triple has a vendor of its own so that configure treats even an x86_64 target as a cross build. The tools then carry the target prefix, as in x86_64-linux-gnu-gcc, and nothing of the forge's own libc or headers leaks into the target side. Only C is built, with libgcc and no libc, which is what a kernel needs.

set -euo pipefail

: "${GK_ID:?}" "${GK_GCC:?}" "${GK_BINUTILS:?}" "${GK_TARGET:?}" "${GK_PREREQS:?}"
: "${GK_GCC_TAR:=gcc-$GK_GCC.tar.xz}" "${GK_BINUTILS_TAR:=binutils-$GK_BINUTILS.tar.xz}"
jobs="${GK_JOBS:-$(nproc)}"
prefix="/opt/gk/t/$GK_ID"
build="$(uname -m)-gkforge-linux-gnu"
work=/tmp/forge
stage=/tmp/stage
rm -rf "$work" "$stage"
mkdir -p "$work" "$stage"
cd "$work"

log() { echo "gk-forge: $*" >&2; }

# Nothing reads the info pages, and the texinfo of the forge that builds the 4.x releases rejects their sources, so every make is told that makeinfo is `true`.

log "unpacking binutils $GK_BINUTILS and gcc $GK_GCC"
tar -xf "/src/$GK_BINUTILS_TAR"
tar -xf "/src/$GK_GCC_TAR"
for p in $GK_PREREQS; do
  tar -xf "/src/infrastructure/$p" -C "gcc-$GK_GCC"
  dir="${p%.tar.*}"
  ln -s "$dir" "gcc-$GK_GCC/${dir%%-*}"
done

log "binutils for $GK_TARGET"
mkdir b-binutils
(
  cd b-binutils
  "../binutils-$GK_BINUTILS/configure" \
    --build="$build" --host="$build" --target="$GK_TARGET" --prefix="$prefix" \
    --disable-nls --disable-werror --disable-multilib --disable-shared --enable-static \
    --disable-gdb --disable-gdbserver --disable-sim --disable-gprofng --disable-readline \
    --disable-libdecnumber --enable-deterministic-archives
  make MAKEINFO=true -j"$jobs" configure-host
  make MAKEINFO=true -j"$jobs" LDFLAGS=-all-static
  make MAKEINFO=true install-strip DESTDIR="$stage"
) > binutils.log 2>&1 || { tail -n 60 binutils.log >&2; exit 1; }

export PATH="$stage$prefix/bin:$PATH"

log "gcc for $GK_TARGET"
mkdir b-gcc
(
  cd b-gcc
  LDFLAGS=-static "../gcc-$GK_GCC/configure" \
    --build="$build" --host="$build" --target="$GK_TARGET" --prefix="$prefix" \
    --enable-languages=c --without-headers --disable-bootstrap --disable-nls \
    --disable-multilib --disable-shared --disable-threads --disable-libssp --disable-libgomp \
    --disable-libquadmath --disable-libatomic --disable-libsanitizer --disable-libvtv \
    --disable-libstdcxx --disable-libcc1 --disable-decimal-float --disable-libmudflap \
    --disable-libmpx --disable-werror
  make MAKEINFO=true -j"$jobs" all-gcc
  make MAKEINFO=true -j"$jobs" all-target-libgcc
  make MAKEINFO=true install-strip-gcc install-target-libgcc DESTDIR="$stage"
) > gcc.log 2>&1 || { tail -n 80 gcc.log >&2; exit 1; }

log "packing"
# The man and info pages are all there is under share, and pod2man stamps the day it ran into every binutils man page, so two forges of the same bundle on different days would differ. Nothing in a cell reads them.
rm -rf "${stage:?}$prefix/share"
# Jessie's tar has no --sort and jessie has no zstd. There the member list is sorted by hand, and the tarball is left uncompressed for gk to compress outside the container.
cd "$stage$prefix"
if tar --sort=name -cf /dev/null --files-from /dev/null 2>/dev/null; then
  list=(--sort=name .)
else
  find . | LC_ALL=C sort > "$work/members"
  list=(--no-recursion -T "$work/members")
fi
out="/out/$GK_ID-$GK_TARGET.tar"
if command -v zstd > /dev/null; then
  tar --mtime=@0 --owner=0 --group=0 --numeric-owner -cf - "${list[@]}" | zstd -q -19 -T0 > "$out.zst.part"
  mv "$out.zst.part" "$out.zst"
  log "done: $out.zst"
else
  tar --mtime=@0 --owner=0 --group=0 --numeric-owner -cf "$out.part" "${list[@]}"
  mv "$out.part" "$out"
  log "done: $out, to be compressed by gk"
fi
