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

use kernologiko::snds::{
    ERO_GRANDO, HAŜETA_BAZO, INODA_GRANDO, Inodo, PROFUNDA_LIMO, SEKTORA_GRANDO, Superbloko,
    TIPO_DOSIERO, TIPO_DOSIERUJO, ero_nomo, haketi,
};

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

impl From<kernologiko::snds::Eraro> for Eraro {
    fn from(eraro: kernologiko::snds::Eraro) -> Self {
        match eraro {
            kernologiko::snds::Eraro::NeValida(kaŭzo) => Eraro::NeValida(kaŭzo),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Dosiero,
    Dosierujo,
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
    fn legi_inodon(&self, indekso: u32, bufro: &mut [u8; SEKTORA_GRANDO]) -> Result<Inodo, Eraro> {
        if indekso >= self.superbloko.nombro_da_inodoj {
            return Err(Eraro::NeValida("inoda indekso ekster la tabelo"));
        }
        let ofseto = self.superbloko.inoda_tabelo as u64 * SEKTORA_GRANDO as u64
            + indekso as u64 * INODA_GRANDO as u64;
        let sektoro = ofseto / SEKTORA_GRANDO as u64;
        let eno = (ofseto % SEKTORA_GRANDO as u64) as usize;
        virtio_blk::legi_blokon(sektoro, bufro).map_err(Eraro::Konservado)?;
        let inodo = Inodo::el_bajtoj(&bufro[eno..eno + INODA_GRANDO]).map_err(Eraro::from)?;
        kernologiko::snds::kontroli_inodon(&self.superbloko, &inodo).map_err(Eraro::from)?;
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
            let (era_nomo, era_inodo) = ero_nomo(&ero).map_err(Eraro::from)?;
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
            let (nomo, infa) = ero_nomo(&ero).map_err(Eraro::from)?;
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
    let Some(sistemo) = (unsafe { (*DOSIERSISTEMO.akiri()).as_ref() }) else {
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
    let Some(sistemo) = (unsafe { (*DOSIERSISTEMO.akiri()).as_ref() }) else {
        return Err(Eraro::NeKomencigita);
    };
    sistemo.legi_datumojn(&dosiero.inodo, &mut bufro[..grando])?;
    Ok(grando)
}

/// Listigu la erojn de dosierujo kaj plenigu `celo` per `Dosierero`-j.
///
/// Redonas la nombron da plenigitaj registroj (maksimume `celo.len()`).
/// `inodo` estas la SNDS-inoda indekso; la ABI uzas ĝin kiel tenilo-valoro
/// ĝis la dosiersistemo ekhavos verajn kapabilajn dosier-tenilojn.
pub fn listi_registrojn(
    vojo: &str,
    celo: &mut [libmikerno::registro::Dosierero],
) -> Result<usize, Eraro> {
    let _gardo = InterrompaGardo::nova();
    let Some(sistemo) = (unsafe { (*DOSIERSISTEMO.akiri()).as_ref() }) else {
        return Err(Eraro::NeKomencigita);
    };
    let inodo = sistemo.solvi(vojo)?;
    if inodo.tipo != TIPO_DOSIERUJO {
        return Err(Eraro::NeDosierujo);
    }
    let mut bufro = [0u8; SEKTORA_GRANDO];
    let nombro = inodo.grando / ERO_GRANDO as u32;
    let mut plenigitaj = 0;
    for indekso in 0..nombro {
        if plenigitaj == celo.len() {
            break;
        }
        let ero = sistemo.legi_eron(&inodo, indekso, &mut bufro)?;
        let (nomo, infa) = ero_nomo(&ero).map_err(Eraro::from)?;
        let infa_inodo = sistemo.legi_inodon(infa, &mut bufro)?;
        let registro = &mut celo[plenigitaj];
        *registro = libmikerno::registro::Dosierero::nova();
        let nomaj = nomo.as_bytes();
        registro.nomo[..nomaj.len()].copy_from_slice(nomaj);
        registro.tipo = if infa_inodo.tipo == TIPO_DOSIERO {
            libmikerno::registro::TIPO_DOSIERO
        } else {
            libmikerno::registro::TIPO_DOSIERUJO
        };
        registro.grandeco = infa_inodo.grando;
        registro.inodo = infa;
        plenigitaj += 1;
    }
    Ok(plenigitaj)
}

/// Listigu la erojn de dosierujo kaj voku `vizito` por ĉiu el ili.
///
/// La vokanto ne rajtas voki SNDS-funkciojn el la `vizito`-kromprogramo.
pub fn listi_dosierujon(vojo: &str, mut vizito: impl FnMut(&str, Tipo)) -> Result<(), Eraro> {
    let mut registroj = [libmikerno::registro::Dosierero::nova(); 32];
    let nombro = listi_registrojn(vojo, &mut registroj)?;
    for registro in &registroj[..nombro] {
        let fino = registro
            .nomo
            .iter()
            .position(|&bajto| bajto == 0)
            .unwrap_or(0);
        let nomo = core::str::from_utf8(&registro.nomo[..fino])
            .map_err(|_| Eraro::NeValida("la nomo ne estas UTF-8"))?;
        let tipo = if registro.tipo == libmikerno::registro::TIPO_DOSIERUJO {
            Tipo::Dosierujo
        } else {
            Tipo::Dosiero
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
    let superbloko = Superbloko::el_bufro(&bufro).map_err(Eraro::from)?;
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
        unsafe { *DOSIERSISTEMO.akiri() = Some(sistemo) };
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
