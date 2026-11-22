//! Arkitekture sendependa logiko de la kerno.
//!
//! Ĉi tiu biblioteko tenas la purajn datumstrukturojn kaj algoritmojn, kiuj
//! ne bezonas aparataron: fizika memoro, virtualaj adresoj, pretaj vicoj,
//! mesaĝujoj, vekotemporiziloj, la SNDS-analizo kaj la voja solvado. La kerno
//! kaj la peraj manoj (`Ring 3`) uzas la samajn tipojn; ĉar la biblioteko ne
//! dependas de `std`, ĝi ankaŭ kompiliĝas por la nudaj celoj.
//!
//! La testoj rulas sur la gastiganto:
//!
//! ```text
//! cargo test -p kernologiko --target $(rustc -vV | sed -n 's/^host: //p')
//! ```
//!
//! `just testoj` faras tion por la nuna gastiganto; `just kontroli` inkluzivas
//! la vokon, do la DI ankaŭ kontrolas ĝin.

#![cfg_attr(not(test), no_std)]

pub mod adreso;
pub mod memoro;
pub mod mesagujo;
pub mod pagado;
pub mod preta_vico;
pub mod snds;
pub mod vekotemporizilo;
pub mod vojo;

/// Fadena identigilo; `0` estas la ripoza kunteksto de la planisto.
pub type FadenoId = u64;

/// La maksimuma nombro da planeblaj fadenoj. La pretaj vicoj kaj la
/// vekotemporizilo havas fiksan kapaciton bazitan sur ĝi.
pub const MAKSAJ_FADENOJ: usize = 8;
