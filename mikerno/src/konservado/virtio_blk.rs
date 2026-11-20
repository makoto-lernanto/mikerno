//! Virtio-blk: vicoj, DMA-bufroj kaj la bloka alirilo.
//!
//! La aparato estas la moderna (virtio 1.0) modelo: `PCI` sur `x86_64` kaj
//! `MMIO` sur `arm64`. La vicoj kaj la petbufro venas el la kerna
//! kadro-alokilo; la aparato atingas ilin per la identeca mapo.

use core::fmt;
use core::sync::atomic::{Ordering, fence};

use crate::arkitekturo::komuna::{VirtioSpeco, VirtioTransporto};
use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::KADRA_GRANDO;
use crate::memoro::kadro::{KadraAlokilo, Kadro};
use crate::memoro::virtuala::AdresaSpaco;

const VICO_GRANDO: u16 = 8;
const VICO_KADROJ: usize = 4;
const OFSETO_DESC: u64 = 0x000;
const OFSETO_DISPONEBLA: u64 = 0x100;
const OFSETO_UZITA: u64 = 0x200;
const OFSETO_PETO: u64 = 0x1000;

const STATUS_ACK: u8 = 1;
const STATUS_DRIVER: u8 = 2;
const STATUS_DRIVER_OK: u8 = 4;
const STATUS_FEATURES_OK: u8 = 8;
const STATUS_FAILED: u8 = 128;

const FLAGO_SEKVA: u16 = 1;
const FLAGO_SKRIBU: u16 = 2;

const BLOKA_GRANDO: usize = 512;
const HAŜETA_BAZO: u64 = 0xcbf2_9ce4_8422_2325;
const HAŜETA_MULTIGILO: u64 = 0x0000_0100_0000_01b3;
const ATENDA_LIMO: u32 = 50_000_000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Eraro {
    Transporto(&'static str),
    NeKomencigita,
    Memoro,
    NevalidaSektoro,
    Aparato(u8),
    Tempolimo,
}

impl fmt::Display for Eraro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eraro::Transporto(kaŭzo) => write!(f, "transporto: {kaŭzo}"),
            Eraro::NeKomencigita => write!(f, "la pelilo ne estas komencigita"),
            Eraro::Memoro => write!(f, "la kadro-alokilo elĉerpiĝis"),
            Eraro::NevalidaSektoro => write!(f, "sektoro ekster la kapacito"),
            Eraro::Aparato(stato) => write!(f, "la aparato redonis staton {stato}"),
            Eraro::Tempolimo => write!(f, "la aparato ne finis la peton"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BlkKapo {
    tipo: u32,
    rezervita: u32,
    sektoro: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Priskribilo {
    adreso: u64,
    longo: u32,
    flagoj: u16,
    sekva: u16,
}

#[repr(C)]
struct Disponebla {
    flagoj: u16,
    indekso: u16,
    ringo: [u16; VICO_GRANDO as usize],
}

#[repr(C)]
struct UzitaEro {
    identigilo: u32,
    longo: u32,
}

#[repr(C)]
struct Uzita {
    flagoj: u16,
    indekso: u16,
    ringo: [UzitaEro; VICO_GRANDO as usize],
}

#[repr(C, align(512))]
struct Peto {
    kapo: BlkKapo,
    stato: u8,
    _spaco: [u8; 495],
    datumoj: [u8; BLOKA_GRANDO],
}

struct Vico {
    bazo: u64,
    indekso: u16,
    lasta: u16,
}

impl Vico {
    fn nova(bazo: u64) -> Self {
        Self {
            bazo,
            indekso: 0,
            lasta: 0,
        }
    }

    fn peto(&self) -> *mut Peto {
        (self.bazo + OFSETO_PETO) as *mut Peto
    }

    fn skribi_priskribilon(
        &self,
        indekso: usize,
        adreso: u64,
        longo: u32,
        flagoj: u16,
        sekva: u16,
    ) {
        let priskribilo = Priskribilo {
            adreso,
            longo,
            flagoj,
            sekva,
        };
        let celo = (self.bazo + OFSETO_DESC) as *mut Priskribilo;
        unsafe { core::ptr::write_volatile(celo.add(indekso), priskribilo) }
    }

    fn sendi(&mut self, trafiko: &VirtioTransporto, skriba: bool) -> Result<(), Eraro> {
        let peto = self.peto() as u64;
        self.skribi_priskribilon(
            0,
            peto,
            core::mem::size_of::<BlkKapo>() as u32,
            FLAGO_SEKVA,
            1,
        );
        let datumaj_flagoj = FLAGO_SEKVA | if skriba { 0 } else { FLAGO_SKRIBU };
        self.skribi_priskribilon(1, peto + 512, BLOKA_GRANDO as u32, datumaj_flagoj, 2);
        self.skribi_priskribilon(2, peto + 16, 1, FLAGO_SKRIBU, 0);

        let disponebla = (self.bazo + OFSETO_DISPONEBLA) as *mut Disponebla;
        let slot = (self.indekso as usize) & (VICO_GRANDO as usize - 1);
        unsafe {
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*disponebla).ringo[slot]), 0);
        }
        fence(Ordering::SeqCst);
        unsafe {
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*disponebla).indekso),
                self.indekso.wrapping_add(1),
            );
        }
        self.indekso = self.indekso.wrapping_add(1);
        fence(Ordering::SeqCst);
        trafiko.sciigi(0);

        let uzita = (self.bazo + OFSETO_UZITA) as *const Uzita;
        let mut atendoj = 0u32;
        loop {
            let indekso =
                unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*uzita).indekso)) };
            if indekso != self.lasta {
                break;
            }
            atendoj += 1;
            if atendoj >= ATENDA_LIMO {
                return Err(Eraro::Tempolimo);
            }
            core::hint::spin_loop();
        }
        fence(Ordering::SeqCst);

        let ero = unsafe {
            core::ptr::read_volatile(core::ptr::addr_of!(
                (*uzita).ringo[(self.lasta as usize) & (VICO_GRANDO as usize - 1)]
            ))
        };
        self.lasta = self.lasta.wrapping_add(1);
        if ero.identigilo != 0 {
            return Err(Eraro::Aparato(0xff));
        }

        let stato = unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*self.peto()).stato)) };
        if stato != 0 {
            return Err(Eraro::Aparato(stato));
        }
        Ok(())
    }
}

