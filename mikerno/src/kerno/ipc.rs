//! Minimuma mesaĝa IPC: finpunktoj, kapabloj, mesagujo.
//!
//! `sendi_mesagon` metas la mesaĝon en la vicon de la celo kaj vekas ĝin se ĝi
//! estis blokita. `ricevi_mesagon` blokas la vokanton ĝis mesaĝo de la donita
//! fonto alvenas. Ĉiuj operacioj tenas interrompojn malŝaltitaj.

use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;

use super::objekto::FadenoId;
use super::planisto::{self, MAKSAJ_FADENOJ};

/// La mesaĝa aranĝo, la permesoj kaj la kapabloj venas el `libmikerno`, do la
/// kerno kaj Ring 3 uzas la saman `#[repr(C)]`-strukturon sen konverto kaj sen
/// duobla difino.
pub use libmikerno::{FinaPunkto, Kapablo, MAKSA_MESAGA_LONGO, Mesago, Permesoj};

/// La kongruo kun la komuna ABI estas fiksita je kompila tempo.
const _: () = {
    assert!(libmikerno::ABI_VERSIO == 2);
    assert!(core::mem::size_of::<Mesago>() == 96);
    assert!(core::mem::align_of::<Mesago>() == 8);
    assert!(core::mem::offset_of!(Mesago, tipo) == 0);
    assert!(core::mem::offset_of!(Mesago, sendinto) == 8);
    assert!(core::mem::offset_of!(Mesago, ricevonto) == 16);
    assert!(core::mem::offset_of!(Mesago, longo) == 24);
};

const MAKSAJ_MESAGOJ: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcEraro {
    NevalidaCelloko,
    MesagujoPlena,
}

#[derive(Clone, Copy)]
struct Ero {
    fonto: FadenoId,
    mesago: Mesago,
}

#[derive(Clone, Copy)]
struct Mesagujo {
    eroj: [Option<Ero>; MAKSAJ_MESAGOJ],
    nombro: usize,
}

impl Mesagujo {
    const fn nova() -> Self {
        Self {
            eroj: [None; MAKSAJ_MESAGOJ],
            nombro: 0,
        }
    }

    fn pusxi(&mut self, ero: Ero) -> Result<(), IpcEraro> {
        if self.nombro >= MAKSAJ_MESAGOJ {
            return Err(IpcEraro::MesagujoPlena);
        }
        self.eroj[self.nombro] = Some(ero);
        self.nombro += 1;
        Ok(())
    }

    fn preni_de(&mut self, fonto: FadenoId) -> Option<Mesago> {
        let indekso = (0..self.nombro).find(|&i| self.eroj[i].is_some_and(|e| e.fonto == fonto))?;
        let ero = self.eroj[indekso].take()?;
        for movo in indekso..self.nombro - 1 {
            self.eroj[movo] = self.eroj[movo + 1];
        }
        self.nombro -= 1;
        self.eroj[self.nombro] = None;
        Some(ero.mesago)
    }
}

static MESAGUJOJ: Kunhavigita<[Mesagujo; MAKSAJ_FADENOJ + 1]> =
    Kunhavigita::nova([Mesagujo::nova(); MAKSAJ_FADENOJ + 1]);

fn mesagujo(id: FadenoId) -> &'static mut Mesagujo {
    &mut MESAGUJOJ.akiri()[id as usize]
}

/// Sendu mesaĝon al fadeno; la fonto estas la nuna fadeno.
///
/// La mesaĝo estas kopiata, kaj `sendinto`/`ricevonto` estas fiksitaj de la
/// kerno: la uzanto ne povas mensogi pri ili.
///
/// # Eraroj
/// `NevalidaCelloko` se la destino ne estas fadena identigilo; `MesagujoPlena`
/// se la vico de la celo jam enhavas `MAKSAJ_MESAGOJ` mesaĝojn.
pub fn sendi_mesagon(destinacio: FadenoId, mesago: &Mesago) -> Result<(), IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    if destinacio as usize > MAKSAJ_FADENOJ {
        return Err(IpcEraro::NevalidaCelloko);
    }
    let nuna = planisto::nuna_id();
    let mut mesago = *mesago;
    mesago.sendinto = nuna;
    mesago.ricevonto = destinacio;
    mesagujo(destinacio).pusxi(Ero {
        fonto: nuna,
        mesago,
    })?;
    planisto::veki(destinacio);
    Ok(())
}

