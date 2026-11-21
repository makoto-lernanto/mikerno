//! Solvado de relativaj kaj `~`-aj vojoj por la ŝelo Ligo.
//!
//! La solvilo estas pure teksta: ĝi normaligas la vojon antaŭ ol la kerno
//! vidas ĝin, do SNDS neniam ricevas `.` aŭ `..`. La signo `~` ĉiam signifas
//! `HEJMO`, kaj `..` ĉe la radiko restas `/`.

/// La hejma dosierujo de Ligo; `~` kaj la starta dosierujo.
pub const HEJMO: &str = "/hejmo";

/// La maksimuma rezulta vojo, inkluzive la finan nulan spacon ne bezonatan.
pub const VOJA_MAKSIMO: usize = 160;

/// Kial la vojo ne solveblas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VojaEraro {
    /// La rezulta vojo ne enas en la celan bufron.
    TroLonga,
    /// La nuna dosierujo mem ne estas absoluta.
    Neabsoluta,
    /// `~` aperas kun uzantnomo, kiun Ligo ne subtenas.
    NevalidaTilde,
}

/// Solvu `vojo` kontraŭ la nuna dosierujo `nuna` kaj skribu la normaligitan
/// absoluton en `eligo`; redonas ĝian longon.
///
/// Absoluta `vojo` ignoras `nuna`. `~` kaj `~/…` komenciĝas ĉe [`HEJMO`];
/// `..` forprenas la lastan komponanton, sed neniam eliras la radikon.
pub fn kunigi(nuna: &str, vojo: &str, eligo: &mut [u8]) -> Result<usize, VojaEraro> {
    if !nuna.starts_with('/') {
        return Err(VojaEraro::Neabsoluta);
    }
    let (bazo, resto) = if let Some(resto) = vojo.strip_prefix('/') {
        ("", resto)
    } else if vojo == "~" {
        (HEJMO, "")
    } else if let Some(resto) = vojo.strip_prefix("~/") {
        (HEJMO, resto)
    } else if vojo.starts_with('~') {
        return Err(VojaEraro::NevalidaTilde);
    } else {
        (nuna, vojo)
    };
    let mut longo = 0;
    apliku(bazo, eligo, &mut longo)?;
    apliku(resto, eligo, &mut longo)?;
    if longo == 0 {
        if eligo.is_empty() {
            return Err(VojaEraro::TroLonga);
        }
        eligo[0] = b'/';
        longo = 1;
    }
    Ok(longo)
}

/// La montro de `nuna` en la prompto: `~` en la hejmo, `~/…` sub ĝi, alie la
/// plena vojo. La rezulto estas skribita en `eligo`.
pub fn por_invito(nuna: &str, eligo: &mut [u8]) -> Result<usize, VojaEraro> {
    if nuna == HEJMO {
        return kopii("~", eligo);
    }
    if let Some(resto) = nuna
        .strip_prefix(HEJMO)
        .and_then(|resto| resto.strip_prefix('/'))
    {
        let longo = 2 + resto.len();
        if longo > eligo.len() {
            return Err(VojaEraro::TroLonga);
        }
        eligo[0] = b'~';
        eligo[1] = b'/';
        eligo[2..longo].copy_from_slice(resto.as_bytes());
        return Ok(longo);
    }
    kopii(nuna, eligo)
}

fn kopii(teksto: &str, eligo: &mut [u8]) -> Result<usize, VojaEraro> {
    if teksto.len() > eligo.len() {
        return Err(VojaEraro::TroLonga);
    }
    eligo[..teksto.len()].copy_from_slice(teksto.as_bytes());
    Ok(teksto.len())
}

fn apliku(parto: &str, eligo: &mut [u8], longo: &mut usize) -> Result<(), VojaEraro> {
    for peco in parto.split('/') {
        match peco {
            "" | "." => {}
            ".." => repreni(eligo, longo),
            _ => aldoni(peco, eligo, longo)?,
        }
    }
    Ok(())
}

