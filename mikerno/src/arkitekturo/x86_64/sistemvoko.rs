//! La `int 0x80`-eniro: instalas la traktilon kaj transdonas la kadron al la
//! komuna sistemvoka dispetĉo.

use crate::arkitekturo::{InterrompaKadro, SistemvokaKadro};
use crate::interrompo::{InterrompaTraktilo, SISTEMVOKA_VEKTORO};
use crate::kerno::sistemvoko as kerna;

/// Vokata de la `0x80`-traktilo post kiam `isr_komuna` konstruis la kadron.
pub fn sistemvoka_dispetĉo(kadro: &mut dyn SistemvokaKadro) {
    // Ring 3-vokantoj bezonas montrilan kontrolon; Ring 0 estas fidinda.
    let el_ring3 = kadro.el_uzanto();
    let rezulto = kerna::voki(
        kadro.numero(),
        kadro.argumento(0),
        kadro.argumento(1),
        kadro.argumento(2),
        kadro.argumento(3),
        el_ring3,
    );
    // NE FORGESU: `iretq` restarigas la rezultregistron el ĉi tiu kadro; sen la
    // skribo la vokanto ĉiam vidus la malnovan numeron.
    kadro.agordi_rezulton(rezulto);
}

struct SistemvokaTraktilo;

impl InterrompaTraktilo for SistemvokaTraktilo {
    fn trakta(&self, kadro: &mut InterrompaKadro) {
        sistemvoka_dispetĉo(kadro);
    }
}

static TRAKTILO: SistemvokaTraktilo = SistemvokaTraktilo;

/// Registru la traktilon de la sistemvoka vektoro.
pub fn instali() {
    crate::interrompo::registri(SISTEMVOKA_VEKTORO, &TRAKTILO);
    crate::skribu!("[sistemvoko] pordego 0x80 (DPL3) instalita\n");
}
