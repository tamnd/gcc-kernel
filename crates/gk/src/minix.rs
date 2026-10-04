//! The root image of a museum boot: a Minix v1 file system, written here byte by byte.
//!
//! Every kernel from 1.0 to 2.4 can mount Minix v1, it is writable, and its layout is small enough to write without a tool, so the image's bytes depend on the init program alone. It is 1440 blocks of 1024 bytes, the size of a 1.44 MB floppy, with 14 character names, as `mkfs.minix` made them by default. Every inode has owner 0 and time 0.
//!
//! The image holds `/sbin/init`, the device nodes the kernels of the era open for init's console, an empty `/proc`, and a `/tmp` that the smoke suite writes to.

/// The block size, which Minix v1 fixes at 1024.
const BLOCK: usize = 1024;

/// The image size in blocks.
const BLOCKS: usize = 1440;

/// The number of inodes, a multiple of the 32 that fit in a block.
const INODES: usize = 480;

/// The magic of Minix v1 with 14 character names.
const MAGIC: u16 = 0x137f;

/// Directory entries are an inode number and a name, 16 bytes in all.
const NAME: usize = 14;

const S_IFDIR: u16 = 0o040_000;
const S_IFREG: u16 = 0o100_000;
const S_IFCHR: u16 = 0o020_000;

/// What goes in a node of the image.
enum Node {
    Dir(Vec<(&'static str, usize)>),
    File(Vec<u8>),
    Char(u8, u8),
}

/// One inode before it is written.
struct Inode {
    mode: u16,
    links: u8,
    node: Node,
}

/// The layout: boot block, super block, one block of inode map, one of zone map, then the inode table and the data zones.
const IMAP: usize = 2;
const ZMAP: usize = 3;
const ITABLE: usize = 4;
const FIRST_ZONE: usize = ITABLE + INODES * 32 / BLOCK;

/// The tree of the image, by inode number from 1, which is the root.
fn nodes(init: &[u8]) -> [Inode; 11] {
    [
        Inode {
            mode: S_IFDIR | 0o755,
            links: 6,
            node: Node::Dir(vec![
                (".", 1),
                ("..", 1),
                ("sbin", 2),
                ("dev", 3),
                ("proc", 4),
                ("tmp", 5),
            ]),
        },
        Inode {
            mode: S_IFDIR | 0o755,
            links: 2,
            node: Node::Dir(vec![(".", 2), ("..", 1), ("init", 6)]),
        },
        Inode {
            mode: S_IFDIR | 0o755,
            links: 2,
            node: Node::Dir(vec![
                (".", 3),
                ("..", 1),
                ("console", 7),
                ("tty0", 8),
                ("tty1", 9),
                ("ttyS0", 10),
                ("null", 11),
            ]),
        },
        Inode {
            mode: S_IFDIR | 0o555,
            links: 2,
            node: Node::Dir(vec![(".", 4), ("..", 1)]),
        },
        Inode {
            mode: S_IFDIR | 0o1777,
            links: 2,
            node: Node::Dir(vec![(".", 5), ("..", 1)]),
        },
        Inode {
            mode: S_IFREG | 0o755,
            links: 1,
            node: Node::File(init.to_vec()),
        },
        Inode {
            mode: S_IFCHR | 0o600,
            links: 1,
            node: Node::Char(5, 1),
        },
        Inode {
            mode: S_IFCHR | 0o620,
            links: 1,
            node: Node::Char(4, 0),
        },
        Inode {
            mode: S_IFCHR | 0o620,
            links: 1,
            node: Node::Char(4, 1),
        },
        Inode {
            mode: S_IFCHR | 0o660,
            links: 1,
            node: Node::Char(4, 64),
        },
        Inode {
            mode: S_IFCHR | 0o666,
            links: 1,
            node: Node::Char(1, 3),
        },
    ]
}

/// The root image with `init` as `/sbin/init`.
#[must_use]
pub fn image(init: &[u8]) -> Vec<u8> {
    let nodes = nodes(init);
    let mut img = vec![0u8; BLOCKS * BLOCK];
    let mut next_zone = FIRST_ZONE;
    let mut alloc = |img: &mut Vec<u8>, data: &[u8]| -> u16 {
        let zone = next_zone;
        next_zone += 1;
        assert!(
            next_zone <= BLOCKS,
            "the init program does not fit the image"
        );
        img[zone * BLOCK..zone * BLOCK + data.len()].copy_from_slice(data);
        u16::try_from(zone).unwrap_or(0)
    };

    for (i, inode) in nodes.iter().enumerate() {
        let (size, data): (usize, Vec<u8>) = match &inode.node {
            Node::Dir(entries) => {
                let mut d = Vec::new();
                for (name, ino) in entries {
                    d.extend_from_slice(&u16::try_from(*ino).unwrap_or(0).to_le_bytes());
                    let mut n = [0u8; NAME];
                    n[..name.len()].copy_from_slice(name.as_bytes());
                    d.extend_from_slice(&n);
                }
                (d.len(), d)
            }
            Node::File(bytes) => (bytes.len(), bytes.clone()),
            Node::Char(..) => (0, Vec::new()),
        };
        let mut zones = [0u16; 9];
        if let Node::Char(major, minor) = inode.node {
            zones[0] = u16::from(major) << 8 | u16::from(minor);
        } else {
            let chunks: Vec<&[u8]> = data.chunks(BLOCK).collect();
            for (n, chunk) in chunks.iter().enumerate().take(7) {
                zones[n] = alloc(&mut img, chunk);
            }
            if chunks.len() > 7 {
                assert!(chunks.len() <= 7 + BLOCK / 2, "the init program is too big");
                let mut table = Vec::new();
                for chunk in &chunks[7..] {
                    table.extend_from_slice(&alloc(&mut img, chunk).to_le_bytes());
                }
                zones[7] = alloc(&mut img, &table);
            }
        }
        let at = ITABLE * BLOCK + i * 32;
        let raw = &mut img[at..at + 32];
        raw[0..2].copy_from_slice(&inode.mode.to_le_bytes());
        raw[4..8].copy_from_slice(&u32::try_from(size).unwrap_or(0).to_le_bytes());
        raw[13] = inode.links;
        for (n, z) in zones.iter().enumerate() {
            raw[14 + 2 * n..16 + 2 * n].copy_from_slice(&z.to_le_bytes());
        }
    }

    // Bit 0 of each map is reserved and set. Bit n of the inode map is inode n, and bit n of the zone map is zone FIRST_ZONE + n - 1. Bits past the end are set too, so the kernel never hands them out.
    let used_zones = next_zone - FIRST_ZONE;
    let zone_bits = BLOCKS - FIRST_ZONE + 1;
    for bit in 0..BLOCK * 8 {
        if bit <= nodes.len() || bit > INODES {
            img[IMAP * BLOCK + bit / 8] |= 1 << (bit % 8);
        }
        if bit <= used_zones || bit >= zone_bits {
            img[ZMAP * BLOCK + bit / 8] |= 1 << (bit % 8);
        }
    }

    let sb = &mut img[BLOCK..BLOCK + 20];
    let put16 = |sb: &mut [u8], at: usize, v: usize| {
        sb[at..at + 2].copy_from_slice(&u16::try_from(v).unwrap_or(0).to_le_bytes());
    };
    put16(sb, 0, INODES);
    put16(sb, 2, BLOCKS);
    put16(sb, 4, 1);
    put16(sb, 6, 1);
    put16(sb, 8, FIRST_ZONE);
    put16(sb, 10, 0);
    sb[12..16].copy_from_slice(&((7 + 512 + 512 * 512) * 1024u32).to_le_bytes());
    sb[16..18].copy_from_slice(&MAGIC.to_le_bytes());
    // The state is "cleanly unmounted", so no kernel complains that it was not checked.
    sb[18..20].copy_from_slice(&1u16.to_le_bytes());
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le16(img: &[u8], at: usize) -> usize {
        usize::from(u16::from_le_bytes([img[at], img[at + 1]]))
    }

    fn inode(img: &[u8], n: usize) -> &[u8] {
        let at = ITABLE * BLOCK + (n - 1) * 32;
        &img[at..at + 32]
    }

    /// Read a regular file back the way the kernel does, through the direct zones and the indirect one.
    fn read(img: &[u8], n: usize) -> Vec<u8> {
        let raw = inode(img, n);
        let size = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]) as usize;
        let mut zones: Vec<usize> = (0..7).map(|i| le16(raw, 14 + 2 * i)).collect();
        let indirect = le16(raw, 28);
        if indirect != 0 {
            zones.extend((0..BLOCK / 2).map(|i| le16(img, indirect * BLOCK + 2 * i)));
        }
        let mut out = Vec::new();
        for z in zones.into_iter().take(size.div_ceil(BLOCK)) {
            out.extend_from_slice(&img[z * BLOCK..(z + 1) * BLOCK]);
        }
        out.truncate(size);
        out
    }

