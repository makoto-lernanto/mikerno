//! Arkitekture komunaj abstraktaĵoj de la fasado.
//!
//! Ĉi tiuj tipoj permesas al la nukleo pritrakti kadrojn kaj firmigajn
//! raportojn sen koni la platforman aranĝon de la registroj.
//!
//! A1 ne ankoraŭ uzas ilin sur `arm64`; ili estos ligitaj en A3/A4.

#![cfg_attr(target_arch = "aarch64", allow(dead_code))]

/// Speco de virtio-transporto: PCI (x86_64) aŭ MMIO (arm64).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VirtioSpeco {
    Pcia,
    /// Nur `arm64` konstruas ĉi tiun specon.
    #[cfg_attr(target_arch = "x86_64", allow(dead_code))]
    Mmio,
}

/// La modernaj (virtio 1.0) registroj de unu aparato.
///
/// La kampoj montras al jam mapita nekaŝebla MMIO. La dua duono de la
/// aliriloj legas kaj skribas per `volatile`, ĉar la aparato ŝanĝas ilin
/// sendepende de la procesoro.
pub struct VirtioTransporto {
    speco: VirtioSpeco,
    komuna: *mut u8,
    sciiga: *mut u8,
    aparata: *mut u8,
    sciiga_multobligo: u32,
}

mod pci_reg {
    pub const STATUS: u64 = 0x14;
    pub const DEVICE_FEATURE_SELECT: u64 = 0x00;
    pub const DEVICE_FEATURE: u64 = 0x04;
    pub const DRIVER_FEATURE_SELECT: u64 = 0x08;
    pub const DRIVER_FEATURE: u64 = 0x0c;
    pub const QUEUE_SELECT: u64 = 0x16;
    pub const QUEUE_SIZE: u64 = 0x18;
    pub const QUEUE_ENABLE: u64 = 0x1c;
    pub const QUEUE_NOTIFY_OFF: u64 = 0x1e;
    pub const QUEUE_DESC: u64 = 0x20;
    pub const QUEUE_DRIVER: u64 = 0x28;
    pub const QUEUE_DEVICE: u64 = 0x30;
}

mod mmio_reg {
    pub const DEVICE_FEATURE: u64 = 0x10;
    pub const DEVICE_FEATURE_SELECT: u64 = 0x14;
    pub const DRIVER_FEATURE: u64 = 0x20;
    pub const DRIVER_FEATURE_SELECT: u64 = 0x24;
    pub const QUEUE_SELECT: u64 = 0x30;
    pub const QUEUE_SIZE_MAX: u64 = 0x34;
    pub const QUEUE_SIZE: u64 = 0x38;
    pub const QUEUE_ENABLE: u64 = 0x44;
    pub const NOTIFY: u64 = 0x50;
    pub const STATUS: u64 = 0x70;
    pub const QUEUE_DESC: u64 = 0x80;
    pub const QUEUE_DRIVER: u64 = 0x90;
    pub const QUEUE_DEVICE: u64 = 0xa0;
    pub const DEVICE_CONFIG: u64 = 0x100;
}

impl VirtioTransporto {
    pub const fn nova_pcia(
        komuna: *mut u8,
        sciiga: *mut u8,
        aparata: *mut u8,
        sciiga_multobligo: u32,
    ) -> Self {
        Self {
            speco: VirtioSpeco::Pcia,
            komuna,
            sciiga,
            aparata,
            sciiga_multobligo,
        }
    }

    #[cfg_attr(target_arch = "x86_64", allow(dead_code))]
    pub const fn nova_mmio(bazo: *mut u8) -> Self {
        Self {
            speco: VirtioSpeco::Mmio,
            komuna: bazo,
            sciiga: bazo,
            aparata: bazo,
            sciiga_multobligo: 1,
        }
    }

    pub fn speco(&self) -> VirtioSpeco {
        self.speco
    }

    pub fn stato(&self) -> u8 {
        match self.speco {
            VirtioSpeco::Pcia => self.legi_u8(self.komuna, pci_reg::STATUS),
            // MMIO-registroj estas nur 32-bitaj; bajta aliro ne atingus la aparaton.
            VirtioSpeco::Mmio => self.legi_u32(self.komuna, mmio_reg::STATUS) as u8,
        }
    }

    pub fn agordi_staton(&self, valoro: u8) {
        match self.speco {
            VirtioSpeco::Pcia => self.skribi_u8(self.komuna, pci_reg::STATUS, valoro),
            VirtioSpeco::Mmio => {
                self.skribi_u32(self.komuna, mmio_reg::STATUS, valoro as u32);
            }
        }
    }

