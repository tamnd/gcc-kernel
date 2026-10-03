#!/bin/bash
# Builds one static toolchain bundle inside a forge container (spec 04.4). `gk forge` runs it and passes everything in through the environment.
#
# The sources come from /src, read only, already checked by `gk fetch`: the binutils and GCC tarballs named by GK_BINUTILS_TAR and GK_GCC_TAR, and GMP, MPFR, MPC and ISL under /src/infrastructure. The bundle is written to /out as a tarball whose members are relative to the bundle root, so it can be unpacked at /opt/gk/t/<id> in any host. Nothing here reaches the network.
#
# The build triple has a vendor of its own so that configure treats even an x86_64 target as a cross build. The tools then carry the target prefix, as in x86_64-linux-gnu-gcc, and nothing of the forge's own libc or headers leaks into the target side. Only C is built, with libgcc and no libc, which is what a kernel needs.

set -euo pipefail

: "${GK_ID:?}" "${GK_GCC:?}" "${GK_BINUTILS:?}" "${GK_TARGET:?}" "${GK_PREREQS?}"
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

# The release as a number, 304 for 3.4.6, for the few steps that differ between old and new releases.
gcc_major="${GK_GCC%%.*}"
gcc_minor="${GK_GCC#*.}"
gcc_minor="${gcc_minor%%.*}"
gcc_series=$((gcc_major * 100 + gcc_minor))

# GCC 3.x does not build with GCC 4's stricter C, as in its casts used as lvalues, so where the forge has gcc-3.4 those releases are built with it.
if [ "$gcc_series" -lt 400 ] && command -v gcc-3.4 > /dev/null; then
  export CC=gcc-3.4
fi

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
  # Releases older than about 2.17 have neither the configure-host target nor install-strip at the top level.
  if make -n configure-host > /dev/null 2>&1; then
    make MAKEINFO=true -j"$jobs" configure-host
  fi
  make MAKEINFO=true -j"$jobs" LDFLAGS=-all-static
  if make -n install-strip > /dev/null 2>&1; then
    make MAKEINFO=true install-strip DESTDIR="$stage"
  else
    make MAKEINFO=true install DESTDIR="$stage"
    strip "$stage$prefix"/bin/* "$stage$prefix/$GK_TARGET"/bin/* 2> /dev/null || true
  fi
) > binutils.log 2>&1 || { tail -n 60 binutils.log >&2; exit 1; }

export PATH="$stage$prefix/bin:$PATH"

# Before 4.3 libgcc is built inside the gcc directory by all-gcc, there is no all-target-libgcc and no install-strip-gcc, and --with-newlib is what keeps libgcc from looking for the target's C headers.
# It is a plain word rather than an array, because the bash of etch fails on an empty array under set -u.
old_gcc=
if [ "$gcc_series" -lt 403 ]; then
  old_gcc=--with-newlib
fi

log "gcc for $GK_TARGET"
mkdir b-gcc
(
  cd b-gcc
  LDFLAGS=-static "../gcc-$GK_GCC/configure" $old_gcc \
    --build="$build" --host="$build" --target="$GK_TARGET" --prefix="$prefix" \
    --enable-languages=c --without-headers --disable-bootstrap --disable-nls \
    --disable-multilib --disable-shared --disable-threads --disable-libssp --disable-libgomp \
    --disable-libquadmath --disable-libatomic --disable-libsanitizer --disable-libvtv \
    --disable-libstdcxx --disable-libcc1 --disable-decimal-float --disable-libmudflap \
    --disable-libmpx --disable-werror
  make MAKEINFO=true -j"$jobs" all-gcc
  if [ "$gcc_series" -ge 403 ]; then
    make MAKEINFO=true -j"$jobs" all-target-libgcc
  fi
  # install-strip-gcc came in 4.4. Before that the tools are installed as they are and stripped here.
  if make -n install-strip-gcc > /dev/null 2>&1; then
    make MAKEINFO=true install-strip-gcc DESTDIR="$stage"
  else
    make MAKEINFO=true install-gcc DESTDIR="$stage"
    for f in "$stage$prefix"/bin/* "$stage$prefix"/libexec/gcc/"$GK_TARGET"/*/*; do
      if file "$f" | grep -q 'ELF.*executable'; then
        strip "$f"
      fi
    done
  fi
  if [ "$gcc_series" -ge 403 ]; then
    make MAKEINFO=true install-target-libgcc DESTDIR="$stage"
  fi
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
