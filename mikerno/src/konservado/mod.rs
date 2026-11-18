//! Kurna konservado (blokaj enhavo, virtio-blk).
//!
//! ĉi tiu modulo subtenas `virtio-blk` pelilon por QEMU: x86_64 uzas la PCI
//! transporton, `arm64` la MMIO-on.

pub mod snds;
pub mod virtio_blk;

pub use virtio_blk::pravalorizi;