/// Ricevu mesaĝon de la donita fonto; bloku la nunan fadenon se mankas.
pub fn ricevi_mesagon(fonto: FadenoId) -> Mesago {
    loop {
        let _gardo = InterrompaGardo::nova();
        let nuna = planisto::nuna_id();
        if nuna == 0 {
            // La ripoza kunteksto ne havas fadenan vivciklon; ne bloku ĝin.
            return Mesago::malplena();
        }
        if let Some(mesago) = mesagujo(nuna).preni_de(fonto) {
            return mesago;
        }
        planisto::bloki();
    }
}

#[cfg(kani)]
mod kani_proofs {
    //! Kani-pruvoj por la mesaĝujo.
    //!
    //! `sendi_mesagon` kaj `ricevi_mesagon` dependas de la planisto, de
    //! interrompoj kaj de tutmonda stato, kiuj ne estas modeligeblaj per Kani
    //! sen eliri la no_std-medion. La pruvoj do celas la enan `Mesagujo`-kodon,
    //! kiun ambaŭ funkcioj rekte uzas.
    //!
    //! La pruvoj uzas la plej malgrandan `unwind`-limon, kiun la koncerna
    //! buklo bezonas: 8 por la FIFO-pruvoj (maksimume tri mesaĝoj kaj tri
    //! `preni_de`-paŝoj), 20 por la plenigo de la 16 fendoj, 70 por la
    //! 64-bajta kopio de `Mesago::nova`. La mesaĝoj de la vicaj pruvoj havas
    //! malplenan datumaron, ĉar la vica ordo dependas nur de `tipo` kaj de la
    //! fonto; la plenlongan buferlimon aparte pruvas `mesaĝa_enhavo_en_limoj`.

    use crate::kerno::objekto::FadenoId;

    use super::{Ero, IpcEraro, MAKSA_MESAGA_LONGO, MAKSAJ_MESAGOJ, Mesago, Mesagujo};

    /// Arbitra mesaĝo kun fiksa `tipo`. La vicaj pruvoj nur dependas de `tipo`
    /// kaj de la fonto, do la datumaro restas malplena kaj la kopi-buklo de
    /// `Mesago::nova` malaperas el la SAT-formulo.
    fn iu_mesago(tipo: u64) -> Mesago {
        Mesago::nova(tipo, 0, 0, &[])
    }

    /// Arbitra mesaĝo, kies `longo` rajtas atingi la realan buferlimon.
    fn plenlonga_mesago() -> Mesago {
        let longo: usize = kani::any();
        kani::assume(longo <= MAKSA_MESAGA_LONGO);
        let datumo: [u8; MAKSA_MESAGA_LONGO] = kani::any();
        Mesago::nova(kani::any(), kani::any(), kani::any(), &datumo[..longo])
    }

    fn iu_ero(tipo: u64) -> Ero {
        Ero {
            fonto: kani::any(),
            mesago: iu_mesago(tipo),
        }
    }

    /// Mesaĝa `longo` neniam eliras la datumaron.
    #[kani::proof]
    #[kani::unwind(70)]
    fn mesaĝa_enhavo_en_limoj() {
        let mesago = plenlonga_mesago();
        assert_eq!(mesago.enhavo().len(), mesago.longo());
        assert!(mesago.longo() <= MAKSA_MESAGA_LONGO);
    }

    /// Plena mesaĝujo rifuzas pliajn mesaĝojn sen ŝanĝi `nombro`.
    #[kani::proof]
    #[kani::unwind(20)]
    fn plena_mesagujo_rifuzas() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();

        let mut indekso = 0;
        while indekso < MAKSAJ_MESAGOJ {
            let ero = Ero {
                fonto,
                mesago: Mesago::malplena(),
            };
            assert!(ujo.pusxi(ero).is_ok());
            indekso += 1;
        }
        assert_eq!(ujo.nombro, MAKSAJ_MESAGOJ);
        assert!(ujo.eroj[0].is_some());

