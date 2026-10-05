#!/bin/bash
# Builds one static toolchain bundle inside a forge container (spec 04.4). `gk forge` runs it and passes everything in through the environment.
#
# The sources come from /src, read only, already checked by `gk fetch`: the binutils and GCC tarballs named by GK_BINUTILS_TAR and GK_GCC_TAR, and GMP, MPFR, MPC and ISL under /src/infrastructure. The bundle is written to /out as a tarball whose members are relative to the bundle root, so it can be unpacked at /opt/gk/t/<id> in any host. Nothing here reaches the network.
#
# The build triple has a vendor of its own so that configure treats even an x86_64 target as a cross build. The tools then carry the target prefix, as in x86_64-linux-gnu-gcc, and nothing of the forge's own libc or headers leaks into the target side. Only C is built, with libgcc and no libc, which is what a kernel needs.

set -eu
# Woody's bash 2.05 has no pipefail.
set -o pipefail 2> /dev/null || true

: "${GK_ID:?}" "${GK_GCC:?}" "${GK_BINUTILS:?}" "${GK_TARGET:?}" "${GK_PREREQS?}"
: "${GK_GCC_TAR:=gcc-$GK_GCC.tar.xz}" "${GK_BINUTILS_TAR:=binutils-$GK_BINUTILS.tar.xz}"
jobs="${GK_JOBS:-$(nproc)}"
prefix="/opt/gk/t/$GK_ID"
# An i386 forge such as woody runs on an x86_64 kernel, where uname says x86_64, so the word size of the userland decides. Slink has no getconf, so it is read from the ELF class of /bin/sh, which is 1 for 32 bits.
if [ "$(od -An -tx1 -j4 -N1 /bin/sh | tr -d ' ')" = 01 ]; then
  build=i686-gkforge-linux-gnu
else
  build="$(uname -m)-gkforge-linux-gnu"
fi
work=/tmp/forge
# The container is thrown away after each build, so the bundle is installed at its own prefix rather than under a DESTDIR, which the install of releases as old as 2.95 does not know.
rm -rf "$work" "$prefix"
# The install of binutils as old as 2.8 makes the prefix with a plain mkdir, which cannot make its parents.
mkdir -p "$work" "$prefix"
cd "$work"

log() { echo "gk-forge: $*" >&2; }

# A failed step shows the first errors in its log as well as the end of it, since a make that keeps going buries the cause under the install that fails after it.
failed() {
  log "first errors in $1:"
  # Configure checks for strerror and ferror_unlocked, which are not errors. With pipefail a grep that head stops early fails the pipeline, so its status is dropped, or the end of the log below would never be printed.
  { grep -n -B3 -E '(^|[^a-z_])error|Error [0-9]' "$1" || true; } | head -n 40 >&2
  log "end of $1:"
  tail -n "$2" "$1" >&2
  exit 1
}

# Woody's tar cannot tell a compression by itself, so the decompressor is picked by the name.
unpack() {
  case "$1" in
    *.gz) gzip -dc "$1" | tar -xf - ;;
    *.bz2) bzip2 -dc "$1" | tar -xf - ;;
    *) tar -xf "$1" ;;
  esac
}

# The release as a number, 304 for 3.4.6, for the few steps that differ between old and new releases.
gcc_major="${GK_GCC%%.*}"
gcc_minor="${GK_GCC#*.}"
gcc_minor="${gcc_minor%%.*}"
gcc_series=$((gcc_major * 100 + gcc_minor))

# GCC 3.x does not build with GCC 4's stricter C, as in its casts used as lvalues, so where the forge has gcc-3.4 those releases are built with it.
if [ "$gcc_series" -lt 400 ] && command -v gcc-3.4 > /dev/null 2>&1; then
  export CC=gcc-3.4
fi
# The gcc directory of 2.95 and older links its programs with neither the LDFLAGS seen by configure nor those given to make, so there the compiler itself is told to link statically.
if [ "$gcc_series" -lt 300 ]; then
  export CC="${CC:-gcc} -static"
fi

# Nothing reads the info pages, and the texinfo of the forge that builds the 4.x releases rejects their sources, so every make is told that makeinfo is `true`.

log "unpacking binutils $GK_BINUTILS and gcc $GK_GCC"
unpack "/src/$GK_BINUTILS_TAR"
unpack "/src/$GK_GCC_TAR"
# EGCS unpacks to egcs-<release>, while the version it gives itself, and so the one the forge is handed, is the GCC one, so its directory takes the name the rest of the script looks for.
for d in egcs-*; do
  if [ -d "$d" ] && [ ! -d "gcc-$GK_GCC" ]; then
    mv "$d" "gcc-$GK_GCC"
  fi
