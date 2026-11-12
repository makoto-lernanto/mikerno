//! Uzantspaca aranĝo de `horizonto`: la ELF-bildo super 4 GiB, la 64 KiB-a
//! stako kaj la memtesto de la bito `U`.
//!
//! La bildo venas kiel praŝarĝa modulo (Multiboot2 tipo 3 aŭ PVH-modlisto) kaj
//! estas mapita en la fenestron `[UZANTA_BAZO, UZANTA_FINO)`. La fenestro kuŝas
//! tute super la identeca mapo de 4 GiB, do kerna kaj uzanta adresoj neniam
//! kongruas.

use core::fmt;
use core::ptr;
use core::slice;

use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::kadro::KadraAlokilo;
use crate::memoro::virtuala::{
    AdresaSpaco, FizikaAdreso, PaĝaEraro, PaĝaPermesoj, UZANTA_BAZO, UZANTA_FINO,
    UZANTA_STAKO_GRANDE, UZANTA_STAKO_SUPRO, VirtuaAdreso,
};
use crate::memoro::{KADRA_GRANDO, Modulo};

const ELF_MAGIO: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const ELF_KLASO_64: u8 = 2;
const ELF_DATUMOJ_LE: u8 = 1;
const ELF_MAŜINO_X86_64: u16 = 62;
const ELF_KAPA_GRANDO: usize = 64;
const ELF_PROGRAMA_GRANDO: usize = 56;
const PT_LOAD: u32 = 1;
const PF_X: u32 = 1;
const PF_W: u32 = 2;
const MAKSAJ_SEGMENTOJ: usize = 16;

/// La provaj paĝoj de la memtesto, fore de la bildo kaj de la stako.
const PROVA_BAZO: u64 = UZANTA_FINO - 16 * 1024 * 1024;

static BILDA_LIMOJ: Kunhavigita<(u64, u64)> = Kunhavigita::nova((UZANTA_BAZO, UZANTA_BAZO));
static BILDA_ENIRPUNKTO: Kunhavigita<u64> = Kunhavigita::nova(0);

/// La mapita bilda intervalo `(komenco, fino)`, se `horizonto` jam ŝarĝis.
pub fn bildaj_limoj() -> (u64, u64) {
    *BILDA_LIMOJ.akiri()
}

/// La ELF-enirpunkto de la ŝargita bildo, se ĝi jam estas muntita.
pub fn enirpunkto() -> Option<u64> {
    let enirpunkto = *BILDA_ENIRPUNKTO.akiri();
    (enirpunkto != 0).then_some(enirpunkto)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ŜargaEraro {
    TroMalgranda,
    NevalidaElfo,
    EksterFenestro,
    NeniuSegmento,
    Paĝa(PaĝaEraro),
}

impl fmt::Display for ŜargaEraro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TroMalgranda => f.write_str("la modulo estas pli mallonga ol ELF-kapo"),
            Self::NevalidaElfo => f.write_str("nevalida ELF64-bildo"),
            Self::EksterFenestro => f.write_str("segmento eliras la uzantan fenestron"),
            Self::NeniuSegmento => f.write_str("la bildo havas neniun PT_LOAD-segmenton"),
            Self::Paĝa(eraro) => write!(f, "paĝa eraro: {eraro}"),
        }
    }
}

impl From<PaĝaEraro> for ŜargaEraro {
    fn from(eraro: PaĝaEraro) -> Self {
        Self::Paĝa(eraro)
    }
}

pub struct UzantaBildo {
    pub komenco: u64,
    pub fino: u64,
    pub enirpunkto: u64,
}

/// Ŝarĝu la bildon, pretigu la stakon kaj rulu la memteston.
pub fn pravalorizi(
    spaco: &mut impl AdresaSpaco,
    alokilo: &mut KadraAlokilo,
    modulo: &Modulo,
) -> Result<(), ŜargaEraro> {
    let bildo = ŝargi_bildon(modulo, spaco, alokilo)?;
    *BILDA_LIMOJ.akiri() = (bildo.komenco, bildo.fino);
    *BILDA_ENIRPUNKTO.akiri() = bildo.enirpunkto;
    prepari_stakon(spaco, alokilo)?;
    crate::skribu!(
        "[uzanto] horizonto: bildo 0x{:x}..0x{:x}, enirpunkto 0x{:x}\n",
        bildo.komenco,
        bildo.fino,
        bildo.enirpunkto
    );
    let staka_komenco = UZANTA_STAKO_SUPRO - UZANTA_STAKO_GRANDE as u64;
    crate::skribu!(
        "[uzanto] stako 0x{:x}..0x{:x} ({} KiB), pinto 0x{:x}\n",
        staka_komenco,
        UZANTA_STAKO_SUPRO,
        UZANTA_STAKO_GRANDE / 1024,
        UZANTA_STAKO_SUPRO
    );
    testi_kontrolon(spaco, alokilo, &bildo)?;
    Ok(())
}

