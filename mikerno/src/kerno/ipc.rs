//! Minimuma mesaĝa IPC: finpunktoj, kapabloj, mesagujo.
//!
//! `sendi_mesagon` metas la mesaĝon en la vicon de la celo kaj vekas ĝin se ĝi
//! estis blokita. `ricevi_mesagon` blokas la vokanton ĝis mesaĝo de la donita
//! fonto alvenas. Ĉiuj operacioj tenas interrompojn malŝaltitaj.

use crate::interrompo::InterrompaGardo;
use crate::kunhavigita::Kunhavigita;

use super::objekto::FadenoId;
use super::planisto::{self, MAKSAJ_FADENOJ};

const MAKSAJ_MESAGOJ: usize = 16;

/// Mesaĝa datumaro estas kopiata en la mesaĝon, do la sendanto rajtas reuzi
/// sian bufron tuj post la voko.
pub const MAKSA_MESAGA_LONGO: usize = 64;

#[derive(Clone, Copy)]
pub struct Permesoj(u8);

impl Permesoj {
    pub const SENDI: Self = Self(1);
    pub const RICEVI: Self = Self(2);
    pub const ĈIUJ: Self = Self(Self::SENDI.0 | Self::RICEVI.0);

    pub const fn posedas(&self, permeso: Self) -> bool {
        self.0 & permeso.0 == permeso.0
    }
}

#[derive(Clone, Copy)]
pub struct FinaPunkto {
    pub id: u64,
    pub permesoj: Permesoj,
}

impl FinaPunkto {
    pub const fn nova(id: u64, permesoj: Permesoj) -> Self {
        Self { id, permesoj }
    }

    pub const fn kapablo(&self) -> Kapablo {
        Kapablo {
            objekto_id: self.id,
            permesoj: self.permesoj,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Kapablo {
    pub objekto_id: u64,
    pub permesoj: Permesoj,
}

#[derive(Clone, Copy)]
pub struct Mesago {
    pub tipo: u64,
    longo: usize,
    datumo: [u8; MAKSA_MESAGA_LONGO],
}

impl Mesago {
    /// # Paniko
    /// Se `datumoj` superas `MAKSA_MESAGA_LONGO`.
    pub const fn nova(tipo: u64, datumoj: &[u8]) -> Self {
        assert!(
            datumoj.len() <= MAKSA_MESAGA_LONGO,
            "mesaĝo superas MAKSA_MESAGA_LONGO"
        );
        let mut bufro = [0u8; MAKSA_MESAGA_LONGO];
        let mut indekso = 0;
        while indekso < datumoj.len() {
            bufro[indekso] = datumoj[indekso];
            indekso += 1;
        }
        Self {
            tipo,
            longo: datumoj.len(),
            datumo: bufro,
        }
    }

    pub const fn malplena() -> Self {
        Self {
            tipo: 0,
            longo: 0,
            datumo: [0; MAKSA_MESAGA_LONGO],
        }
    }

    pub fn enhavo(&self) -> &[u8] {
        &self.datumo[..self.longo]
    }
}

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
/// # Eraroj
/// `NevalidaCelloko` se la destino ne estas fadena identigilo; `MesagujoPlena`
/// se la vico de la celo jam enhavas `MAKSAJ_MESAGOJ` mesaĝojn.
pub fn sendi_mesagon(destinacio: FadenoId, mesago: &Mesago) -> Result<(), IpcEraro> {
    let _gardo = InterrompaGardo::nova();
    if destinacio as usize > MAKSAJ_FADENOJ {
        return Err(IpcEraro::NevalidaCelloko);
    }
    mesagujo(destinacio).pusxi(Ero {
        fonto: planisto::nuna_id(),
        mesago: *mesago,
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

    use crate::kerno::objekto::FadenoId;

    use super::{Ero, IpcEraro, MAKSA_MESAGA_LONGO, MAKSAJ_MESAGOJ, Mesago, Mesagujo};

    /// Arbitra mesaĝo kun fiksa `tipo`; la enhavo estas arbitra, sed `longo`
    /// restas en la limoj de la bufero.
    fn iu_mesago(tipo: u64) -> Mesago {
        let longo: usize = kani::any();
        kani::assume(longo <= MAKSA_MESAGA_LONGO);
        Mesago {
            tipo,
            longo,
            datumo: kani::any(),
        }
    }

    fn iu_ero(tipo: u64) -> Ero {
        Ero {
            fonto: kani::any(),
            mesago: iu_mesago(tipo),
        }
    }

    /// Mesaĝa `longo` neniam eliras la datumaron.
    #[kani::proof]
    fn mesaĝa_enhavo_en_limoj() {
        let mesago = iu_mesago(kani::any());
        assert_eq!(mesago.enhavo().len(), mesago.longo);
        assert!(mesago.longo <= MAKSA_MESAGA_LONGO);
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
    #[kani::unwind(10)]
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
        assert_eq!(ricevita.longo, unua.longo);

        let ricevita = ujo.preni_de(fonto).unwrap();
        assert_eq!(ricevita.tipo, 2);
        assert_eq!(ricevita.longo, dua.longo);

        assert_eq!(ujo.nombro, 0);
        assert!(ujo.preni_de(fonto).is_none());
        assert!(ujo.eroj[0].is_none());
    }

    /// FIFO restas valida ankaŭ inter malsamaj fontoj: forpreno de unu fonto
    /// ne renversas la ordon de la aliaj.
    #[kani::proof]
    #[kani::unwind(10)]
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
