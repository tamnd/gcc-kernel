#!/bin/sh
# The incremental runs of spec 09.7, as sweep.yml runs them on a self hosted machine.
#
# usage: scripts/sweep.sh BASE     run what the pin changes since the git revision BASE call for
#        scripts/sweep.sh FILE     run the gk command lines in FILE, one per line
#
# Each line is a gk command such as `search 7.2.8 --platform x86_64`. Bundles a line needs and the cache does not have are forged first. Everything runs under nice, and the store and cache are GK_STORE and GK_CACHE as usual. A cell that does not work is a result like any other. A line gk cannot run at all is reported and the rest still run, and the script exits 1 at the end if there was one.
set -eu
cd "$(dirname "$0")/.."

cargo build --release --locked -q -p gk
rustup target add x86_64-unknown-linux-musl > /dev/null 2>&1 || true
cargo build --release --locked -q -p gk-cc --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/gk-cc target/release/gk-cc
gk=./target/release/gk

runs=$(mktemp)
trap 'rm -f "$runs"' EXIT
if [ -f "${1:?name a git revision or a file of runs}" ]; then
  grep -v '^[[:space:]]*\(#\|$\)' "$1" > "$runs" || true
elif [ -z "$(echo "$1" | tr -d 0)" ]; then
  echo "the base revision is all zeros, which is a new branch; there is nothing to compare against"
  exit 0
else
  $gk pins changed "$1" --sweep > "$runs"
  for gcc in $($gk pins changed "$1" --bundles); do
    for p in x86_64 i386 arm64; do
      triple=$(sed -n "/^name = \"$p\"/,/^triple/s/^triple = \"\(.*\)\"/\1/p" platforms.toml)
      if ! ls "${GK_CACHE:-$HOME/.cache/gk}/bundles/$gcc-$triple".* > /dev/null 2>&1; then
        nice $gk forge "$gcc" --target "$triple" || echo "forging $gcc for $triple failed"
      fi
    done
  done
fi

n=$(wc -l < "$runs" | tr -d ' ')
echo "$n runs"
failed=0
i=0
while IFS= read -r line; do
  i=$((i + 1))
  echo "== $i/$n gk $line"
  # shellcheck disable=SC2086
  status=0
  nice $gk $line < /dev/null || status=$?
  # 1 is a cell that did not work, which is a result. 2 and above is gk failing to run it.
  if [ "$status" -ge 2 ]; then echo "gk $line could not run"; failed=1; fi
done < "$runs"
exit "$failed"
