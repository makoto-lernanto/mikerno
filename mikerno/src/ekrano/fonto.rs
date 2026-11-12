//! Bitmapaj tiparoj por la framebuffer-konzolo.
//!
//! La glifoj estas antaŭe rasterigitaj el **DejaVu Sans Mono** en kvar
//! memstarajn strekojn: 8x16, 12x24, 16x32 kaj 32x64. Ĉiu streko havas sian
//! propran pikselgrandecon, do la konzolo neniam skalfaktoras bitbildon.
//!
//! La bildigilo estas `tools/font/generi.py`; la fontdosiero mem ne estas
//! konservata en la deponejo. Vidu `third_party/dejavu/LICENSE.txt`.
//!
//! Ĉiu streko enhavas 111 glifojn en la sama ordo: unue ASCII 0x20..=0x7E,
//! poste la Esperantaj literoj `ĉ Ĉ ĝ Ĝ ĥ Ĥ ĵ Ĵ ŝ Ŝ ŭ Ŭ` kaj la citiloj
//! `« » ‹ ›`. Vicoj estas sinsekvaj, plej supra bito = plej maldekstra pikselo.

const GLIFOJ: usize = 111;
const ALDONAJ: [char; 16] = [
    'ĉ', 'Ĉ', 'ĝ', 'Ĝ', 'ĥ', 'Ĥ', 'ĵ', 'Ĵ', 'ŝ', 'Ŝ', 'ŭ', 'Ŭ', '«', '»', '‹', '›',
];

static TIPARO_8X16: &[u8] = include_bytes!("fonto/tiparo-8x16.bin");
static TIPARO_12X24: &[u8] = include_bytes!("fonto/tiparo-12x24.bin");
static TIPARO_16X32: &[u8] = include_bytes!("fonto/tiparo-16x32.bin");
static TIPARO_32X64: &[u8] = include_bytes!("fonto/tiparo-32x64.bin");

const _: () = {
    assert!(TIPARO_8X16.len() == GLIFOJ * 16);
    assert!(TIPARO_12X24.len() == GLIFOJ * 24 * 2);
    assert!(TIPARO_16X32.len() == GLIFOJ * 32 * 2);
    assert!(TIPARO_32X64.len() == GLIFOJ * 64 * 4);
};

/// Unu piksela streko de la tiparo.
pub struct Tiparo {
    pub larĝo: usize,
    pub alto: usize,
    /// Bajtoj por unu vico; ĉiam `(larĝo + 7) / 8`.
    pub streko: usize,
    glifoj: &'static [u8],
}

/// La kvar strekoj, de la plej malgranda al la plej granda.
pub static TIPAROJ: [Tiparo; 4] = [
    Tiparo {
        larĝo: 8,
        alto: 16,
        streko: 1,
        glifoj: TIPARO_8X16,
    },
    Tiparo {
        larĝo: 12,
        alto: 24,
        streko: 2,
        glifoj: TIPARO_12X24,
    },
    Tiparo {
        larĝo: 16,
        alto: 32,
        streko: 2,
        glifoj: TIPARO_16X32,
    },
    Tiparo {
        larĝo: 32,
        alto: 64,
        streko: 4,
        glifoj: TIPARO_32X64,
    },
];

fn indekso(signo: char) -> usize {
    if (' '..='~').contains(&signo) {
        signo as usize - ' ' as usize
    } else {
        ALDONAJ
            .iter()
            .position(|s| *s == signo)
            .map_or('?' as usize - ' ' as usize, |indekso| 95 + indekso)
    }
}

/// Liveru la bitojn de `signo` en `tiparo`; nekonataj signoj fariĝas `?`.
pub fn glifo(tiparo: &'static Tiparo, signo: char) -> &'static [u8] {
    let longo = tiparo.alto * tiparo.streko;
    let komenco = indekso(signo) * longo;
    &tiparo.glifoj[komenco..komenco + longo]
}
