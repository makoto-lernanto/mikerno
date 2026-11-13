//! IPC-mesaĝo: fiksa `#[repr(C)]` aranĝo, kiun ambaŭ flankoj kopias sen konverto.

/// Maksimuma longo de la datumaro en unu mesaĝo.
pub const MAKSA_MESAGA_LONGO: usize = 64;

/// Enlinia mesaĝo inter du finpunktoj.
///
/// `sendinto` estas la fadena identigilo de la sendanto. `ricevonto` estas
/// hodiaŭ la celo-fadeno, sed ekde la finpunktaj objektoj (Fazo 2 de M14) ĝi
/// estos la finpunkta identigilo; la aranĝo restas 96 bajtoj. La kerno
/// plenigas ambaŭ kampojn dum la sendo, do la uzanto ne povas mensogi pri ili.
/// La datumaro estas kopiata, do la sendanto rajtas reuzi sian bufron tuj post
/// la voko.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Mesago {
    pub tipo: u64,
    pub sendinto: u64,
    pub ricevonto: u64,
    /// Publika por la ABI-layoutaj asertoj; uzu `nova` por valida mesaĝo.
    pub longo: usize,
    datumo: [u8; MAKSA_MESAGA_LONGO],
}

impl Mesago {
    /// # Paniko
    /// Se `enhavo` superas `MAKSA_MESAGA_LONGO`.
    pub const fn nova(tipo: u64, sendinto: u64, ricevonto: u64, enhavo: &[u8]) -> Self {
        assert!(
            enhavo.len() <= MAKSA_MESAGA_LONGO,
            "mesaĝo superas MAKSA_MESAGA_LONGO"
        );
        let mut bufro = [0u8; MAKSA_MESAGA_LONGO];
        let mut indekso = 0;
        while indekso < enhavo.len() {
            bufro[indekso] = enhavo[indekso];
            indekso += 1;
        }
        Self {
            tipo,
            sendinto,
            ricevonto,
            longo: enhavo.len(),
            datumo: bufro,
        }
    }

    pub const fn malplena() -> Self {
        Self {
            tipo: 0,
            sendinto: 0,
            ricevonto: 0,
            longo: 0,
            datumo: [0; MAKSA_MESAGA_LONGO],
        }
    }

    pub const fn longo(&self) -> usize {
        self.longo
    }

    pub fn enhavo(&self) -> &[u8] {
        &self.datumo[..self.longo]
    }

    /// La tuta fiksa datumaro, inkluzive de la nula remburgo post `longo`.
    pub const fn datumaro(&self) -> &[u8; MAKSA_MESAGA_LONGO] {
        &self.datumo
    }
}

const _: () = {
    assert!(core::mem::size_of::<Mesago>() == 96);
    assert!(core::mem::align_of::<Mesago>() == 8);
    assert!(core::mem::offset_of!(Mesago, tipo) == 0);
    assert!(core::mem::offset_of!(Mesago, sendinto) == 8);
    assert!(core::mem::offset_of!(Mesago, ricevonto) == 16);
    assert!(core::mem::offset_of!(Mesago, longo) == 24);
};