/// Mapu la 64 KiB-an uzantan stakon, kiun Ring 3 uzos post `iretq`.
fn prepari_stakon(
    spaco: &mut impl AdresaSpaco,
    alokilo: &mut KadraAlokilo,
) -> Result<(), PaĝaEraro> {
    let komenco = UZANTA_STAKO_SUPRO - UZANTA_STAKO_GRANDE as u64;
    spaco.mapi_uzantan_regionon(
        VirtuaAdreso::nova(komenco),
        UZANTA_STAKO_GRANDE,
        PaĝaPermesoj::UZANTA_DATUMOJ,
        alokilo,
    )
}

/// Legu la ELF64-kapon, mapu ĉiun `PT_LOAD`-segmenton kaj redonu la bildan
/// intervalon kun la enirpunkto el la ELF-kapo.
fn ŝargi_bildon(
    modulo: &Modulo,
    spaco: &mut impl AdresaSpaco,
    alokilo: &mut KadraAlokilo,
) -> Result<UzantaBildo, ŜargaEraro> {
    if modulo.longo < ELF_KAPA_GRANDO as u64 {
        return Err(ŜargaEraro::TroMalgranda);
    }
    let bazo = modulo.komenco;
    // SAFETY: la modulo estas identece mapita kaj almenaŭ `ELF_KAPA_GRANDO`
    // bajtojn longa, do la tuta ELF-kapo estas legiva.
    let kapo = unsafe { slice::from_raw_parts(bazo as *const u8, ELF_KAPA_GRANDO) };
    if kapo[0..4] != ELF_MAGIO
        || kapo[4] != ELF_KLASO_64
        || kapo[5] != ELF_DATUMOJ_LE
        || legi_u16(kapo, 18) != ELF_MAŜINO_X86_64
        || legi_u16(kapo, 54) as usize != ELF_PROGRAMA_GRANDO
        || legi_u16(kapo, 56) == 0
    {
        return Err(ŜargaEraro::NevalidaElfo);
    }

    let enirpunkto = legi_u64(kapo, 24);
    let programa_komenco = legi_u64(kapo, 32);
    let programa_nombro = legi_u16(kapo, 56) as u64;
    let programa_fino = programa_komenco + programa_nombro * ELF_PROGRAMA_GRANDO as u64;
    if programa_fino > modulo.longo {
        return Err(ŜargaEraro::NevalidaElfo);
    }

    let mut segmentoj = [Segmento::MALPLENA; MAKSAJ_SEGMENTOJ];
    let mut segmenta_nombro = 0;
    let mut bilda_komenco = u64::MAX;
    let mut bilda_fino = 0;
    for indekso in 0..programa_nombro {
        let ero = bazo + programa_komenco + indekso * ELF_PROGRAMA_GRANDO as u64;
        // SAFETY: la programkapa tabelo restas ene de la identece mapita
        // modulo (kontrolite per `programa_fino`), do ĉiu ero estas legiva.
        let ero = unsafe { slice::from_raw_parts(ero as *const u8, ELF_PROGRAMA_GRANDO) };
        if legi_u32(ero, 0) != PT_LOAD {
            continue;
        }
        let flagoj = legi_u32(ero, 4);
        let dosiera = legi_u64(ero, 8);
        let virtuala = legi_u64(ero, 16);
        let dosiera_longo = legi_u64(ero, 32);
        let memora_longo = legi_u64(ero, 40);
        if memora_longo == 0 {
            continue;
        }
        let Some(dosiera_fino) = dosiera.checked_add(dosiera_longo) else {
            return Err(ŜargaEraro::NevalidaElfo);
        };
        if dosiera_longo > memora_longo || dosiera_fino > modulo.longo {
            return Err(ŜargaEraro::NevalidaElfo);
        }
        let Some(virtuala_fino) = virtuala.checked_add(memora_longo) else {
            return Err(ŜargaEraro::NevalidaElfo);
        };
        if virtuala < UZANTA_BAZO || virtuala_fino > UZANTA_FINO {
            return Err(ŜargaEraro::EksterFenestro);
        }
        if segmenta_nombro == MAKSAJ_SEGMENTOJ {
            return Err(ŜargaEraro::NevalidaElfo);
        }
        segmentoj[segmenta_nombro] = Segmento {
            virtuala,
            virtuala_fino,
            dosiero: bazo + dosiera,
            dosiera_longo,
            flagoj,
        };
        segmenta_nombro += 1;
        bilda_komenco = bilda_komenco.min(virtuala);
        bilda_fino = bilda_fino.max(virtuala_fino);
    }
    if segmenta_nombro == 0 {
        return Err(ŜargaEraro::NeniuSegmento);
    }
    if enirpunkto < bilda_komenco || enirpunkto >= bilda_fino {
        return Err(ŜargaEraro::NevalidaElfo);
    }

    mapigi_bildon(
        &segmentoj[..segmenta_nombro],
        bilda_komenco,
        bilda_fino,
        spaco,
        alokilo,
    )?;

    Ok(UzantaBildo {
        komenco: bilda_komenco,
        fino: bilda_fino,
        enirpunkto,
    })
}

