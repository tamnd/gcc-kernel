//! The root disk of the first 2.6 kernels: an ext2 file system, written here byte by byte.
//!
//! Before 2.6.6 a kernel unpacks the initramfs and then mounts `root=` whatever the archive holds, so it never runs `/init` from it. Their defconfigs build IDE disks and ext2 in, so they boot from this image on the first IDE disk instead. It is revision 0 ext2 with 1024 byte blocks and a single group, the size of a 1.44 MB floppy, and it holds what the initramfs of a kernel with no devtmpfs holds. Every inode has owner 0 and time 0, so the image's bytes depend on the init program alone.

/// The block size.
const BLOCK: usize = 1024;

/// The image size in blocks.
const BLOCKS: usize = 1440;

/// The number of inodes, which fill the inode table's blocks.
const INODES: usize = 64;

/// The size of an inode in revision 0.
const INODE: usize = 128;

/// The magic of ext2.
const MAGIC: u16 = 0xef53;

const S_IFDIR: u16 = 0o040_000;
const S_IFREG: u16 = 0o100_000;
const S_IFCHR: u16 = 0o020_000;

/// The layout: boot block, super block, group descriptors, block bitmap, inode bitmap, then the inode table and the data blocks.
const GDT: usize = 2;
const BMAP: usize = 3;
const IMAP: usize = 4;
const ITABLE: usize = 5;
const FIRST_DATA: usize = ITABLE + INODES * INODE / BLOCK;

/// The direct blocks of an inode, before the indirect one.
const DIRECT: usize = 12;