pub struct Konservado {
    trafiko: VirtioTransporto,
    vico: Vico,
    kapacito: u64,
    _kadroj: Kadro,
}

static KONSERVADO: Kunhavigita<Option<Konservado>> = Kunhavigita::nova(None);

fn nuligi(bazo: Kadro, kadroj: usize) {
    let longo = KADRA_GRANDO as usize * kadroj;
    unsafe { core::ptr::write_bytes(bazo.adreso() as *mut u8, 0, longo) }
}

impl Konservado {
    fn nova(trafiko: VirtioTransporto, alokilo: &mut KadraAlokilo) -> Result<Self, Eraro> {
        trafiko.agordi_staton(0);
        for _ in 0..1000 {
            if trafiko.stato() == 0 {
                break;
            }
        }
        if trafiko.stato() != 0 {
            return Err(Eraro::Transporto("la aparato ne restariĝis"));
        }

        trafiko.agordi_staton(STATUS_ACK);
        trafiko.agordi_staton(STATUS_ACK | STATUS_DRIVER);
        if trafiko.legi_trajtojn(1) & 1 == 0 {
            return Err(Eraro::Transporto("la aparato ne ofertas virtio 1.0"));
        }
        trafiko.agordi_trajtojn(0, 0);
        trafiko.agordi_trajtojn(1, 1);
        trafiko.agordi_staton(STATUS_ACK | STATUS_DRIVER | STATUS_FEATURES_OK);
        let stato = trafiko.stato();
        if stato & STATUS_FAILED != 0 || stato & STATUS_FEATURES_OK == 0 {
            return Err(Eraro::Transporto("la trajta interkonsento malsukcesis"));
        }

        trafiko.elekti_vicon(0);
        if trafiko.vica_grando() < VICO_GRANDO {
            return Err(Eraro::Transporto("la unua vico estas tro malgranda"));
        }
        trafiko.agordi_vican_grandon(VICO_GRANDO);

        let kadro = alokilo.aloki_kontiguajn(VICO_KADROJ).ok_or(Eraro::Memoro)?;
        nuligi(kadro, VICO_KADROJ);
        let bazo = kadro.adreso();
        trafiko.agordi_vicajn_adresojn(
            bazo + OFSETO_DESC,
            bazo + OFSETO_DISPONEBLA,
            bazo + OFSETO_UZITA,
        );
        trafiko.ebligi_vicon();
        trafiko.agordi_staton(STATUS_ACK | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK);

        let kapacito = trafiko.kapacito();
        Ok(Self {
            trafiko,
            vico: Vico::nova(bazo),
            kapacito,
            _kadroj: kadro,
        })
    }

    fn legi(&mut self, sektoro: u64, bufro: &mut [u8; BLOKA_GRANDO]) -> Result<(), Eraro> {
        if sektoro >= self.kapacito {
            return Err(Eraro::NevalidaSektoro);
        }
        let peto = self.vico.peto();
        unsafe {
            core::ptr::write(
                core::ptr::addr_of_mut!((*peto).kapo),
                BlkKapo {
                    tipo: 0,
                    rezervita: 0,
                    sektoro,
                },
            );
            core::ptr::write(core::ptr::addr_of_mut!((*peto).stato), 0xff);
        }
        fence(Ordering::SeqCst);
        self.vico.sendi(&self.trafiko, false)?;
        unsafe {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*peto).datumoj) as *const u8,
                bufro.as_mut_ptr(),
                BLOKA_GRANDO,
            );
        }
        Ok(())
    }

    fn skribi(&mut self, sektoro: u64, bufro: &[u8; BLOKA_GRANDO]) -> Result<(), Eraro> {
        if sektoro >= self.kapacito {
            return Err(Eraro::NevalidaSektoro);
        }
        let peto = self.vico.peto();
        unsafe {
            core::ptr::write(
                core::ptr::addr_of_mut!((*peto).kapo),
                BlkKapo {
                    tipo: 1,
                    rezervita: 0,
                    sektoro,
                },
            );
            core::ptr::write(core::ptr::addr_of_mut!((*peto).stato), 0xff);
            core::ptr::copy_nonoverlapping(
                bufro.as_ptr(),
                core::ptr::addr_of_mut!((*peto).datumoj) as *mut u8,
                BLOKA_GRANDO,
            );
        }
        fence(Ordering::SeqCst);
        self.vico.sendi(&self.trafiko, true)
    }
}