done
for p in $GK_PREREQS; do
  (cd "gcc-$GK_GCC" && unpack "/src/infrastructure/$p")
  dir="${p%.tar.*}"
  ln -s "$dir" "gcc-$GK_GCC/${dir%%-*}"
done

log "binutils for $GK_TARGET"
mkdir b-binutils
# set -e does not reach into a subshell on the left of ||, so each step is chained, here and in both GCC builds below. Without that a make that fails goes on to the next step, and the log ends on an install that cannot find what the failed step should have made.
(
  cd b-binutils &&
    "../binutils-$GK_BINUTILS/configure" \
      --build="$build" --host="$build" --target="$GK_TARGET" --prefix="$prefix" \
      --disable-nls --disable-werror --disable-multilib --disable-shared --enable-static \
      --disable-gdb --disable-gdbserver --disable-sim --disable-gprofng --disable-readline \
      --disable-libdecnumber --enable-deterministic-archives &&
    # Releases older than about 2.17 have neither the configure-host target nor install-strip at the top level.
    if make -n configure-host > /dev/null 2>&1; then
      make MAKEINFO=true -j"$jobs" configure-host
    fi &&
    make MAKEINFO=true -j"$jobs" LDFLAGS=-all-static &&
    if make -n install-strip > /dev/null 2>&1; then
      make MAKEINFO=true install-strip
    else
      make MAKEINFO=true install && { strip "$prefix"/bin/* "$prefix/$GK_TARGET"/bin/* 2> /dev/null || true; }
    fi &&
    test -x "$prefix/bin/$GK_TARGET-as"
) > binutils.log 2>&1 || failed binutils.log 60

export PATH="$prefix/bin:$PATH"

# Before 4.3 libgcc is built inside the gcc directory by all-gcc, there is no all-target-libgcc and no install-strip-gcc, and --with-newlib is what keeps libgcc from looking for the target's C headers.
# It is a plain word rather than an array, because the bash of etch fails on an empty array under set -u.
old_gcc=
if [ "$gcc_series" -lt 403 ]; then
  old_gcc=--with-newlib
fi
# Variables on make's command line are handed down to the gcc directory, which is how the two releases below are kept away from the target's headers. EGCS has no --enable-languages, and builds every front end it ships, the Objective C runtime among them, which wants the target's stdio.h, so LANGUAGES keeps both its build and its install to C.
make_vars=
if [ "$gcc_series" -lt 295 ]; then
  make_vars=LANGUAGES=c
fi
# The 3.3 Linux headers of i386 and a few others leave the signal frame unwinder out of libgcc only for libc5, where 3.2 and 3.4 leave it out whenever there is no libc. Without it libgcc wants signal.h, so libgcc is built as if for libc5. LIBGCC2_INCLUDES is empty otherwise and only reaches libgcc.
if [ "$gcc_series" -ge 303 ] && [ "$gcc_series" -lt 304 ]; then
  make_vars=LIBGCC2_INCLUDES=-DUSE_GNULIBC_1
fi

# Before egcs GCC is a single directory, with no top level to build it from and no way to build libgcc without the target's headers. A kernel of that age needs only the driver, cc1, cpp and the headers GCC brings with it, so those are built in the tree and put in place by hand.
if [ ! -d "gcc-$GK_GCC/gcc" ]; then
  log "gcc for $GK_TARGET, without libgcc"
  # The Makefile of these releases calls the compiler cc unless told otherwise. The config.sub of 2.5 does not know a vendor of our own, nor i686, so there the forge calls itself plain i386-linux. Before 2.7 gcc.c and cccp.c declare sys_errlist without the const glibc gives it, unless bsd4_4 is defined, and the compile stops on the conflict. In those two files bsd4_4 decides nothing else, so it is defined on the command line, which leaves the sources as they were released.
  lib="$prefix/lib/gcc-lib/$GK_TARGET/$GK_GCC"
  host="$build"
  if ! sh "gcc-$GK_GCC/config.sub" "$host" > /dev/null 2>&1; then
    host=i386-linux
  fi
  cc="${CC:-gcc}"
  if [ "$gcc_series" -lt 207 ]; then
    cc="$cc -Dbsd4_4"
  fi
  (
    cd "gcc-$GK_GCC" &&
      ./configure --host="$host" --target="$GK_TARGET" --prefix="$prefix" --with-gnu-as --with-gnu-ld &&
      make CC="$cc" MAKEINFO=true LANGUAGES=c -j"$jobs" xgcc cc1 cpp specs stmp-int-hdrs &&
      mkdir -p "$lib/include" "$prefix/bin" &&
      cp xgcc "$prefix/bin/$GK_TARGET-gcc" &&
      cp cc1 cpp specs "$lib/" &&
      cp -R include/. "$lib/include/" &&
      strip "$prefix/bin/$GK_TARGET-gcc" "$lib/cc1" "$lib/cpp"
  ) > gcc.log 2>&1 || failed gcc.log 80
else
log "gcc for $GK_TARGET"
mkdir b-gcc
(
  cd b-gcc &&
    LDFLAGS=-static "../gcc-$GK_GCC/configure" $old_gcc \
      --build="$build" --host="$build" --target="$GK_TARGET" --prefix="$prefix" \
      --enable-languages=c --without-headers --disable-bootstrap --disable-nls \
      --disable-multilib --disable-shared --disable-threads --disable-libssp --disable-libgomp \
      --disable-libquadmath --disable-libatomic --disable-libsanitizer --disable-libvtv \
      --disable-libstdcxx --disable-libcc1 --disable-decimal-float --disable-libmudflap \
      --disable-libmpx --disable-werror &&
    # Before 4.3 the top level does not hand the LDFLAGS seen by configure down to the gcc directory, so the driver and cc1 come out dynamic unless make is told as well.
    if [ "$gcc_series" -lt 403 ]; then
      make MAKEINFO=true $make_vars -j"$jobs" LDFLAGS=-static all-gcc
    else
      make MAKEINFO=true -j"$jobs" all-gcc
    fi &&
    if [ "$gcc_series" -ge 403 ]; then
      make MAKEINFO=true -j"$jobs" all-target-libgcc
    fi &&
    # install-strip-gcc came in 4.4. Before that the tools are installed as they are and stripped here.
    if make -n install-strip-gcc > /dev/null 2>&1; then
      make MAKEINFO=true install-strip-gcc
    else
      # Before 3.0 install-info has no pages to copy when makeinfo is `true`, and the install stops there, before it gets to the driver, which it installs last. 2.95 stops without an error and EGCS with one, so before 3.0 a failed install-gcc is let through, and the check for the driver below still catches one that did not get installed.
      { make MAKEINFO=true $make_vars install-gcc || [ "$gcc_series" -lt 300 ]; } &&
        if [ ! -f "$prefix/bin/$GK_TARGET-gcc" ]; then
          (cd gcc && make MAKEINFO=true $make_vars install-driver)
        fi &&
        for f in "$prefix"/bin/* "$prefix"/libexec/gcc/"$GK_TARGET"/*/* "$prefix"/lib/gcc-lib/"$GK_TARGET"/*/*; do
          if [ -f "$f" ] && file "$f" | grep -q 'ELF.*executable'; then
            strip "$f"
          fi
        done
    fi &&
    if [ "$gcc_series" -ge 403 ]; then
      make MAKEINFO=true install-target-libgcc
    fi
) > gcc.log 2>&1 || failed gcc.log 80
fi

# A bundle runs in hosts as old as sarge, which have neither the forge's libc nor a 64 bit loader, so every program in it has to be static. Plugins such as liblto_plugin.so are shared objects and are left alone. Before 3.4 cc1 and collect2 live under lib/gcc-lib rather than libexec/gcc.
for f in "$prefix"/bin/* "$prefix/$GK_TARGET"/bin/* "$prefix"/libexec/gcc/"$GK_TARGET"/*/* "$prefix"/lib/gcc-lib/"$GK_TARGET"/*/*; do
  if [ -f "$f" ] && file "$f" | grep -q 'executable.*dynamically linked'; then
    log "$f is dynamically linked"
    exit 1
  fi
done

# GCC's own install of the driver ignores its errors, so a make that passes can still leave no driver, and the log is the only place that says why.
if [ ! -f "$prefix/bin/$GK_TARGET-gcc" ]; then
  log "no $GK_TARGET-gcc was installed"
  log "what was installed under bin: $(cd "$prefix/bin" && echo *)"
  grep -n -B2 -A4 -E 'install-driver|xgcc' gcc.log | tail -n 60 >&2 || true
  failed gcc.log 80
fi

log "packing"
# The man and info pages are all there is under share, or under man and info before 3.0, and pod2man stamps the day it ran into every binutils man page, so two forges of the same bundle on different days would differ. Nothing in a cell reads them.
rm -rf "${prefix:?}/share" "${prefix:?}/man" "${prefix:?}/info"
# Jessie's tar has no --sort and jessie has no zstd. There the member list is sorted by hand, and the tarball is left uncompressed for gk to compress outside the container. Woody's tar cannot set member times either, so there the installed tree itself is left for gk to pack.
cd "$prefix"
if ! tar --mtime=@0 -cf /dev/null --files-from /dev/null 2> /dev/null; then
  tree="/out/$GK_ID-$GK_TARGET.tree"
  rm -rf "$tree"
  cp -a . "$tree"
  log "done: $tree, to be packed by gk"
  exit 0
fi
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
