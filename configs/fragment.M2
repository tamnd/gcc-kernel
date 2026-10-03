# The test fragment for era M2, kernels 2.1 to 2.2.26 (spec 07.2). It is merged after the configuration target and only turns things on, except for the four switches at the top. The merged .config is settled with oldconfig, which in this era is `Configure -d` and reads arch/i386/defconfig when there is no .config.

# Off everywhere, so that every column of a row gets the same configuration. None of the four exist in this era, so the merge notes them and moves on. They stay so the fragment of every era starts the same way.
CONFIG_WERROR=n
CONFIG_GCC_PLUGINS=n
CONFIG_RUST=n
CONFIG_DEBUG_INFO_BTF=n

# The serial console the boot rig reads, from 2.1.25.
CONFIG_SERIAL=y
CONFIG_SERIAL_CONSOLE=y

# The boot image is a ROMFS initial RAM disk mounted as root, since initramfs is a 2.6 thing.
CONFIG_BLK_DEV_RAM=y
CONFIG_BLK_DEV_INITRD=y
CONFIG_ROMFS_FS=y
CONFIG_PROC_FS=y