/// Unu `PT_LOAD`-segmento ene de la identeca mapo de la modulo.
#[derive(Clone, Copy)]
struct Segmento {
    virtuala: u64,
    virtuala_fino: u64,
    /// La fizika komenco de la dosieraj bajtoj (identece mapita).
    dosiero: u64,
    dosiera_longo: u64,
    flagoj: u32,
}

impl Segmento {
    const MALPLENA: Self = Self {
        virtuala: 0,
        virtuala_fino: 0,
        dosiero: 0,
        dosiera_longo: 0,
        flagoj: 0,
    };
}

/// Mapu la bildon paĝon post paĝo. Pluraj segmentoj povas kunhavi paĝon
/// (la ligilo pakas ilin), do la permesoj estas la kunaĵo por ĉiu paĝo.
fn mapigi_bildon(
    segmentoj: &[Segmento],
    bilda_komenco: u64,
    bilda_fino: u64,
    spaco: &mut impl AdresaSpaco,
    alokilo: &mut KadraAlokilo,
) -> Result<(), ŜargaEraro> {
    let mut paĝo = bilda_komenco & !(KADRA_GRANDO - 1);
    let fino = bilda_fino.div_ceil(KADRA_GRANDO) * KADRA_GRANDO;
    while paĝo < fino {
        let mut skribebla = false;
        let mut plenumebla = false;
        for segmento in segmentoj {
            if segmento.virtuala < paĝo + KADRA_GRANDO && segmento.virtuala_fino > paĝo {
                skribebla |= segmento.flagoj & PF_W != 0;
                plenumebla |= segmento.flagoj & PF_X != 0;
            }
        }
        let virtuala_paĝo = VirtuaAdreso::nova(paĝo);
        if spaco.traduki(virtuala_paĝo).is_some() {
            return Err(ŜargaEraro::Paĝa(PaĝaEraro::Konflikto));
        }
        let kadro = alokilo.aloki().ok_or(PaĝaEraro::Elĉerpita)?;
        // SAFETY: la alokilo liveras kadron en la identeca mapo de la unuaj
        // 4 GiB, do la nukleo povas nuligi la tutan kadron per kruda montrilo.
        unsafe {
            ptr::write_bytes(kadro.adreso() as *mut u8, 0, KADRA_GRANDO as usize);
        }

        for segmento in segmentoj {
            let kopi_komenco = paĝo.max(segmento.virtuala);
            let kopi_fino = (paĝo + KADRA_GRANDO).min(segmento.virtuala + segmento.dosiera_longo);
            if kopi_komenco >= kopi_fino {
                continue;
            }
            let longo = (kopi_fino - kopi_komenco) as usize;
            let fonto = (segmento.dosiero + (kopi_komenco - segmento.virtuala)) as *const u8;
            let celo = (kadro.adreso() + (kopi_komenco - paĝo)) as *mut u8;
            // SAFETY: la fonto kuŝas en la identece mapita modulo (limigita per
            // `dosiera_longo`), la celo en la freŝe alokita kadro, kaj la
            // intervaloj estas disaj (`copy_nonoverlapping`).
            unsafe {
                ptr::copy_nonoverlapping(fonto, celo, longo);
            }
        }

        spaco.mapi_uzantan(
            virtuala_paĝo,
            FizikaAdreso::nova(kadro.adreso()),
            permesoj_el(skribebla, plenumebla),
            alokilo,
        )?;
        paĝo += KADRA_GRANDO;
    }
    Ok(())
}

fn permesoj_el(skribebla: bool, plenumebla: bool) -> PaĝaPermesoj {
    if skribebla {
        PaĝaPermesoj {
            skribebla: true,
            plenumebla,
            kaŝebla: true,
            uzanta: true,
        }
    } else if plenumebla {
        PaĝaPermesoj::UZANTA_KODO
    } else {
        PaĝaPermesoj::UZANTA_LEGA
    }
}