fn aldoni(peco: &str, eligo: &mut [u8], longo: &mut usize) -> Result<(), VojaEraro> {
    let bezonata = longo.saturating_add(1).saturating_add(peco.len());
    if bezonata > eligo.len() {
        return Err(VojaEraro::TroLonga);
    }
    eligo[*longo] = b'/';
    let komenco = *longo + 1;
    eligo[komenco..bezonata].copy_from_slice(peco.as_bytes());
    *longo = bezonata;
    Ok(())
}

fn repreni(eligo: &mut [u8], longo: &mut usize) {
    if let Some(indekso) = eligo[..*longo].iter().rposition(|bajto| *bajto == b'/') {
        *longo = indekso;
    }
}

#[cfg(test)]
mod testoj {
    use super::*;

    fn solvi(nuna: &str, vojo: &str) -> String {
        let mut bufro = [0u8; VOJA_MAKSIMO];
        let longo = kunigi(nuna, vojo, &mut bufro).expect("la vojo solveblas");
        core::str::from_utf8(&bufro[..longo])
            .expect("la vojo restas UTF-8")
            .to_string()
    }

    fn invito(nuna: &str) -> String {
        let mut bufro = [0u8; VOJA_MAKSIMO];
        let longo = por_invito(nuna, &mut bufro).expect("la invito enas");
        core::str::from_utf8(&bufro[..longo])
            .expect("la invito restas UTF-8")
            .to_string()
    }

    #[test]
    fn relativa_kreskas() {
        assert_eq!(solvi(HEJMO, "notoj"), "/hejmo/notoj");
        assert_eq!(solvi(HEJMO, "notoj/unua.txt"), "/hejmo/notoj/unua.txt");
        assert_eq!(solvi("/", "saluto.elf"), "/saluto.elf");
    }

    #[test]
    fn absoluta_ignoras_nunan() {
        assert_eq!(solvi("/hejmo/notoj", "/bonveno.txt"), "/bonveno.txt");
        assert_eq!(solvi("/hejmo", "/"), "/");
    }

    #[test]
    fn patro_restas_en_radiko() {
        assert_eq!(solvi("/hejmo/notoj", ".."), "/hejmo");
        assert_eq!(solvi("/hejmo", "../../.."), "/");
        assert_eq!(solvi("/", ".."), "/");
    }

    #[test]
    fn punktoj_kaj_oblaj_skladoj_malaperas() {
        assert_eq!(solvi("/hejmo", "./notoj/."), "/hejmo/notoj");
        assert_eq!(solvi("/hejmo", "notoj//unua.txt/"), "/hejmo/notoj/unua.txt");
        assert_eq!(solvi("/hejmo", "//notoj"), "/notoj");
    }

    #[test]
    fn tilde_estas_hejmo() {
        assert_eq!(solvi("/", "~"), HEJMO);
        assert_eq!(solvi("/bonveno.txt", "~/notoj"), "/hejmo/notoj");
        assert_eq!(solvi(HEJMO, "~/../.."), "/");
    }

    #[test]
    fn nevalida_tilde_estas_eraro() {
        let mut bufro = [0u8; VOJA_MAKSIMO];
        assert_eq!(
            kunigi("/", "~uzanto", &mut bufro),
            Err(VojaEraro::NevalidaTilde)
        );
    }

    #[test]
    fn neabsoluta_nuna_estas_eraro() {
        let mut bufro = [0u8; VOJA_MAKSIMO];
        assert_eq!(
            kunigi("hejmo", "notoj", &mut bufro),
            Err(VojaEraro::Neabsoluta)
        );
    }

    #[test]
    fn tro_longaj_vojoj_estas_rifuzataj() {
        let mut bufro = [0u8; 8];
        assert_eq!(
            kunigi("/", "notoj/unua.txt", &mut bufro),
            Err(VojaEraro::TroLonga)
        );
    }

    #[test]
    fn utf8_restas_sendifekta() {
        assert_eq!(solvi(HEJMO, "ĉambro/noto"), "/hejmo/ĉambro/noto");
    }

    #[test]
    fn invito_montras_tildon_nur_en_hejmo() {
        assert_eq!(invito(HEJMO), "~");
        assert_eq!(invito("/hejmo/notoj"), "~/notoj");
        assert_eq!(invito("/"), "/");
        assert_eq!(invito("/hejmo2"), "/hejmo2");
        assert_eq!(invito("/notoj"), "/notoj");
    }
}
