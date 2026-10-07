# The binutils sweep

Which binutils builds and boots each kernel when GCC is held at the kernel's era GCC (spec 04.6). Each row starts from the matrix cell, whose binutils is the one the era GCC's bundle was built with, and the frontier search runs out from there over the last point of each binutils series until it finds the oldest and the newest release that work. A release between two that work is taken to work too, as the GCC search takes it, and a column nobody ran is left blank. When no binutils takes a kernel through every rung, say because its KUnit run fails whatever the binutils, the row is held to the highest rung any of its cells reached instead, which the column `held to` gives.

Written by `gk report binutils` from the result store. `gk binutils-sweep K --platform P` runs a row.

## x86_64, defconfig+gk

| kernel | GCC | held to | from | to | older edge | newer edge |
|---|---|---|---|---|---|---|
| 5.10.271 | gcc-10.5.0 | L7 | 2.41 | 2.42 | 2.40 fails at L2: `over-budget` | 2.43.1 builds at L4: `unclassified` |
| 6.12.112 | gcc-12.2.0 | works | 2.35.2 | 2.40 | 2.34 fails at L2: `unclassified` | 2.41 fails at L2: `over-budget` |

## Every column

One character for each binutils series, oldest first: `#` works, `+` builds or runs but does not pass every rung, `x` fails, and a dot is not run.

```
                 2.8 to 2.47
x86_64 5.10.271  ...............................x+++....  defconfig+gk
x86_64 6.12.112  ......................x.xx#.####x......  defconfig+gk
```
