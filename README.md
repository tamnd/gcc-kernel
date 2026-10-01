# gcc-kernel

Which GCC builds which Linux kernel? This repository answers that by trying it. It pins every kernel release from 0.01 to today, every GCC release it can build from source, and every platform QEMU can boot, then builds each pairing it picks, boots the result, runs the kernel's own tests, and publishes a matrix. Every edge of that matrix comes with an explanation: the first GCC that works for a kernel, the last one that does, why the ones outside fail, and the kernel commit that fixed it.

It exists because of [rucc](https://github.com/tamnd/rucc), a C compiler in Rust that aims to be compatible with GCC 16, and [rucc-kernel](https://github.com/tamnd/rucc-kernel), the harness that builds kernels with it. Both depend on facts about GCC that nobody has written down in one place, such as which kernels GCC 16 itself builds and boots, and whether the GCC a kernel era is graded against really works for every version in that era. gcc-kernel never runs rucc. It only measures real GCCs, so it stays an independent reference, and its results flow one way into rucc-kernel.

Recent kernels come first, because that is where rucc is going. Old kernels are there as reference, to show how the kernel and GCC moved together.

## How it works

A cell is one kernel, one GCC, one binutils, one platform, one configuration and one host environment. Each cell climbs a ladder, from L0 (the tree is fetched and checks out) through L4 (it links) and L6 (it boots and passes the smoke suite) to L8 (KUnit passes and the console is clean). The highest rung it reaches gives its verdict: works, runs, builds, fails, or n/a.

The kernel picks the host. `HOSTCC` and the host tools come from a Debian container of the kernel's own era, and the GCC under test comes in as a static toolchain bundle through `CROSS_COMPILE`, so a failure along a row is the toolchain's and not the host's.

The matrix is not swept densely. Each row is searched outward from the GCC the kernel was built with when it was current, until the edges are found, with samples inside to catch holes. A few stripes are run in full: the kernels kernel.org maintains today, the last point of every longterm line, the GCC 16 column, and the era diagonal.

## The rules

**Nothing is patched.** Kernels are unmodified tarballs checked by hash. GCC and binutils are built from release tarballs. Anything that cannot be built without a patch is recorded as unbuildable, with its log.

**Every failure gets a name.** The first error of every failing cell is matched against a catalog of signatures, each with the kernel commit that fixed it, and that commit is confirmed by bisection.

**No hidden configuration.** Everything that changes a cell is in a committed file: kernels, toolchains, hosts, platforms, configs, signatures and the search seed.

**Same schema as rucc-kernel.** The compiler shim `gk-cc` is rucc-kernel's `rk-cc`, and a gcc-kernel cell directory is a rucc-kernel build directory, so rucc-kernel's diff tools read it unchanged.

## Platforms

Tier 1 is x86_64, i386 and arm64. Tier 2 is arm, riscv64, ppc64le, s390x and loongarch64. Tier 3 gets build only cells on current kernels.

## Milestones

The full plan is in [docs/spec](docs/spec). It is split into seven milestones, G0 to G6. Each one has a GitHub milestone and a tracking issue of the same name.

| | Scope |
|---|---|
| G0 | the instrument: model, shim, modern forge, one dense row |
| G1 | the current stripe on tier 1, boots and KUnit, the GCC 16 column |
| G2 | GCC 4.6 to 7.5, kernels 3.0 to 4.20, the failure catalog, the eras report |
| G3 | the legacy forge, 2.6.x, 2.4 and the museum, the 0.x experiment |
| G4 | tier 2 platforms |
| G5 | distribution GCCs and the binutils sweep |
| G6 | steady state: new kernels and GCCs show up without anyone touching the repo |

## Building

`cargo build --release` builds `gk`. The Rust toolchain is pinned in `rust-toolchain.toml`. Right now `gk` knows `check`, `pins`, `fetch`, `forge`, `hosts check`, `pins changed`, `probe`, `cell`, `ladder` and `version`. `gk check` reads the pin files at the top of the repository and checks that they agree with each other. `gk pins` applies the rules in `sets.toml` to kernel.org and the GNU mirror and shows how the pins would change, and `gk pins --write` writes them. `gk fetch` downloads pinned tarballs into `~/.cache/gk` and checks them against their pins and the signing keys in `keys/`. `gk forge` builds a static GCC and binutils bundle in a forge container, and `gk forge verify` checks every bundle in every host. `gk hosts check` runs each host and forge container and compares its GCC, binutils and make with `hosts.toml`. `gk probe 7.2.8 gcc-16.2.0 --platform x86_64` asks whether that kernel accepts that GCC at all, by running `defconfig` and preprocessing `init/main.c`. `gk cell` with the same arguments builds the cell up to L4 in its host container, with `gk-cc` logging every compiler call, and writes the cell directory under `~/.cache/gk/store/cells`. It needs a static `gk-cc` next to `gk`, which `cargo build --release --target x86_64-unknown-linux-musl -p gk-cc` gives. `gk pins changed BASE` lists the accept probes that the pins changed since a git revision call for, and `pr.yml` runs them on every pull request that touches a pin file. The rest of the commands in [docs/spec/10-gcc-kernel-repo.md](docs/spec/10-gcc-kernel-repo.md) arrive with the milestones that need them. Running cells will need Linux, Docker and QEMU.

## House style

Prose in this repository is plain English with one paragraph per line, no em or en dashes and no horizontal rules. `scripts/style.sh` checks it on every pull request.

## License

Apache-2.0. The kernel trees and toolchains it fetches are under their own licenses and are never stored here.
