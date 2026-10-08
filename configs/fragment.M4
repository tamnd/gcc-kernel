# The test fragment for era M4, kernels 2.5.x and 2.6.0 to 2.6.15 (spec 07.2). It is merged after the configuration target and only turns things on, except for the four switches at the top. KUnit arrives in 5.5, so this era has none. Kernels before 3.7 have no olddefconfig, so the merged .config is settled with oldconfig, which takes the default for every new symbol when it gets no answers.

# Off everywhere, so that every column of a row gets the same configuration. None of the four exist in this era, so the merge notes them and moves on. They stay so the fragment of every era starts the same way.
CONFIG_WERROR=n
CONFIG_GCC_PLUGINS=n
CONFIG_RUST=n
CONFIG_DEBUG_INFO_BTF=n

# The serial console the boot rig reads.
CONFIG_SERIAL_8250=y
CONFIG_SERIAL_8250_CONSOLE=y

# What gk-init needs: an initramfs, device nodes, and the pseudo file systems it mounts. DEVTMPFS arrives in 2.6.32, after this era, so the boot rig's archive carries `/dev/null` itself and the merge notes these two as missing. They stay so every era asks for the same things.
CONFIG_BLK_DEV_INITRD=y
# Up to 2.6.16 BLK_DEV_INITRD depends on BLK_DEV_RAM=y, which i386's defconfig leaves out, so without this line the merge drops the initramfs and the kernel looks for a root device instead. 340e48e662dc in 2.6.17 removed the dependency.
CONFIG_BLK_DEV_RAM=y
CONFIG_DEVTMPFS=y
CONFIG_DEVTMPFS_MOUNT=y
CONFIG_TMPFS=y
CONFIG_PROC_FS=y
CONFIG_SYSFS=y

# Timestamps on the console. PANIC_ON_OOPS arrives in 3.5, and the oops=panic the boot rig passes does the same here.
CONFIG_PRINTK_TIME=y