/// Legu unu 512-bajtan sektoron en `bufro`.
pub fn legi_blokon(sektoro: u64, bufro: &mut [u8; BLOKA_GRANDO]) -> Result<(), Eraro> {
    let _gardo = InterrompaGardo::nova();
    let Some(konservado) = (unsafe { (*KONSERVADO.akiri()).as_mut() }) else {
        return Err(Eraro::NeKomencigita);
    };
    konservado.legi(sektoro, bufro)
}

/// Skribu unu 512-bajtan sektoron el `bufro`.
pub fn skribi_blokon(sektoro: u64, bufro: &[u8; BLOKA_GRANDO]) -> Result<(), Eraro> {
    let _gardo = InterrompaGardo::nova();
    let Some(konservado) = (unsafe { (*KONSERVADO.akiri()).as_mut() }) else {
        return Err(Eraro::NeKomencigita);
    };
    konservado.skribi(sektoro, bufro)
}

/// La kapacito de la aparato en sektoroj (0 se la pelilo mankas).
pub fn kapacito() -> u64 {
    let _gardo = InterrompaGardo::nova();
    unsafe { (*KONSERVADO.akiri()).as_ref() }.map_or(0, |konservado| konservado.kapacito)
}

fn haketi(bufro: &[u8; BLOKA_GRANDO], haketo: &mut u64) {
    for &bajto in bufro {
        *haketo ^= bajto as u64;
        *haketo = haketo.wrapping_mul(HAŜETA_MULTIGILO);
    }
}

fn memprovo() -> Result<(), Eraro> {
    let mut bufro = [0u8; BLOKA_GRANDO];
    let mut haketo = HAŜETA_BAZO;
    let kapacito = kapacito();
    if kapacito < 64 {
        return Err(Eraro::Transporto("la kapacito estas malpli ol 64 sektoroj"));
    }
    for sektoro in 0..64 {
        legi_blokon(sektoro, &mut bufro)?;
        haketi(&bufro, &mut haketo);
    }
    crate::skribu!(
        "[konservado] virtio-blk: 64 sektoroj legitaj, haketo 0x{:016x}\n",
        haketo
    );

    let originalo = bufro;
    let lasta = kapacito - 1;
    for (indekso, bajto) in bufro.iter_mut().enumerate() {
        *bajto = (indekso as u8).wrapping_mul(37).wrapping_add(0x5a);
    }
    skribi_blokon(lasta, &bufro)?;
    let mut kontrolo = [0u8; BLOKA_GRANDO];
    legi_blokon(lasta, &mut kontrolo)?;
    if kontrolo != bufro {
        return Err(Eraro::Transporto("la skribo ne legeblas reen"));
    }
    skribi_blokon(lasta, &originalo)?;
    legi_blokon(lasta, &mut kontrolo)?;
    if kontrolo != originalo {
        return Err(Eraro::Transporto("la restarigo malsukcesis"));
    }
    crate::skribu!("[konservado] virtio-blk: skribo kaj restarigo (bone)\n");
    Ok(())
}

/// Trovu, komencigu kaj memprovi la virtio-blk-aparaton.
pub fn pravalorizi(spaco: &mut dyn AdresaSpaco, alokilo: &mut KadraAlokilo) -> Result<(), Eraro> {
    let trafiko =
        crate::arkitekturo::virtio_blk_transporto(spaco, alokilo).map_err(Eraro::Transporto)?;
    let speco = match trafiko.speco() {
        #[cfg(target_arch = "x86_64")]
        VirtioSpeco::Pcia => "PCI",
        #[cfg(target_arch = "aarch64")]
        VirtioSpeco::Mmio => "MMIO",
    };
    let konservado = Konservado::nova(trafiko, alokilo)?;
    crate::skribu!(
        "[konservado] virtio-blk ({speco}): {} sektoroj, {} bajtoj po sektoro\n",
        konservado.kapacito,
        BLOKA_GRANDO
    );
    {
        let _gardo = InterrompaGardo::nova();
        unsafe { *KONSERVADO.akiri() = Some(konservado) };
    }
    memprovo()
}