        assert_eq!(ujo.pusxi(iu_ero(1)), Err(IpcEraro::MesagujoPlena));
        assert_eq!(ujo.nombro, MAKSAJ_MESAGOJ);
        assert!(ujo.eroj[0].is_some());
    }

    /// FIFO: de la sama fonto, la unue sendita mesaĝo unue revenas.
    #[kani::proof]
    #[kani::unwind(8)]
    fn fifo_ordo() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();
        let unua = iu_mesago(1);
        let dua = iu_mesago(2);

        assert!(
            ujo.pusxi(Ero {
                fonto,
                mesago: unua
            })
            .is_ok()
        );
        assert!(ujo.pusxi(Ero { fonto, mesago: dua }).is_ok());

        let ricevita = ujo.preni_de(fonto).unwrap();
        assert_eq!(ricevita.tipo, 1);
        assert_eq!(ricevita.longo(), unua.longo());

        let ricevita = ujo.preni_de(fonto).unwrap();
        assert_eq!(ricevita.tipo, 2);
        assert_eq!(ricevita.longo(), dua.longo());

        assert_eq!(ujo.nombro, 0);
        assert!(ujo.preni_de(fonto).is_none());
        assert!(ujo.eroj[0].is_none());
    }

    /// FIFO restas valida ankaŭ inter malsamaj fontoj: forpreno de unu fonto
    /// ne renversas la ordon de la aliaj.
    #[kani::proof]
    #[kani::unwind(8)]
    fn fifo_ordo_inter_fontoj() {
        let mut ujo = Mesagujo::nova();
        let fonto_a: FadenoId = kani::any();
        let fonto_b: FadenoId = kani::any();
        kani::assume(fonto_a != fonto_b);

        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_a,
                mesago: iu_mesago(1),
            })
            .is_ok()
        );
        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_b,
                mesago: iu_mesago(2),
            })
            .is_ok()
        );
        assert!(
            ujo.pusxi(Ero {
                fonto: fonto_a,
                mesago: iu_mesago(3),
            })
            .is_ok()
        );

        assert_eq!(ujo.preni_de(fonto_a).unwrap().tipo, 1);
        assert_eq!(ujo.preni_de(fonto_b).unwrap().tipo, 2);
        assert_eq!(ujo.preni_de(fonto_a).unwrap().tipo, 3);
        assert!(ujo.preni_de(fonto_a).is_none());
        assert!(ujo.preni_de(fonto_b).is_none());
        assert_eq!(ujo.nombro, 0);
    }

    /// Malplena mesaĝujo redonas `None` sen subtrahi de nulo.
    #[kani::proof]
    fn malplena_mesagujo_ricevas_nenion() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();

        assert!(ujo.preni_de(fonto).is_none());
        assert_eq!(ujo.nombro, 0);
        assert!(ujo.eroj[0].is_none());
    }

    /// Forpreno konservas la invariantojn: `nombro` restas en limoj kaj ĉiuj
    /// fendoj ekster `nombro` restas malplenaj.
    #[kani::proof]
    #[kani::unwind(20)]
    fn preno_konservas_la_invarianton() {
        let mut ujo = Mesagujo::nova();
        let fonto: FadenoId = kani::any();
        assert!(ujo.pusxi(iu_ero(1)).is_ok());

        let _ = ujo.preni_de(fonto);
        assert!(ujo.nombro <= MAKSAJ_MESAGOJ);

        let mut indekso = ujo.nombro;
        while indekso < MAKSAJ_MESAGOJ {
            assert!(ujo.eroj[indekso].is_none());
            indekso += 1;
        }
    }
}

#[cfg(kani)]
mod kani_tenilo {
    //! Pruvo de la `Tenilo`-kodigo: valida paro (indekso, generacio) revenas
    //! senŝanĝe, dum generacio `0` kaj indekso super `15` estas rifuzataj — kaj
    //! ĉe la kunmeto kaj ĉe la malpakado de kruda valoro.

    use libmikerno::Tenilo;

    #[kani::proof]
    fn tenilo_kodigo_roundtrip() {
        let indekso: u8 = kani::any();
        let generacio: u32 = kani::any();
        let valida = indekso <= Tenilo::MAKSA_INDEKSO
            && generacio != 0
            && generacio <= Tenilo::MAKSA_GENERACIO;

        match Tenilo::nova(indekso, generacio) {
            Some(tenilo) => {
                assert!(valida);
                assert_eq!(tenilo.indekso(), indekso);
                assert_eq!(tenilo.generacio(), generacio);
                let rekonstruita = Tenilo::el_nombro(tenilo.nombro()).unwrap();
                assert!(rekonstruita == tenilo);
            }
            None => assert!(!valida),
        }

        // Generacio 0 kaj indekso super 15 estas malvalidaj en ambaŭ vojoj.
        assert!(Tenilo::nova(0, 0).is_none());
        assert!(Tenilo::nova(16, 1).is_none());
        assert!(Tenilo::nova(255, 1).is_none());
        assert!(Tenilo::el_nombro(0).is_none());
        assert!(Tenilo::el_nombro((1 << 8) | 16).is_none());

        let kruda: u32 = kani::any();
        match Tenilo::el_nombro(kruda) {
            Some(tenilo) => {
                assert!(tenilo.indekso() <= Tenilo::MAKSA_INDEKSO);
                assert_ne!(tenilo.generacio(), 0);
                assert_eq!(tenilo.nombro(), kruda);
            }
            None => {
                let indekso = (kruda & 0xFF) as u8;
                assert!(indekso > Tenilo::MAKSA_INDEKSO || kruda >> 8 == 0);
            }
        }
    }
}
