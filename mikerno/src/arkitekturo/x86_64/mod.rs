pub mod apic;
pub mod eniro;
pub mod gdt;
pub mod idt;
pub mod mb2;
pub mod pagxado;
pub mod planado;
pub mod procesoro;
pub mod pvh;
pub mod seria;
pub mod temporizilo;
pub mod tss;

pub fn pravalorizi() {
    seria::pravalorizi();
    gdt::instali();
    idt::instali();
    pagxado::pravalorizi();
}
