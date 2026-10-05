#!/bin/bash
# Builds the bundle of a distribution column inside the distribution's own host image (spec 04.3). Nothing is compiled: the bundle is a bin directory of links to the distribution's GCC and binutils under /usr/bin, which only resolve inside that image. A .gk-distribution file next to the links records the package versions and the SHA-256 of each binary, so the bundle's digest changes when the image's compiler does.
#
# It writes the tree to /out/$GK_ID-$GK_TARGET.tree, which gk packs, and prints nothing else on success.

set -euo pipefail

: "${GK_ID:?}" "${GK_TARGET:?}" "${GK_PACKAGE:?}"
driver="/usr/bin/$GK_PACKAGE"
[ -x "$driver" ] || { echo "no $driver in this image" >&2; exit 1; }
cpp="/usr/bin/${GK_PACKAGE/gcc/cpp}"

tree="/out/$GK_ID-$GK_TARGET.tree"
rm -rf "$tree"
mkdir -p "$tree/bin"
ln -s "$driver" "$tree/bin/$GK_TARGET-gcc"
[ -x "$cpp" ] && ln -s "$cpp" "$tree/bin/$GK_TARGET-cpp"
for tool in as ld ld.bfd ar nm objcopy objdump readelf strip ranlib size strings addr2line c++filt elfedit; do
  [ -x "/usr/bin/$tool" ] && ln -s "/usr/bin/$tool" "$tree/bin/$GK_TARGET-$tool"
done

version() { dpkg-query -W -f '${Version}' "$1"; }
cc1="$("$driver" -print-prog-name=cc1)"
{
  echo "package=$GK_PACKAGE $(version "$GK_PACKAGE")"
  echo "binutils=$(version binutils)"
  echo "driver=$("$driver" --version | head -n 1)"
  echo "driver-sha256=$(sha256sum "$(readlink -f "$driver")" | cut -d' ' -f1)"
  echo "cc1-sha256=$(sha256sum "$cc1" | cut -d' ' -f1)"
  echo "as-sha256=$(sha256sum "$(readlink -f /usr/bin/as)" | cut -d' ' -f1)"
  echo "ld-sha256=$(sha256sum "$(readlink -f /usr/bin/ld)" | cut -d' ' -f1)"
} > "$tree/bin/.gk-distribution"
