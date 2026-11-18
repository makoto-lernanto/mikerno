//! SNDS: Simpla NurLega Dosiersistemo.
//!
//! La bildo konsistas el superbloko (sektoro 0), inoda tabelo kaj datumaj
//! sektoroj. Ĉiu inodo priskribas dosieron aŭ dosierujon per tipo, grando,
//! unua sektoro kaj nombro da sektoroj; dosieruja ero havas 4-bajtan inodan
//! indekson kaj 60-bajtan nomon kun fina NUL. Ĉio estas little-endian.
//!
//! La dosiersistemo estas nurlega kaj muntas sin post la virtio-blk-pelilo.
//! Ĝi legas la bildon per `virtio_blk` kaj ne tenas kaŝmemoron de sektoroj.

use core::fmt;

use super::virtio_blk;
use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;

const SEKTORA_GRANDO: usize = 512;
const MAGIO: &[u8] = b"SNDS";
const VERSIO: u16 = 1;
const INODA_GRANDO: usize = 16;
const ERO_GRANDO: usize = 64;
const NOMA_MAKSIMO: usize = 59;
const DOSIERA_MAKSIMO: u32 = 64 * 1024;
const TIPO_DOSIERO: u8 = 1;
const TIPO_DOSIERUJO: u8 = 2;
const PROFUNDA_LIMO: u8 = 16;
const HAŜETA_BAZO: u64 = 0xcbf2_9ce4_8422_2325;
const HAŜETA_MULTIGILO: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy)]
pub enum Eraro {
    Konservado(virtio_blk::Eraro),
    NeKomencigita,
    NeValida(&'static str),
    NeTrovo,
    NeDosiero,
    NeDosierujo,
    BufroTroMalgranda,
    Memprovo(&'static str),
}

impl fmt::Display for Eraro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eraro::Konservado(eraro) => write!(f, "konservado: {eraro}"),
            Eraro::NeKomencigita => write!(f, "SNDS ne estas muntita"),
            Eraro::NeValida(kaŭzo) => write!(f, "nevalida bildo: {kaŭzo}"),
            Eraro::NeTrovo => write!(f, "la vojo ne troviĝas"),
            Eraro::NeDosiero => write!(f, "la vojo ne estas dosiero"),
            Eraro::NeDosierujo => write!(f, "la vojo ne estas dosierujo"),
            Eraro::BufroTroMalgranda => write!(f, "la bufro estas tro malgranda"),
            Eraro::Memprovo(kaŭzo) => write!(f, "memprovo: {kaŭzo}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Dosiero,
    Dosierujo,
}

#[derive(Clone, Copy)]
struct Inodo {
    tipo: u8,
    grando: u32,
    unua_sektoro: u32,
    sektoroj: u32,
}

impl Inodo {
    fn el_bajtoj(bajtoj: &[u8]) -> Result<Self, Eraro> {
        if bajtoj.len() < INODA_GRANDO {
            return Err(Eraro::NeValida("inodo estas tro mallonga"));
        }
        Ok(Self {
            tipo: bajtoj[0],
            grando: u32_el(bajtoj, 4),
            unua_sektoro: u32_el(bajtoj, 8),
            sektoroj: u32_el(bajtoj, 12),
        })
    }
}

#[derive(Clone, Copy)]
struct Superbloko {
    nombro_da_inodoj: u32,
    inoda_tabelo: u32,
    unua_datumo: u32,
    kapacito: u32,
    radika_inodo: u32,
}

impl Superbloko {
    fn el_bufro(bufro: &[u8; SEKTORA_GRANDO]) -> Result<Self, Eraro> {
        if !bufro.starts_with(MAGIO) {
            return Err(Eraro::NeValida("magio malkongruas"));
        }
        if u16_el(bufro, 4) != VERSIO {
            return Err(Eraro::NeValida("nesubtenata versio"));
        }
        if u16_el(bufro, 6) as usize != SEKTORA_GRANDO {
            return Err(Eraro::NeValida("la sektora grando ne estas 512"));
        }
        if u16_el(bufro, 8) as usize != INODA_GRANDO {
            return Err(Eraro::NeValida("la inoda grando ne estas 16"));
        }
        let superbloko = Self {
            nombro_da_inodoj: u32_el(bufro, 12),
            inoda_tabelo: u32_el(bufro, 16),
            unua_datumo: u32_el(bufro, 20),
            kapacito: u32_el(bufro, 24),
            radika_inodo: u32_el(bufro, 28),
        };
        if superbloko.nombro_da_inodoj == 0 {
            return Err(Eraro::NeValida("mankas inodoj"));
        }
        if superbloko.inoda_tabelo == 0
            || superbloko.unua_datumo <= superbloko.inoda_tabelo
            || superbloko.unua_datumo >= superbloko.kapacito
        {
            return Err(Eraro::NeValida("la sektoraj limoj malkongruas"));
        }
        let tabela_fino = superbloko.inoda_tabelo as u64 * SEKTORA_GRANDO as u64
            + superbloko.nombro_da_inodoj as u64 * INODA_GRANDO as u64;
        if tabela_fino > superbloko.unua_datumo as u64 * SEKTORA_GRANDO as u64 {
            return Err(Eraro::NeValida("la inoda tabelo transiras la datumojn"));
        }
        if superbloko.radika_inodo >= superbloko.nombro_da_inodoj {
            return Err(Eraro::NeValida("la radika inodo ne ekzistas"));
        }
        Ok(superbloko)
    }
}

fn u16_el(bajtoj: &[u8], ofseto: usize) -> u16 {
    u16::from_le_bytes([bajtoj[ofseto], bajtoj[ofseto + 1]])
}

fn u32_el(bajtoj: &[u8], ofseto: usize) -> u32 {
    u32::from_le_bytes([
        bajtoj[ofseto],
        bajtoj[ofseto + 1],
        bajtoj[ofseto + 2],
        bajtoj[ofseto + 3],
    ])
}

fn haketi(haketo: &mut u64, bajtoj: &[u8]) {
    for &bajto in bajtoj {
        *haketo ^= bajto as u64;
        *haketo = haketo.wrapping_mul(HAŜETA_MULTIGILO);
    }
}

fn ero_nomo(ero: &[u8; ERO_GRANDO]) -> Result<(&str, u32), Eraro> {
    let indekso = u32_el(ero, 0);
    let nomaj = &ero[4..];
    let fino = nomaj
        .iter()
        .position(|&bajto| bajto == 0)
        .ok_or(Eraro::NeValida("la nomo ne havas finan NUL"))?;
    if fino == 0 || fino > NOMA_MAKSIMO {
        return Err(Eraro::NeValida("la nomo estas malplena aŭ tro longa"));
    }
    let nomo = core::str::from_utf8(&nomaj[..fino])
        .map_err(|_| Eraro::NeValida("la nomo ne estas UTF-8"))?;
    Ok((nomo, indekso))
}

pub struct Dosiero {
    inodo: Inodo,
}

impl Dosiero {
    pub fn grando(&self) -> u32 {
        self.inodo.grando
    }
}

struct SimplaDosiersistemo {
    superbloko: Superbloko,
    dosieroj: u32,
    dosierujoj: u32,
    haketo: u64,
}

impl SimplaDosiersistemo {
    fn kontroli_inodon(&self, inodo: &Inodo) -> Result<(), Eraro> {
        if inodo.tipo != TIPO_DOSIERO && inodo.tipo != TIPO_DOSIERUJO {
            return Err(Eraro::NeValida("nekonata inoda tipo"));
        }
        if inodo.grando > DOSIERA_MAKSIMO {
            return Err(Eraro::NeValida("la dosiero transiras 64 KiB"));
        }
        if inodo.sektoroj != inodo.grando.div_ceil(SEKTORA_GRANDO as u32) {
            return Err(Eraro::NeValida(
                "la sektora nombro malkongruas kun la grando",
            ));
        }
        if inodo.unua_sektoro < self.superbloko.unua_datumo {
            return Err(Eraro::NeValida("la inodo montras antaŭ la datumojn"));
        }
        if inodo.unua_sektoro as u64 + inodo.sektoroj as u64 > self.superbloko.kapacito as u64 {
            return Err(Eraro::NeValida("la inodo transiras la kapaciton"));
        }
        if inodo.tipo == TIPO_DOSIERUJO && !inodo.grando.is_multiple_of(ERO_GRANDO as u32) {
            return Err(Eraro::NeValida(
                "la dosieruja grando ne estas multoblo de 64",
            ));
        }
        if inodo.tipo == TIPO_DOSIERUJO
            && inodo.grando / ERO_GRANDO as u32 > self.superbloko.nombro_da_inodoj
        {
            return Err(Eraro::NeValida("la dosierujo havas pli da eroj ol inodoj"));
        }
        Ok(())
    }

    fn legi_inodon(&self, indekso: u32, bufro: &mut [u8; SEKTORA_GRANDO]) -> Result<Inodo, Eraro> {
        if indekso >= self.superbloko.nombro_da_inodoj {
            return Err(Eraro::NeValida("inoda indekso ekster la tabelo"));
        }
        let ofseto = self.superbloko.inoda_tabelo as u64 * SEKTORA_GRANDO as u64
            + indekso as u64 * INODA_GRANDO as u64;
        let sektoro = ofseto / SEKTORA_GRANDO as u64;
        let eno = (ofseto % SEKTORA_GRANDO as u64) as usize;
        virtio_blk::legi_blokon(sektoro, bufro).map_err(Eraro::Konservado)?;
        let inodo = Inodo::el_bajtoj(&bufro[eno..eno + INODA_GRANDO])?;
        self.kontroli_inodon(&inodo)?;
        Ok(inodo)
    }

    fn legi_eron(
        &self,
        inodo: &Inodo,
        indekso: u32,
        bufro: &mut [u8; SEKTORA_GRANDO],
    ) -> Result<[u8; ERO_GRANDO], Eraro> {
        let ofseto = indekso as u64 * ERO_GRANDO as u64;
        let sektoro = inodo.unua_sektoro as u64 + ofseto / SEKTORA_GRANDO as u64;
        let eno = (ofseto % SEKTORA_GRANDO as u64) as usize;
        virtio_blk::legi_blokon(sektoro, bufro).map_err(Eraro::Konservado)?;
        let mut ero = [0u8; ERO_GRANDO];
        ero.copy_from_slice(&bufro[eno..eno + ERO_GRANDO]);
        Ok(ero)
    }

    fn serĉi(
        &self,
        dosierujo: &Inodo,
        nomo: &str,
        bufro: &mut [u8; SEKTORA_GRANDO],
    ) -> Result<Inodo, Eraro> {
        let nombro = dosierujo.grando / ERO_GRANDO as u32;
        for indekso in 0..nombro {
            let ero = self.legi_eron(dosierujo, indekso, bufro)?;
            let (era_nomo, era_inodo) = ero_nomo(&ero)?;
            if era_nomo == nomo {
                return self.legi_inodon(era_inodo, bufro);
            }
        }
        Err(Eraro::NeTrovo)
    }

    fn solvi(&self, vojo: &str) -> Result<Inodo, Eraro> {
        let mut bufro = [0u8; SEKTORA_GRANDO];
        let mut nuna = self.legi_inodon(self.superbloko.radika_inodo, &mut bufro)?;
        for peco in vojo.split('/') {
            if peco.is_empty() {
                continue;
            }
            if peco == "." || peco == ".." {
                return Err(Eraro::NeValida("la vojo enhavas . aŭ .."));
            }
            if nuna.tipo != TIPO_DOSIERUJO {
                return Err(Eraro::NeDosierujo);
            }
            nuna = self.serĉi(&nuna, peco, &mut bufro)?;
        }
        Ok(nuna)
    }

    fn legi_datumojn(&self, inodo: &Inodo, celo: &mut [u8]) -> Result<(), Eraro> {
        let mut bloko = [0u8; SEKTORA_GRANDO];
        let mut farita = 0;
        while farita < celo.len() {
            let sektoro = inodo.unua_sektoro as u64 + (farita / SEKTORA_GRANDO) as u64;
            virtio_blk::legi_blokon(sektoro, &mut bloko).map_err(Eraro::Konservado)?;
            let eno = farita % SEKTORA_GRANDO;
            let nombro = core::cmp::min(SEKTORA_GRANDO - eno, celo.len() - farita);
            celo[farita..farita + nombro].copy_from_slice(&bloko[eno..eno + nombro]);
            farita += nombro;
        }
        Ok(())
    }

    fn haketi_dosieron(
        &mut self,
        inodo: &Inodo,
        bufro: &mut [u8; SEKTORA_GRANDO],
    ) -> Result<(), Eraro> {
        let mut restantaj = inodo.grando;
        let mut sektoro = inodo.unua_sektoro as u64;
        while restantaj > 0 {
            virtio_blk::legi_blokon(sektoro, bufro).map_err(Eraro::Konservado)?;
            let nombro = core::cmp::min(restantaj as usize, SEKTORA_GRANDO);
            haketi(&mut self.haketo, &bufro[..nombro]);
            restantaj -= nombro as u32;
            sektoro += 1;
        }
        Ok(())
    }

    fn esplori(
        &mut self,
        indekso: u32,
        profundo: u8,
        bufro: &mut [u8; SEKTORA_GRANDO],
    ) -> Result<(), Eraro> {
        let inodo = self.legi_inodon(indekso, bufro)?;
        if inodo.tipo == TIPO_DOSIERO {
            self.dosieroj = self.dosieroj.saturating_add(1);
            haketi(&mut self.haketo, &inodo.grando.to_le_bytes());
            self.haketi_dosieron(&inodo, bufro)?;
            return Ok(());
        }

        let nombro = inodo.grando / ERO_GRANDO as u32;
        for indekso in 0..nombro {
            let ero = self.legi_eron(&inodo, indekso, bufro)?;
            let (nomo, infa) = ero_nomo(&ero)?;
            haketi(&mut self.haketo, nomo.as_bytes());
            haketi(&mut self.haketo, &[0]);
            let infa_inodo = self.legi_inodon(infa, bufro)?;
            if infa_inodo.tipo == TIPO_DOSIERUJO {
                if profundo + 1 > PROFUNDA_LIMO {
                    return Err(Eraro::NeValida("la dosierujo estas tro profunda"));
                }
                self.dosierujoj = self.dosierujoj.saturating_add(1);
                self.esplori(infa, profundo + 1, bufro)?;
                haketi(&mut self.haketo, &[0xff]);
            } else {
                self.esplori(infa, profundo + 1, bufro)?;
            }
        }
        Ok(())
    }
}

static DOSIERSISTEMO: Kunhavigita<Option<SimplaDosiersistemo>> = Kunhavigita::nova(None);

/// Malfermu dosieron per vojo relative al la radiko.
pub fn malfermi(vojo: &str) -> Result<Dosiero, Eraro> {
    let _gardo = InterrompaGardo::nova();
    let Some(sistemo) = DOSIERSISTEMO.akiri().as_ref() else {
        return Err(Eraro::NeKomencigita);
    };
    let inodo = sistemo.solvi(vojo)?;
    if inodo.tipo != TIPO_DOSIERO {
        return Err(Eraro::NeDosiero);
    }
    Ok(Dosiero { inodo })
}

/// Legu la tutan dosieron en `bufro`; la bufro devas esti sufiĉe granda.
pub fn legi(dosiero: &Dosiero, bufro: &mut [u8]) -> Result<usize, Eraro> {
    let grando = dosiero.inodo.grando as usize;
    if bufro.len() < grando {
        return Err(Eraro::BufroTroMalgranda);
    }
    let _gardo = InterrompaGardo::nova();
    let Some(sistemo) = DOSIERSISTEMO.akiri().as_ref() else {
        return Err(Eraro::NeKomencigita);
    };
    sistemo.legi_datumojn(&dosiero.inodo, &mut bufro[..grando])?;
    Ok(grando)
}

/// Listigu la erojn de dosierujo kaj voku `vizito` por ĉiu el ili.
///
/// La vokanto ne rajtas voki SNDS-funkciojn el la `vizito`-kromprogramo.
pub fn listi_dosierujon(vojo: &str, mut vizito: impl FnMut(&str, Tipo)) -> Result<(), Eraro> {
    let _gardo = InterrompaGardo::nova();
    let Some(sistemo) = DOSIERSISTEMO.akiri().as_ref() else {
        return Err(Eraro::NeKomencigita);
    };
    let inodo = sistemo.solvi(vojo)?;
    if inodo.tipo != TIPO_DOSIERUJO {
        return Err(Eraro::NeDosierujo);
    }
    let mut bufro = [0u8; SEKTORA_GRANDO];
    let nombro = inodo.grando / ERO_GRANDO as u32;
    for indekso in 0..nombro {
        let ero = sistemo.legi_eron(&inodo, indekso, &mut bufro)?;
        let (nomo, infa) = ero_nomo(&ero)?;
        let infa_inodo = sistemo.legi_inodon(infa, &mut bufro)?;
        let tipo = if infa_inodo.tipo == TIPO_DOSIERO {
            Tipo::Dosiero
        } else {
            Tipo::Dosierujo
        };
        vizito(nomo, tipo);
    }
    Ok(())
}

fn memprovo() -> Result<(), Eraro> {
    let dosiero = malfermi("bonveno.txt")?;
    let mut bufro = [0u8; 64];
    let legita = legi(&dosiero, &mut bufro)?;
    if &bufro[..legita] != b"Saluton, MiKerno!\n" {
        return Err(Eraro::Memprovo(
            "bonveno.txt ne enhavas la atendatan tekston",
        ));
    }
    crate::skribu!(
        "[dosiersistemo] legado (bone): bonveno.txt, {} bajtoj\n",
        dosiero.grando()
    );

    let mut unua = true;
    crate::skribu!("[dosiersistemo] listado de notoj:");
    listi_dosierujon("notoj", |nomo, tipo| {
        if !unua {
            crate::skribu!(",");
        }
        crate::skribu!(" {}", nomo);
        if tipo == Tipo::Dosierujo {
            crate::skribu!("/");
        }
        unua = false;
    })?;
    crate::skribu!("\n");
    Ok(())
}

/// Trovu, validigu kaj memprovi la SNDS-bildon.
pub fn pravalorizi() -> Result<(), Eraro> {
    let mut bufro = [0u8; SEKTORA_GRANDO];
    virtio_blk::legi_blokon(0, &mut bufro).map_err(Eraro::Konservado)?;
    let superbloko = Superbloko::el_bufro(&bufro)?;
    if superbloko.kapacito as u64 > virtio_blk::kapacito() {
        return Err(Eraro::NeValida("la bildo estas pli granda ol la aparato"));
    }

    let mut sistemo = SimplaDosiersistemo {
        superbloko,
        dosieroj: 0,
        dosierujoj: 0,
        haketo: HAŜETA_BAZO,
    };
    sistemo.esplori(superbloko.radika_inodo, 0, &mut bufro)?;
    let dosieroj = sistemo.dosieroj;
    let dosierujoj = sistemo.dosierujoj;
    let haketo = sistemo.haketo;
    {
        let _gardo = InterrompaGardo::nova();
        *DOSIERSISTEMO.akiri() = Some(sistemo);
    }
    crate::skribu!(
        "[dosiersistemo] SNDS: {} dosieroj, {} dosierujo{}. Haketo: 0x{:016x}\n",
        dosieroj,
        dosierujoj,
        if dosierujoj == 1 { "" } else { "j" },
        haketo
    );
    memprovo()
}
