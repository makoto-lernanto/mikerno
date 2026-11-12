//! Legas la registrojn el la interrompa kadro kaj dispetĉas al la kerna
//! efektivigo de la sistemvokoj.

use crate::interrompo::InterrompaKadro;
use crate::kerno::sistemvoko as kerna;

/// Vokata de la `0x80`-traktilo post kiam `isr_komuna` konstruis la kadron.
pub fn sistemvoka_dispetĉo(kadro: &mut InterrompaKadro) {
    // Ring 3-vokantoj bezonas montrilan kontrolon; Ring 0 estas fidinda.
    let el_ring3 = kadro.cs & 3 == 3;
    let rezulto = kerna::voki(
        kadro.rax as usize,
        kadro.rdi,
        kadro.rsi,
        kadro.rdx,
        kadro.r10,
        el_ring3,
    );
    // NE FORGESU: `iretq` restarigas `rax` el ĉi tiu kadro; sen la skribo la
    // vokanto ĉiam vidus la malnovan numeron.
    kadro.rax = rezulto;
}
