//! Pura analizo kaj validigo de SNDS (Simpla NurLega Dosiersistemo).
//!
//! Ĉi tiu modulo tenas la bildan aranĝon kaj ĉiujn kontrolojn; la legado de
//! la sektoroj mem restas en la kerna pelilo (`virtio_blk`), kiu uzas ĉi tiujn
//! tipojn. La bildo konsistas el superbloko (sektoro 0), inoda tabelo kaj
//! datumaj sektoroj. Ĉiu inodo priskribas dosieron aŭ dosierujon per tipo,
//! grando, unua sektoro kaj nombro da sektoroj; dosieruja ero havas 4-bajtan
//! inodan indekson kaj 60-bajtan nomon kun fina NUL. Ĉio estas little-endian.

use core::fmt;

pub const SEKTORA_GRANDO: usize = 512;
const MAGIO: &[u8] = b"SNDS";
const VERSIO: u16 = 1;
pub const INODA_GRANDO: usize = 16;
pub const ERO_GRANDO: usize = 64;
pub const NOMA_MAKSIMO: usize = 59;
pub const DOSIERA_MAKSIMO: u32 = 128 * 1024;
pub const TIPO_DOSIERO: u8 = 1;
pub const TIPO_DOSIERUJO: u8 = 2;
pub const PROFUNDA_LIMO: u8 = 16;
pub const HAŜETA_BAZO: u64 = 0xcbf2_9ce4_8422_2325;
pub const HAŜETA_MULTIGILO: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eraro {
    NeValida(&'static str),
}

impl fmt::Display for Eraro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Eraro::NeValida(kaŭzo) => write!(f, "nevalida bildo: {kaŭzo}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inodo {
    pub tipo: u8,
    pub grando: u32,
    pub unua_sektoro: u32,
    pub sektoroj: u32,
}

impl Inodo {
    pub fn el_bajtoj(bajtoj: &[u8]) -> Result<Self, Eraro> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Superbloko {
    pub nombro_da_inodoj: u32,
    pub inoda_tabelo: u32,
    pub unua_datumo: u32,
    pub kapacito: u32,
    pub radika_inodo: u32,
}

impl Superbloko {
    pub fn el_bufro(bufro: &[u8; SEKTORA_GRANDO]) -> Result<Self, Eraro> {
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

/// Aldonu bajtojn al la FNV-1a-haketo de la bildo.
pub fn haketi(haketo: &mut u64, bajtoj: &[u8]) {
    for &bajto in bajtoj {
        *haketo ^= bajto as u64;
        *haketo = haketo.wrapping_mul(HAŜETA_MULTIGILO);
    }
}

pub fn ero_nomo(ero: &[u8; ERO_GRANDO]) -> Result<(&str, u32), Eraro> {
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

/// Kontrolu inodon kontraŭ la superbloko: la tipo, la grando, la sektora
/// aritmetiko kaj la limoj de la datuma regiono.
pub fn kontroli_inodon(superbloko: &Superbloko, inodo: &Inodo) -> Result<(), Eraro> {
    if inodo.tipo != TIPO_DOSIERO && inodo.tipo != TIPO_DOSIERUJO {
        return Err(Eraro::NeValida("nekonata inoda tipo"));
    }
    if inodo.grando > DOSIERA_MAKSIMO {
        return Err(Eraro::NeValida("la dosiero transiras 128 KiB"));
    }
    if inodo.sektoroj != inodo.grando.div_ceil(SEKTORA_GRANDO as u32) {
        return Err(Eraro::NeValida(
            "la sektora nombro malkongruas kun la grando",
        ));
    }
    if inodo.unua_sektoro < superbloko.unua_datumo {
        return Err(Eraro::NeValida("la inodo montras antaŭ la datumojn"));
    }
    if inodo.unua_sektoro as u64 + inodo.sektoroj as u64 > superbloko.kapacito as u64 {
        return Err(Eraro::NeValida("la inodo transiras la kapaciton"));
    }
    if inodo.tipo == TIPO_DOSIERUJO && !inodo.grando.is_multiple_of(ERO_GRANDO as u32) {
        return Err(Eraro::NeValida(
            "la dosieruja grando ne estas multoblo de 64",
        ));
    }
    if inodo.tipo == TIPO_DOSIERUJO
        && inodo.grando / ERO_GRANDO as u32 > superbloko.nombro_da_inodoj
    {
        return Err(Eraro::NeValida("la dosierujo havas pli da eroj ol inodoj"));
    }
    Ok(())
}

#[cfg(test)]
mod testoj {
    use super::*;

    fn superbloko_bajtoj() -> [u8; SEKTORA_GRANDO] {
        let mut bufro = [0u8; SEKTORA_GRANDO];
        bufro[0..4].copy_from_slice(MAGIO);
        bufro[4..6].copy_from_slice(&VERSIO.to_le_bytes());
        bufro[6..8].copy_from_slice(&(SEKTORA_GRANDO as u16).to_le_bytes());
        bufro[8..10].copy_from_slice(&(INODA_GRANDO as u16).to_le_bytes());
        bufro[12..16].copy_from_slice(&8u32.to_le_bytes());
        bufro[16..20].copy_from_slice(&1u32.to_le_bytes());
        bufro[20..24].copy_from_slice(&2u32.to_le_bytes());
        bufro[24..28].copy_from_slice(&64u32.to_le_bytes());
        bufro[28..32].copy_from_slice(&0u32.to_le_bytes());
        bufro
    }

    fn valida_superbloko() -> Superbloko {
        Superbloko::el_bufro(&superbloko_bajtoj()).unwrap()
    }

    fn inodo(tipo: u8, grando: u32, unua_sektoro: u32, sektoroj: u32) -> Inodo {
        Inodo {
            tipo,
            grando,
            unua_sektoro,
            sektoroj,
        }
    }

    fn ero(indekso: u32, nomo: &[u8]) -> [u8; ERO_GRANDO] {
        let mut ero = [0u8; ERO_GRANDO];
        ero[0..4].copy_from_slice(&indekso.to_le_bytes());
        ero[4..4 + nomo.len()].copy_from_slice(nomo);
        ero
    }

    #[test]
    fn valida_superbloko_analiziĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(superbloko.nombro_da_inodoj, 8);
        assert_eq!(superbloko.unua_datumo, 2);
        assert_eq!(superbloko.kapacito, 64);
    }

    #[test]
    fn magio_malkongrua_rifuziĝas() {
        let mut bufro = superbloko_bajtoj();
        bufro[0] = b'X';
        assert_eq!(
            Superbloko::el_bufro(&bufro),
            Err(Eraro::NeValida("magio malkongruas"))
        );
    }

    #[test]
    fn nesubtenata_versio_rifuziĝas() {
        let mut bufro = superbloko_bajtoj();
        bufro[4..6].copy_from_slice(&2u16.to_le_bytes());
        assert_eq!(
            Superbloko::el_bufro(&bufro),
            Err(Eraro::NeValida("nesubtenata versio"))
        );
    }

    #[test]
    fn malĝusta_sektora_grando_rifuziĝas() {
        let mut bufro = superbloko_bajtoj();
        bufro[6..8].copy_from_slice(&1024u16.to_le_bytes());
        assert_eq!(
            Superbloko::el_bufro(&bufro),
            Err(Eraro::NeValida("la sektora grando ne estas 512"))
        );
    }

    #[test]
    fn malĝusta_inoda_grando_rifuziĝas() {
        let mut bufro = superbloko_bajtoj();
        bufro[8..10].copy_from_slice(&32u16.to_le_bytes());
        assert_eq!(
            Superbloko::el_bufro(&bufro),
            Err(Eraro::NeValida("la inoda grando ne estas 16"))
        );
    }

    #[test]
    fn bildo_sen_inodoj_rifuziĝas() {
        let mut bufro = superbloko_bajtoj();
        bufro[12..16].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(
            Superbloko::el_bufro(&bufro),
            Err(Eraro::NeValida("mankas inodoj"))
        );
    }

    #[test]
    fn sektoraj_limoj_malkongruaj_rifuziĝas() {
        let mut bufro = superbloko_bajtoj();
        bufro[20..24].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(
            Superbloko::el_bufro(&bufro),
            Err(Eraro::NeValida("la sektoraj limoj malkongruas"))
        );
    }

    #[test]
    fn tro_granda_inoda_tabelo_rifuziĝas() {
        let mut bufro = superbloko_bajtoj();
        bufro[12..16].copy_from_slice(&100u32.to_le_bytes());
        assert_eq!(
            Superbloko::el_bufro(&bufro),
            Err(Eraro::NeValida("la inoda tabelo transiras la datumojn"))
        );
    }

    #[test]
    fn radika_inodo_ekster_la_tabelo_rifuziĝas() {
        let mut bufro = superbloko_bajtoj();
        bufro[28..32].copy_from_slice(&8u32.to_le_bytes());
        assert_eq!(
            Superbloko::el_bufro(&bufro),
            Err(Eraro::NeValida("la radika inodo ne ekzistas"))
        );
    }

    #[test]
    fn valida_dosiera_inodo_akceptiĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(
            kontroli_inodon(&superbloko, &inodo(TIPO_DOSIERO, 600, 2, 2)),
            Ok(())
        );
    }

    #[test]
    fn tro_mallonga_inodo_rifuziĝas() {
        assert_eq!(
            Inodo::el_bajtoj(&[0u8; INODA_GRANDO - 1]),
            Err(Eraro::NeValida("inodo estas tro mallonga"))
        );
    }

    #[test]
    fn nekonata_inoda_tipo_rifuziĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(
            kontroli_inodon(&superbloko, &inodo(3, 0, 2, 0)),
            Err(Eraro::NeValida("nekonata inoda tipo"))
        );
    }

    #[test]
    fn tro_granda_dosiero_rifuziĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(
            kontroli_inodon(
                &superbloko,
                &inodo(TIPO_DOSIERO, DOSIERA_MAKSIMO + 1, 2, 257)
            ),
            Err(Eraro::NeValida("la dosiero transiras 128 KiB"))
        );
    }

    #[test]
    fn sektora_nombro_malkongrua_rifuziĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(
            kontroli_inodon(&superbloko, &inodo(TIPO_DOSIERO, 600, 2, 1)),
            Err(Eraro::NeValida(
                "la sektora nombro malkongruas kun la grando"
            ))
        );
    }

    #[test]
    fn inodo_antau_la_datumoj_rifuziĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(
            kontroli_inodon(&superbloko, &inodo(TIPO_DOSIERO, 512, 1, 1)),
            Err(Eraro::NeValida("la inodo montras antaŭ la datumojn"))
        );
    }

    #[test]
    fn inodo_transiras_la_kapaciton_rifuziĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(
            kontroli_inodon(&superbloko, &inodo(TIPO_DOSIERO, 1024, 63, 2)),
            Err(Eraro::NeValida("la inodo transiras la kapaciton"))
        );
    }

    #[test]
    fn dosieruja_grando_ne_multoblo_rifuziĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(
            kontroli_inodon(&superbloko, &inodo(TIPO_DOSIERUJO, 100, 2, 1)),
            Err(Eraro::NeValida(
                "la dosieruja grando ne estas multoblo de 64"
            ))
        );
    }

    #[test]
    fn dosierujo_kun_tro_da_eroj_rifuziĝas() {
        let superbloko = valida_superbloko();
        assert_eq!(
            kontroli_inodon(
                &superbloko,
                &inodo(TIPO_DOSIERUJO, (ERO_GRANDO as u32) * 9, 2, 2)
            ),
            Err(Eraro::NeValida("la dosierujo havas pli da eroj ol inodoj"))
        );
    }

    #[test]
    fn ero_nomo_estas_legata() {
        let ero = ero(5, b"notoj");
        let (nomo, indekso) = ero_nomo(&ero).unwrap();
        assert_eq!(nomo, "notoj");
        assert_eq!(indekso, 5);
    }

    #[test]
    fn ero_sen_fina_nul_rifuziĝas() {
        let mut ero = [b'A'; ERO_GRANDO];
        ero[0..4].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(
            ero_nomo(&ero),
            Err(Eraro::NeValida("la nomo ne havas finan NUL"))
        );
    }

    #[test]
    fn ero_kun_malplena_nomo_rifuziĝas() {
        assert_eq!(
            ero_nomo(&ero(1, b"")),
            Err(Eraro::NeValida("la nomo estas malplena aŭ tro longa"))
        );
    }

    #[test]
    fn ero_kun_ne_utf8_nomo_rifuziĝas() {
        assert_eq!(
            ero_nomo(&ero(1, &[0xff, 0xfe])),
            Err(Eraro::NeValida("la nomo ne estas UTF-8"))
        );
    }

    #[test]
    fn ero_kun_maksimuma_nomo_akceptiĝas() {
        let nomo = [b'n'; NOMA_MAKSIMO];
        let ero = ero(2, &nomo);
        let (legita, _) = ero_nomo(&ero).unwrap();
        assert_eq!(legita.len(), NOMA_MAKSIMO);
    }

    #[test]
    fn haketo_matchas_la_konatan_fnv1a_valoron() {
        let mut haketo = HAŜETA_BAZO;
        haketi(&mut haketo, b"");
        assert_eq!(haketo, 0xcbf2_9ce4_8422_2325);

        let mut haketo = HAŜETA_BAZO;
        haketi(&mut haketo, b"a");
        assert_eq!(haketo, 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn haketo_de_sinsekvaj_pecoj_egalas_la_tutan() {
        let mut disigita = HAŜETA_BAZO;
        haketi(&mut disigita, b"ab");
        haketi(&mut disigita, b"c");

        let mut tuta = HAŜETA_BAZO;
        haketi(&mut tuta, b"abc");

        assert_eq!(disigita, tuta);
    }
}