/// Ring 0-memtesto de la bito `U` kaj de `kontroli_uzantan`.
///
/// Oni mapas du provajn paĝojn — unu skribeblan kaj unu nurlegan — kaj
/// kontrolas pozitivajn kaj negativajn kazojn, inkluzive de la enirpunkta
/// paĝo de la bildo. Fine oni skribas markon tra la uzanta virtuala adreso;
/// ĉar la spaco estas la proceza, ne la aktiva kerna, la provo ŝaltas `CR3`
/// provizore kaj poste restarigas la antaŭan radikon.
fn testi_kontrolon(
    spaco: &mut impl AdresaSpaco,
    alokilo: &mut KadraAlokilo,
    bildo: &UzantaBildo,
) -> Result<(), PaĝaEraro> {
    spaco.mapigi_freŝan_regionon(
        VirtuaAdreso::nova(PROVA_BAZO),
        KADRA_GRANDO as usize,
        PaĝaPermesoj::UZANTA_DATUMOJ,
        alokilo,
    )?;
    spaco.mapigi_freŝan_regionon(
        VirtuaAdreso::nova(PROVA_BAZO + KADRA_GRANDO),
        KADRA_GRANDO as usize,
        PaĝaPermesoj::UZANTA_LEGA,
        alokilo,
    )?;

    let skribo = spaco.kontroli_uzantan(PROVA_BAZO, KADRA_GRANDO as usize, true);
    let lego = spaco.kontroli_uzantan(PROVA_BAZO, KADRA_GRANDO as usize, false);
    let nurlega_skribo =
        spaco.kontroli_uzantan(PROVA_BAZO + KADRA_GRANDO, KADRA_GRANDO as usize, true);
    let kerna = spaco.kontroli_uzantan(0x1000, KADRA_GRANDO as usize, false);
    let nemapita = spaco.kontroli_uzantan(PROVA_BAZO + 0x20_0000, KADRA_GRANDO as usize, false);
    let translima = spaco.kontroli_uzantan(PROVA_BAZO, 3 * KADRA_GRANDO as usize, false);
    let eksterfenestra = spaco.kontroli_uzantan(UZANTA_FINO, 1, false);
    let enirpunkta = spaco.kontroli_uzantan(bildo.enirpunkto, 1, false);

    let bone = skribo
        && lego
        && !nurlega_skribo
        && !kerna
        && !nemapita
        && !translima
        && !eksterfenestra
        && enirpunkta;
    crate::skribu!(
        "[uzanto] kontrolo ({}): skriba={} lega={} nurlega_skribo={} kerna={} nemapita={} translima={} eksterfenestra={} enirpunkta={}\n",
        if bone { "bone" } else { "MALBONE" },
        skribo,
        lego,
        nurlega_skribo,
        kerna,
        nemapita,
        translima,
        eksterfenestra,
        enirpunkta
    );

    let marko: u64 = 0x4D69_4B65_726E_6F21;
    // La bildo loĝas en la proceza spaco, ne en la aktiva kerna spaco. Ŝaltu
    // la procezan `CR3` provizore, skribu kaj legu tra la uzanta virtuala
    // adreso, poste restarigu la antaŭan radikon.
    let antaŭa = crate::arkitekturo::aktiva_adreso_spaco();
    let _gardo = InterrompaGardo::nova();
    spaco.ŝalti();
    // SAFETY: `PROVA_BAZO` estas paĝ-aliniiĝinta uzanta paĝo, kiun ni ĵus
    // mapis skribebla en `spaco`; dum ĉi tiu bloko `spaco` estas aktiva, do la
    // virtuala adreso tradukiĝas al la freŝa kadro.
    let legita = unsafe {
        ptr::write_volatile(PROVA_BAZO as *mut u64, marko);
        ptr::read_volatile(PROVA_BAZO as *const u64)
    };
    // SAFETY: `antaŭa` estis la radiko de la aktiva spaco antaŭ la provo; la
    // kerna radiko kaj ĝiaj mapoj restas vivaj, do la reveno estas sekura.
    unsafe { crate::arkitekturo::ŝalti_adresan_spacon(antaŭa) };
    crate::skribu!(
        "[uzanto] memoro: marko 0x{:x}, legita 0x{:x}\n",
        marko,
        legita
    );
    Ok(())
}

fn legi_u16(kapo: &[u8], de: usize) -> u16 {
    u16::from_le_bytes([kapo[de], kapo[de + 1]])
}

fn legi_u32(kapo: &[u8], de: usize) -> u32 {
    u32::from_le_bytes([kapo[de], kapo[de + 1], kapo[de + 2], kapo[de + 3]])
}

fn legi_u64(kapo: &[u8], de: usize) -> u64 {
    u64::from_le_bytes([
        kapo[de],
        kapo[de + 1],
        kapo[de + 2],
        kapo[de + 3],
        kapo[de + 4],
        kapo[de + 5],
        kapo[de + 6],
        kapo[de + 7],
    ])
}
