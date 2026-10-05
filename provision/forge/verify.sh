#!/bin/bash
# Checks one unpacked toolchain bundle at /bundle, inside a host container (spec 04.4). `gk forge verify` runs it in every G0 host, which shows the bundle runs in each of them.
#
# Every ELF file under bin and libexec has to be static: no program interpreter and no needed libraries. The driver has to report the pinned version, and it has to turn a small piece of kernel style C into an object the bundle's own binutils read back. Each check prints one line, and the script exits non zero on the first failure.

set -euo pipefail

: "${GK_TARGET:?}" "${GK_GCC:?}"
t="/bundle/bin/$GK_TARGET-"
fail() { echo "FAIL $*"; exit 1; }

if [ -f /bundle/bin/.gk-distribution ]; then
  # A distribution column links to its image's own GCC, which is dynamic by design (spec 04.3).
  echo "ok distribution $(grep '^package=' /bundle/bin/.gk-distribution | cut -d= -f2)"
else
  dynamic=0
  while IFS= read -r f; do
    head -c 4 "$f" | grep -q $'\x7fELF' || continue
    if "${t}readelf" -lW "$f" 2>/dev/null | grep -q "Requesting program interpreter"; then
      echo "dynamic: ${f#/bundle/}"
      dynamic=1
    fi
    if "${t}readelf" -dW "$f" 2>/dev/null | grep -q "(NEEDED)"; then
      echo "needs libraries: ${f#/bundle/}"
      dynamic=1
    fi
  done < <(find /bundle/bin /bundle/libexec -type f)
  [ "$dynamic" = 0 ] || fail "the bundle has dynamic binaries"
  echo "ok static"
fi

v="$("${t}gcc" -dumpfullversion 2>/dev/null || "${t}gcc" -dumpversion)"
[ "$v" = "$GK_GCC" ] || fail "gcc says $v, the pin says $GK_GCC"
echo "ok version $v"
echo "ok banner $("${t}gcc" --version | head -n 1)"
echo "ok as $("${t}as" --version | head -n 1)"

cat > /tmp/t.c <<'C'
struct list_head { struct list_head *next, *prev; };
static inline void list_add(struct list_head *n, struct list_head *h)
{
	n->next = h->next;
	n->prev = h;
	h->next->prev = n;
	h->next = n;
}
int __attribute__((section(".init.text"))) gk_probe(struct list_head *h, struct list_head *n)
{
	list_add(n, h);
	return __builtin_expect(h->next == n, 1);
}
C
"${t}gcc" -O2 -ffreestanding -nostdinc -fno-common -fno-strict-aliasing -Wall -Werror -c /tmp/t.c -o /tmp/t.o \
  || fail "gcc could not compile the probe"
"${t}objdump" -h /tmp/t.o | grep -q "\.init\.text" || fail "the object has no .init.text"
"${t}nm" /tmp/t.o | grep -q " T gk_probe" || fail "the object has no gk_probe"
"${t}ld" -r /tmp/t.o -o /tmp/t2.o || fail "ld -r failed"
echo "ok compiles and links a kernel style object"