    pub fn legi_trajtojn(&self, paĝo: u32) -> u32 {
        match self.speco {
            VirtioSpeco::Pcia => {
                self.skribi_u32(self.komuna, pci_reg::DEVICE_FEATURE_SELECT, paĝo);
                self.legi_u32(self.komuna, pci_reg::DEVICE_FEATURE)
            }
            VirtioSpeco::Mmio => {
                self.skribi_u32(self.komuna, mmio_reg::DEVICE_FEATURE_SELECT, paĝo);
                self.legi_u32(self.komuna, mmio_reg::DEVICE_FEATURE)
            }
        }
    }

    pub fn agordi_trajtojn(&self, paĝo: u32, valoro: u32) {
        match self.speco {
            VirtioSpeco::Pcia => {
                self.skribi_u32(self.komuna, pci_reg::DRIVER_FEATURE_SELECT, paĝo);
                self.skribi_u32(self.komuna, pci_reg::DRIVER_FEATURE, valoro);
            }
            VirtioSpeco::Mmio => {
                self.skribi_u32(self.komuna, mmio_reg::DRIVER_FEATURE_SELECT, paĝo);
                self.skribi_u32(self.komuna, mmio_reg::DRIVER_FEATURE, valoro);
            }
        }
    }

    pub fn elekti_vicon(&self, vico: u16) {
        match self.speco {
            VirtioSpeco::Pcia => {
                self.skribi_u16(self.komuna, pci_reg::QUEUE_SELECT, vico);
            }
            VirtioSpeco::Mmio => {
                self.skribi_u32(self.komuna, mmio_reg::QUEUE_SELECT, vico as u32);
            }
        }
    }

    pub fn vica_grando(&self) -> u16 {
        match self.speco {
            VirtioSpeco::Pcia => self.legi_u16(self.komuna, pci_reg::QUEUE_SIZE),
            VirtioSpeco::Mmio => self.legi_u32(self.komuna, mmio_reg::QUEUE_SIZE_MAX) as u16,
        }
    }

    pub fn agordi_vican_grandon(&self, grandeco: u16) {
        match self.speco {
            VirtioSpeco::Pcia => {
                self.skribi_u16(self.komuna, pci_reg::QUEUE_SIZE, grandeco);
            }
            VirtioSpeco::Mmio => {
                self.skribi_u32(self.komuna, mmio_reg::QUEUE_SIZE, grandeco as u32);
            }
        }
    }

    pub fn vica_sciiga_offsets(&self) -> u16 {
        match self.speco {
            VirtioSpeco::Pcia => self.legi_u16(self.komuna, pci_reg::QUEUE_NOTIFY_OFF),
            // MMIO sciigas per la vica numero mem.
            VirtioSpeco::Mmio => 0,
        }
    }

    pub fn agordi_vicajn_adresojn(&self, desc: u64, disponebla: u64, uzita: u64) {
        match self.speco {
            VirtioSpeco::Pcia => {
                self.skribi_u64(self.komuna, pci_reg::QUEUE_DESC, desc);
                self.skribi_u64(self.komuna, pci_reg::QUEUE_DRIVER, disponebla);
                self.skribi_u64(self.komuna, pci_reg::QUEUE_DEVICE, uzita);
            }
            VirtioSpeco::Mmio => {
                self.skribi_u32(self.komuna, mmio_reg::QUEUE_DESC, desc as u32);
                self.skribi_u32(self.komuna, mmio_reg::QUEUE_DESC + 4, (desc >> 32) as u32);
                self.skribi_u32(self.komuna, mmio_reg::QUEUE_DRIVER, disponebla as u32);
                self.skribi_u32(
                    self.komuna,
                    mmio_reg::QUEUE_DRIVER + 4,
                    (disponebla >> 32) as u32,
                );
                self.skribi_u32(self.komuna, mmio_reg::QUEUE_DEVICE, uzita as u32);
                self.skribi_u32(
                    self.komuna,
                    mmio_reg::QUEUE_DEVICE + 4,
                    (uzita >> 32) as u32,
                );
            }
        }
    }

    pub fn ebligi_vicon(&self) {
        match self.speco {
            VirtioSpeco::Pcia => {
                self.skribi_u16(self.komuna, pci_reg::QUEUE_ENABLE, 1);
            }
            VirtioSpeco::Mmio => {
                self.skribi_u32(self.komuna, mmio_reg::QUEUE_ENABLE, 1);
            }
        }
    }

    pub fn sciigi(&self, vico: u16) {
        match self.speco {
            VirtioSpeco::Pcia => {
                let offset = self.vica_sciiga_offsets() as u64 * self.sciiga_multobligo as u64;
                self.skribi_u16(self.sciiga, offset, vico);
            }
            VirtioSpeco::Mmio => {
                self.skribi_u32(self.komuna, mmio_reg::NOTIFY, vico as u32);
            }
        }
    }

