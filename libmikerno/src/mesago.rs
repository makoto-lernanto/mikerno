//! IPC-mesaĝo: fiksa `#[repr(C)]` aranĝo, kiun ambaŭ flankoj kopias sen konverto.

/// Maksimuma longo de la datumaro en unu mesaĝo.
pub const MAKSA_MESAGA_LONGO: usize = 64;

/// Enlinia mesaĝo inter du finpunktoj.
///
/// `sendinto` kaj `ricevonto` estas fadenaj identigiloj, kiujn la kerno
/// plenigas dum la sendo; la mesaĝujo ankoraŭ konservas la fonton flanke por
/// filtrado. La datumaro estas kopiata, do la sendanto rajtas reuzi sian
/// bufron tuj post la voko.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Mesago {
    pub tipo: u64,
    pub sendinto: u64,
    pub ricevonto: u64,
    longo: usize,
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
}