    #[test]
    fn the_super_block_is_minix_v1() {
        let img = image(b"init");
        assert_eq!(img.len(), BLOCKS * BLOCK);
        assert_eq!(le16(&img, BLOCK), INODES);
        assert_eq!(le16(&img, BLOCK + 2), BLOCKS);
        assert_eq!(le16(&img, BLOCK + 8), FIRST_ZONE);
        assert_eq!(le16(&img, BLOCK + 16), 0x137f);
    }

    #[test]
    fn init_reads_back_through_the_indirect_zone() {
        let init: Vec<u8> = (0..20_000u32).map(|i| (i * 31 % 251) as u8).collect();
        let img = image(&init);
        assert_eq!(read(&img, 6), init);
        assert_ne!(le16(inode(&img, 6), 28), 0);
    }

    #[test]
    fn the_root_names_sbin_and_dev() {
        let img = image(b"x");
        let root = le16(inode(&img, 1), 14);
        let names: Vec<String> = (0..6)
            .map(|i| {
                let at = root * BLOCK + 16 * i + 2;
                String::from_utf8_lossy(&img[at..at + NAME])
                    .trim_end_matches('\0')
                    .to_owned()
            })
            .collect();
        assert_eq!(names, [".", "..", "sbin", "dev", "proc", "tmp"]);
        assert_eq!(le16(inode(&img, 9), 14), 0x0401);
    }

    #[test]
    fn the_maps_cover_what_is_used() {
        let img = image(b"x");
        // Root, sbin, dev, proc, tmp and init each take one zone.
        let used = 6;
        let bit = |map: usize, n: usize| img[map * BLOCK + n / 8] >> (n % 8) & 1;
        assert_eq!(bit(IMAP, 11), 1);
        assert_eq!(bit(IMAP, 12), 0);
        assert_eq!(bit(ZMAP, used), 1);
        assert_eq!(bit(ZMAP, used + 1), 0);
        assert_eq!(bit(ZMAP, BLOCKS - FIRST_ZONE + 1), 1);
    }

    #[test]
    fn the_image_depends_on_init_alone() {
        assert_eq!(image(b"same"), image(b"same"));
        assert_ne!(image(b"same"), image(b"other"));
    }
}