    /// La kapacito de la aparato en 512-bajtaj sektoroj.
    pub fn kapacito(&self) -> u64 {
        let bazo = match self.speco {
            VirtioSpeco::Pcia => 0,
            VirtioSpeco::Mmio => mmio_reg::DEVICE_CONFIG,
        };
        let malalta = self.legi_u32(self.aparata, bazo) as u64;
        let alta = self.legi_u32(self.aparata, bazo + 4) as u64;
        malalta | (alta << 32)
    }

    fn legi_u8(&self, bazo: *mut u8, offset: u64) -> u8 {
        unsafe { core::ptr::read_volatile(bazo.add(offset as usize)) }
    }

    fn legi_u16(&self, bazo: *mut u8, offset: u64) -> u16 {
        unsafe { core::ptr::read_volatile(bazo.add(offset as usize).cast::<u16>()) }
    }

    fn legi_u32(&self, bazo: *mut u8, offset: u64) -> u32 {
        unsafe { core::ptr::read_volatile(bazo.add(offset as usize).cast::<u32>()) }
    }

    fn skribi_u8(&self, bazo: *mut u8, offset: u64, valoro: u8) {
        unsafe { core::ptr::write_volatile(bazo.add(offset as usize), valoro) }
    }

    fn skribi_u16(&self, bazo: *mut u8, offset: u64, valoro: u16) {
        unsafe { core::ptr::write_volatile(bazo.add(offset as usize).cast::<u16>(), valoro) }
    }

    fn skribi_u32(&self, bazo: *mut u8, offset: u64, valoro: u32) {
        unsafe { core::ptr::write_volatile(bazo.add(offset as usize).cast::<u32>(), valoro) }
    }

    fn skribi_u64(&self, bazo: *mut u8, offset: u64, valoro: u64) {
        unsafe { core::ptr::write_volatile(bazo.add(offset as usize).cast::<u64>(), valoro) }
    }
}

/// Speco de procesora escepto, rekonita de la malantaŭo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EsceptoTipo {
    PaĝaEraro,
    ProtektaEraro,
    Nekonata(u64),
}

/// Aliro al la konservita interrompa kadro sen koni la platformajn kampojn.
pub trait SistemvokaKadro {
    /// La sistemvoka numero aŭ, por aliaj esceptoj, la vektoro.
    fn numero(&self) -> usize;
    /// La `indekso`-a argumento (0–3).
    fn argumento(&self, indekso: usize) -> u64;
    /// Skribu la rezulton reen en la kadron.
    fn agordi_rezulton(&mut self, rezulto: u64);
    /// Ĉu la escepto venis el uzanta reĝimo?
    fn el_uzanto(&self) -> bool;
    /// La platforma vektoro aŭ interrompa identigilo.
    fn vektoro(&self) -> u64;
    /// La platforma erarkodo (aŭ `0` se la platformo ne havas unu).
    fn erarkodo(&self) -> u64;
    /// La adreso de la interrompita instrukcio.
    fn instrukcipunkto(&self) -> u64;
    /// La stakmontrilo de la interrompita kunteksto.
    fn stakmontrilo(&self) -> u64;
    /// La procesora stata registro (aŭ ĝia platforma ekvivalento).
    fn stato(&self) -> u64;
}

/// Unu firmiga trajto raportata ĉe la starto.
#[derive(Clone, Copy)]
pub struct FirmigaTrajto {
    pub nomo: &'static str,
    pub subtenata: bool,
    pub aktiva: bool,
    pub priskribo: &'static str,
}

impl FirmigaTrajto {
    const MALPLENA: Self = Self {
        nomo: "",
        subtenata: false,
        aktiva: false,
        priskribo: "",
    };
}

pub const MAKSAJ_FIRMIGAJ_TRAJTOJ: usize = 4;

/// La firmigaj trajtoj de la aktiva platformo.
pub struct FirmigaRaporto {
    trajtoj: [FirmigaTrajto; MAKSAJ_FIRMIGAJ_TRAJTOJ],
    nombro: usize,
}

impl FirmigaRaporto {
    pub const fn nova() -> Self {
        Self {
            trajtoj: [FirmigaTrajto::MALPLENA; MAKSAJ_FIRMIGAJ_TRAJTOJ],
            nombro: 0,
        }
    }

    pub fn aldonu(&mut self, trajto: FirmigaTrajto) {
        if self.nombro < self.trajtoj.len() {
            self.trajtoj[self.nombro] = trajto;
            self.nombro += 1;
        }
    }

    pub fn trajtoj(&self) -> &[FirmigaTrajto] {
        &self.trajtoj[..self.nombro]
    }
}