/// What goes in a node of the image.
enum Node {
    Dir(Vec<(&'static str, usize)>),
    File(Vec<u8>),
    Char(u8, u8),
}

/// One inode before it is written.
struct Inode {
    number: usize,
    mode: u16,
    links: u16,
    node: Node,
}

fn dir(number: usize, mode: u16, links: u16, entries: Vec<(&'static str, usize)>) -> Inode {
    Inode {
        number,
        mode: S_IFDIR | mode,
        links,
        node: Node::Dir(entries),
    }
}

fn chr(number: usize, mode: u16, major: u8, minor: u8) -> Inode {
    Inode {
        number,
        mode: S_IFCHR | mode,
        links: 1,
        node: Node::Char(major, minor),
    }
}

/// The tree of the image: the root, which is inode 2, then the inodes from 11 on, as revision 0 reserves the ten before it.
fn nodes(init: &[u8]) -> Vec<Inode> {
    vec![
        dir(
            2,
            0o755,
            6,
            vec![
                (".", 2),
                ("..", 2),
                ("dev", 11),
                ("proc", 12),
                ("sys", 13),
                ("tmp", 14),
                ("init", 15),
            ],
        ),
        dir(
            11,
            0o755,
            2,
            vec![(".", 11), ("..", 2), ("console", 16), ("null", 17)],
        ),
        dir(12, 0o555, 2, vec![(".", 12), ("..", 2)]),
        dir(13, 0o555, 2, vec![(".", 13), ("..", 2)]),
        dir(14, 0o1777, 2, vec![(".", 14), ("..", 2)]),
        Inode {
            number: 15,
            mode: S_IFREG | 0o755,
            links: 1,
            node: Node::File(init.to_vec()),
        },
        chr(16, 0o600, 5, 1),
        chr(17, 0o666, 1, 3),
    ]
}

fn put16(img: &mut [u8], at: usize, v: usize) {
    img[at..at + 2].copy_from_slice(&u16::try_from(v).unwrap_or(0).to_le_bytes());
}

fn put32(img: &mut [u8], at: usize, v: usize) {
    img[at..at + 4].copy_from_slice(&u32::try_from(v).unwrap_or(0).to_le_bytes());
}

/// A directory's one block. Each entry is the inode, the record length, and the name's length as 16 bits, which is what revision 0 has where later ones split it with a file type. The last record runs to the end of the block.
fn dir_block(entries: &[(&str, usize)]) -> Vec<u8> {
    let mut d = Vec::with_capacity(BLOCK);
    for (n, (name, ino)) in entries.iter().enumerate() {
        let len = if n + 1 == entries.len() {
            BLOCK - d.len()
        } else {
            (8 + name.len()).next_multiple_of(4)
        };
        let at = d.len();
        d.resize(at + len, 0);
        put32(&mut d, at, *ino);
        put16(&mut d, at + 4, len);
        put16(&mut d, at + 6, name.len());
        d[at + 8..at + 8 + name.len()].copy_from_slice(name.as_bytes());
    }
    d
}

/// The root image with `init` as `/init`, `/dev/console` and `/dev/null`, and the mount points init uses.
#[must_use]
pub fn image(init: &[u8]) -> Vec<u8> {
    let nodes = nodes(init);
    let mut img = vec![0u8; BLOCKS * BLOCK];
    let mut next = FIRST_DATA;
    let mut alloc = |img: &mut Vec<u8>, data: &[u8]| -> usize {
        let block = next;
        next += 1;
        assert!(next <= BLOCKS, "the init program does not fit the image");
        img[block * BLOCK..block * BLOCK + data.len()].copy_from_slice(data);
        block
    };

    for inode in &nodes {
        let (size, data) = match &inode.node {
            Node::Dir(entries) => (BLOCK, dir_block(entries)),
            Node::File(bytes) => (bytes.len(), bytes.clone()),
            Node::Char(..) => (0, Vec::new()),
        };
        let mut blocks = [0usize; 15];
        let mut used = 0;
        if let Node::Char(major, minor) = inode.node {
            // The old encoding in the first block pointer, which every 2.6 kernel reads.
            blocks[0] = usize::from(major) << 8 | usize::from(minor);
        } else {
            let chunks: Vec<&[u8]> = data.chunks(BLOCK).collect();
            assert!(
                chunks.len() <= DIRECT + BLOCK / 4,
                "the init program is too big"
            );
            for (n, chunk) in chunks.iter().enumerate().take(DIRECT) {
                blocks[n] = alloc(&mut img, chunk);
            }
            used = chunks.len();
            if chunks.len() > DIRECT {
                let mut table = Vec::new();
                for chunk in &chunks[DIRECT..] {
                    table.extend_from_slice(
                        &u32::try_from(alloc(&mut img, chunk))
                            .unwrap_or(0)
                            .to_le_bytes(),
                    );
                }
                blocks[DIRECT] = alloc(&mut img, &table);
                used += 1;
            }
        }
        let at = ITABLE * BLOCK + (inode.number - 1) * INODE;
        let raw = &mut img[at..at + INODE];
        put16(raw, 0, usize::from(inode.mode));
        put32(raw, 4, size);
        put16(raw, 26, usize::from(inode.links));
        // The count is in 512 byte sectors and takes in the indirect block.
        put32(raw, 28, used * BLOCK / 512);
        for (n, b) in blocks.iter().enumerate() {
            put32(raw, 40 + 4 * n, *b);
        }
    }

    // Bit n of the block bitmap is block n + 1, as block 0 comes before the first group. Bit n of the inode bitmap is inode n + 1. The reserved inodes are marked used, and so are the bits past the end of each, so the kernel never hands them out.
    let used_blocks = next - 1;
    let used_inodes = nodes.iter().map(|n| n.number).max().unwrap_or(0);
    for bit in 0..BLOCK * 8 {
        if bit < used_blocks || bit >= BLOCKS - 1 {
            img[BMAP * BLOCK + bit / 8] |= 1 << (bit % 8);
        }
        if bit < used_inodes || bit >= INODES {
            img[IMAP * BLOCK + bit / 8] |= 1 << (bit % 8);
        }
    }
    let free_blocks = BLOCKS - 1 - used_blocks;
    let free_inodes = INODES - used_inodes;
    let dirs = nodes
        .iter()
        .filter(|n| matches!(n.node, Node::Dir(_)))
        .count();

    let gd = GDT * BLOCK;
    put32(&mut img, gd, BMAP);
    put32(&mut img, gd + 4, IMAP);
    put32(&mut img, gd + 8, ITABLE);
    put16(&mut img, gd + 12, free_blocks);
    put16(&mut img, gd + 14, free_inodes);
    put16(&mut img, gd + 16, dirs);

    let sb = BLOCK;
    put32(&mut img, sb, INODES);
    put32(&mut img, sb + 4, BLOCKS);
    put32(&mut img, sb + 12, free_blocks);
    put32(&mut img, sb + 16, free_inodes);
    put32(&mut img, sb + 20, 1);
    put32(&mut img, sb + 32, BLOCK * 8);
    put32(&mut img, sb + 36, BLOCK * 8);
    put32(&mut img, sb + 40, INODES);
    // No mount count limit, so no kernel asks for a check.
    put16(&mut img, sb + 54, 0xffff);
    put16(&mut img, sb + 56, usize::from(MAGIC));
    // The state is "cleanly unmounted", and errors are carried on with.
    put16(&mut img, sb + 58, 1);
    put16(&mut img, sb + 60, 1);
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le16(img: &[u8], at: usize) -> usize {
        usize::from(u16::from_le_bytes([img[at], img[at + 1]]))
    }

    fn le32(img: &[u8], at: usize) -> usize {
        u32::from_le_bytes([img[at], img[at + 1], img[at + 2], img[at + 3]]) as usize
    }

    fn inode(img: &[u8], n: usize) -> &[u8] {
        let at = ITABLE * BLOCK + (n - 1) * INODE;
        &img[at..at + INODE]
    }

    /// Read a file back the way the kernel does, through the direct blocks and the indirect one.
    fn read(img: &[u8], n: usize) -> Vec<u8> {
        let raw = inode(img, n);
        let size = le32(raw, 4);
        let mut blocks: Vec<usize> = (0..DIRECT).map(|i| le32(raw, 40 + 4 * i)).collect();
        let indirect = le32(raw, 40 + 4 * DIRECT);
        if indirect != 0 {
            blocks.extend((0..BLOCK / 4).map(|i| le32(img, indirect * BLOCK + 4 * i)));
        }
        let mut out = Vec::new();
        for b in blocks.into_iter().take(size.div_ceil(BLOCK)) {
            out.extend_from_slice(&img[b * BLOCK..(b + 1) * BLOCK]);
        }
        out.truncate(size);
        out
    }

    /// The names in a directory, following the record lengths to the end of its block.
    fn names(img: &[u8], n: usize) -> Vec<(String, usize)> {
        let block = le32(inode(img, n), 40) * BLOCK;
        let mut at = 0;
        let mut out = Vec::new();
        while at < BLOCK {
            let len = le16(img, block + at + 6);
            let name = &img[block + at + 8..block + at + 8 + len];
            out.push((
                String::from_utf8_lossy(name).into_owned(),
                le32(img, block + at),
            ));
            at += le16(img, block + at + 4);
        }
        assert_eq!(at, BLOCK);
        out
    }

    #[test]
    fn the_super_block_is_ext2_revision_0() {
        let img = image(b"init");
        assert_eq!(img.len(), BLOCKS * BLOCK);
        assert_eq!(le32(&img, BLOCK), INODES);
        assert_eq!(le32(&img, BLOCK + 4), BLOCKS);
        assert_eq!(le32(&img, BLOCK + 20), 1);
        assert_eq!(le16(&img, BLOCK + 56), 0xef53);
        assert_eq!(le32(&img, BLOCK + 76), 0);
    }

    #[test]
    fn init_reads_back_through_the_indirect_block() {
        let init: Vec<u8> = (0..20_000u32).map(|i| (i * 31 % 251) as u8).collect();
        let img = image(&init);
        assert_eq!(read(&img, 15), init);
        assert_ne!(le32(inode(&img, 15), 40 + 4 * DIRECT), 0);
        // 20 blocks of data and the indirect one, in sectors.
        assert_eq!(le32(inode(&img, 15), 28), 42);
    }

    #[test]
    fn the_root_names_init_and_dev() {
        let img = image(b"x");
        let root: Vec<String> = names(&img, 2).into_iter().map(|(n, _)| n).collect();
        assert_eq!(root, [".", "..", "dev", "proc", "sys", "tmp", "init"]);
        assert_eq!(
            names(&img, 11),
            [
                (".".into(), 11),
                ("..".into(), 2),
                ("console".into(), 16),
                ("null".into(), 17)
            ]
        );
        assert_eq!(le32(inode(&img, 16), 40), 0x0501);
        assert_eq!(le32(inode(&img, 17), 40), 0x0103);
    }

    #[test]
    fn the_counts_match_the_bitmaps() {
        let img = image(b"x");
        let set = |map: usize, bits: usize| {
            (0..bits)
                .filter(|n| img[map * BLOCK + n / 8] >> (n % 8) & 1 == 1)
                .count()
        };
        assert_eq!(BLOCKS - 1 - set(BMAP, BLOCKS - 1), le32(&img, BLOCK + 12));
        assert_eq!(INODES - set(IMAP, INODES), le32(&img, BLOCK + 16));
        assert_eq!(le16(&img, GDT * BLOCK + 12), le32(&img, BLOCK + 12));
        assert_eq!(le16(&img, GDT * BLOCK + 16), 5);
        // The metadata and one block for each of five directories and init.
        assert_eq!(set(BMAP, BLOCKS - 1), FIRST_DATA - 1 + 6);
    }

    #[test]
    fn the_image_depends_on_init_alone() {
        assert_eq!(image(b"same"), image(b"same"));
        assert_ne!(image(b"same"), image(b"other"));
    }
}
