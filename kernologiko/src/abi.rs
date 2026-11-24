//! La ABI 2-kontrakto: grandezoj, versio kaj la dudek voknumeroj.
//!
//! `libmikerno` asertas la aranĝon dum la kompilado; ĉi tiuj testoj kontrolas
//! la samajn valorojn sur la gastiganto kaj fiksas la voknumerojn. La dua
//! testo legas la realajn konstantojn de `libmikerno` kaj pruvas, ke la dudek
//! numeroj estas unikaj kaj kontinuaj.

use core::mem::{align_of, size_of};

use libmikerno::syscall::{
    SYS_BRK, SYS_CEDI, SYS_DONI, SYS_DORMI, SYS_ENIGI, SYS_EXEC, SYS_FERMI, SYS_FINI,
    SYS_KREI_FINPUNKTON, SYS_LANĜI, SYS_LEGI, SYS_LISTI, SYS_MORTIGI, SYS_PROCEZOJ, SYS_REVOKI,
    SYS_RICEVI, SYS_RICEVI_TEMPO, SYS_SENDI, SYS_SENDI_TEMPO, SYS_SKRIBI,
};
use libmikerno::{ABI_VERSIO, Mesago, Tenilo};

#[test]
fn la_komuna_aranĝo_estas_fiksita() {
    assert_eq!(size_of::<Tenilo>(), 4);
    assert_eq!(align_of::<Tenilo>(), 4);
    assert_eq!(size_of::<Mesago>(), 96);
    assert_eq!(ABI_VERSIO, 2);
}

#[test]
fn la_dudek_numeroj_estas_unikaj_kaj_kontinuaj() {
    let numeroj = [
        SYS_SENDI,
        SYS_RICEVI,
        SYS_SKRIBI,
        SYS_FINI,
        SYS_CEDI,
        SYS_KREI_FINPUNKTON,
        SYS_DONI,
        SYS_REVOKI,
        SYS_RICEVI_TEMPO,
        SYS_SENDI_TEMPO,
        SYS_FERMI,
        SYS_DORMI,
        SYS_BRK,
        SYS_EXEC,
        SYS_ENIGI,
        SYS_LISTI,
        SYS_LEGI,
        SYS_PROCEZOJ,
        SYS_MORTIGI,
        SYS_LANĜI,
    ];
    let mut ordigitaj = numeroj;
    ordigitaj.sort_unstable();
    assert_eq!(
        ordigitaj,
        core::array::from_fn(|indekso: usize| indekso + 1)
    );
}
