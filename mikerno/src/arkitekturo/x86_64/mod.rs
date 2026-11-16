pub mod apic;
pub mod eniro;
pub mod firmigo;
pub mod gdt;
pub mod hazardigo;
pub mod idt;
pub mod interrompo;
pub mod mb2;
pub mod pagxado;
pub mod pci;
pub mod planado;
pub mod procesoro;
pub mod pvh;
pub mod seria;
pub mod sistemvoko;
pub mod temporizilo;
pub mod tss;
pub mod uzanta;
pub mod virtio_pci;

pub fn pravalorizi() {
    seria::pravalorizi();
    gdt::instali();
    idt::instali();
    pagxado::pravalorizi();
    firmigo::pravalorizi();
}
