//! Framebuffer-teksta konzolo: bitmapaj glifoj desegnitaj rekte en la
//! framebuffer, kiun la praŝarĝilo liveras (Limine per Multiboot2) aŭ, sur
//! `arm64`, kiun la kerno anoncas al `ramfb`.
//!
//! La konzolo elektas inter la kvar strekoj de `fonto` laŭ la framebuffer-
//! grando: ju pli alta la ekrano, des pli granda la tiparo — ĉiam kun proksimume
//! 33 tekstovicoj. Tiel la teksto restas akra kaj legebla sen skalado.
//!
//! Se framebuffer mankas (Xen-PVH aŭ malnova firmvaro), la konzolo restas
//! malŝaltita kaj la eligo okazas nur sur la seria konzolo.

use core::fmt::{self, Write};
use core::ptr;

use crate::kunhavigita::Kunhavigita;

mod fonto;

const MINIMUMA_LARĜO: u32 = 320;
const MINIMUMA_ALTO: u32 = 32;
/// Celo: proksimume 33 tekstovicoj, kiel sur kutima terminalo.
const CELA_VICOJ: usize = 33;
/// Neniu tiparo rajtas produkti malpli ol 80 kolumnojn aŭ 24 vicojn.
const MINIMUMAJ_KOLUMNOJ: usize = 80;
const MINIMUMAJ_VICOJ: usize = 24;
const FONA_RGB: u32 = 0x0000_0022; // #000022
const TEKSTA_RGB: u32 = 0x00FF_FFFF; // #FFFFFF
const TITOLA_RGB: u32 = 0x0000_FF00; // #00FF00
const TITOLO: &str = "MiKerno: La estonteco nun.\n";

#[derive(Clone, Copy)]
pub struct FramebufferInformo {
    pub bazo: u64,
    pub larĝo: u32,
    pub alto: u32,
    pub paŝo: u32,
    pub bpp: u8,
    pub ruĝa: u8,
    pub verda: u8,
    pub blua: u8,
}

struct Ekrano {
    informo: FramebufferInformo,
    tiparo: &'static fonto::Tiparo,
    kolumnoj: usize,
    vicoj: usize,
    x: usize,
    y: usize,
}

static EKRANO: Kunhavigita<Option<Ekrano>> = Kunhavigita::nova(None);

/// Elektu la strekon kun almenaŭ 80 kolumnoj kaj 24 vicoj, kies vicoj plej
/// proksimas al 33. Se neniu taŭgas, uzu la plej malgrandan.
fn elekti_tiparon(larĝo: u32, alto: u32) -> &'static fonto::Tiparo {
    let mut elektita = &fonto::TIPAROJ[0];
    let mut diferenco = usize::MAX;
    for tiparo in fonto::TIPAROJ.iter() {
        let kolumnoj = larĝo as usize / tiparo.larĝo;
        let vicoj = alto as usize / tiparo.alto;
        if kolumnoj < MINIMUMAJ_KOLUMNOJ || vicoj < MINIMUMAJ_VICOJ {
            continue;
        }
        let nuna = vicoj.abs_diff(CELA_VICOJ);
        if nuna < diferenco {
            elektita = tiparo;
            diferenco = nuna;
        }
    }
    elektita
}

/// Ŝaltu la framebuffer-konzolon; `false` se la framebuffer ne taŭgas.
pub fn pravalorizi(informo: FramebufferInformo) -> bool {
    if informo.bazo == 0
        || !informo.bazo.is_multiple_of(4)
        || informo.bpp != 32
        || informo.larĝo < MINIMUMA_LARĜO
        || informo.alto < MINIMUMA_ALTO
        || (informo.paŝo as u64) < informo.larĝo as u64 * 4
        || [informo.ruĝa, informo.verda, informo.blua]
            .iter()
            .any(|pozicio| *pozicio >= 32)
    {
        return false;
    }
    let tiparo = elekti_tiparon(informo.larĝo, informo.alto);
    let mut ekrano = Ekrano {
        informo,
        tiparo,
        kolumnoj: informo.larĝo as usize / tiparo.larĝo,
        vicoj: informo.alto as usize / tiparo.alto,
        x: 0,
        y: 0,
    };
    ekrano.malplenigi();
    *EKRANO.akiri() = Some(ekrano);
    true
}

/// Skribu al la framebuffer, se ĝi estas ŝaltita. Vokata sub interrompa gardo.
pub fn skribi_formaton(argumentoj: fmt::Arguments) {
    if let Some(ekrano) = EKRANO.akiri().as_mut() {
        let _ = ekrano.write_fmt(argumentoj);
    }
}

/// Rezulto de la piksela fiksaĵo, por la kompara linio de la starto.
pub struct PikselaRaporto {
    pub larĝo: u32,
    pub alto: u32,
    pub bpp: u8,
    pub tiparo: (usize, usize),
    pub haketo: u64,
}

/// Desegnu determinan provbildon en la tutan frambufferon, resumu la pikselojn
/// kaj restarigu puran paĝon. `None` se la konzolo ne estas ŝaltita.
///
/// La bildo dependas nur de la framebuffer-grando, de la elektita tiparo kaj de
/// la koloraj deŝovoj, do ĝia haketo estas komparebla inter la platformoj kiam
/// la geometrio kongruas.
pub fn testi_pikselojn() -> Option<PikselaRaporto> {
    let ekrano = EKRANO.akiri().as_mut()?;
    Some(ekrano.testi_pikselojn())
}

impl Ekrano {
    fn pikselo(&self, x: usize, y: usize, koloro: u32) {
        let adreso = self.informo.bazo + y as u64 * self.informo.paŝo as u64 + x as u64 * 4;
        unsafe { ptr::write_volatile(adreso as *mut u32, koloro) };
    }

    fn koloro(&self, rgb: u32) -> u32 {
        (((rgb >> 16) & 0xFF) << self.informo.ruĝa)
            | (((rgb >> 8) & 0xFF) << self.informo.verda)
            | ((rgb & 0xFF) << self.informo.blua)
    }

    fn plenigi(&mut self, rgb: u32) {
        let koloro = self.koloro(rgb);
        for y in 0..self.informo.alto as usize {
            for x in 0..self.informo.larĝo as usize {
                self.pikselo(x, y, koloro);
            }
        }
    }

    fn malplenigi(&mut self) {
        self.plenigi(FONA_RGB);
        self.x = 0;
        self.y = 0;
        self.skribi(TITOLO, TITOLA_RGB);
    }

    fn novlinio(&mut self) {
        self.x = 0;
        self.y += 1;
        if self.y >= self.vicoj {
            self.malplenigi();
        }
    }

    fn skribi_signon(&mut self, signo: char, rgb: u32) {
        if signo == '\n' {
            self.novlinio();
            return;
        }
        if signo == '\r' {
            self.x = 0;
            return;
        }
        if self.x >= self.kolumnoj {
            self.novlinio();
        }
        let fono = self.koloro(FONA_RGB);
        let antaŭo = self.koloro(rgb);
        let tiparo = self.tiparo;
        let glifo = fonto::glifo(tiparo, signo);
        let baza_x = self.x * tiparo.larĝo;
        let baza_y = self.y * tiparo.alto;
        for vico in 0..tiparo.alto {
            let bitoj = &glifo[vico * tiparo.streko..(vico + 1) * tiparo.streko];
            for kolumno in 0..tiparo.larĝo {
                let koloro = if bitoj[kolumno / 8] & (0x80 >> (kolumno % 8)) != 0 {
                    antaŭo
                } else {
                    fono
                };
                self.pikselo(baza_x + kolumno, baza_y + vico, koloro);
            }
        }
        self.x += 1;
    }

    fn skribi(&mut self, teksto: &str, rgb: u32) {
        for signo in teksto.chars() {
            self.skribi_signon(signo, rgb);
        }
    }

    /// Determina provbildo: kolora gradiento kun teksto super ĝi.
    fn ŝablono(&mut self) {
        let larĝo = self.informo.larĝo.max(2);
        let alto = self.informo.alto.max(2);
        for y in 0..self.informo.alto as usize {
            let verda = (y as u32 * 255 / (alto - 1)) & 0xFF;
            for x in 0..self.informo.larĝo as usize {
                let ruĝa = (x as u32 * 255 / (larĝo - 1)) & 0xFF;
                let blua = (x as u32 ^ y as u32) & 0xFF;
                let koloro = self.koloro((ruĝa << 16) | (verda << 8) | blua);
                self.pikselo(x, y, koloro);
            }
        }
        self.x = 0;
        self.y = 0;
        self.skribi("MiKerno\npiksela fiksaĵo\n", TITOLA_RGB);
    }

    /// FNV-1a 64 super ĉiuj pikseloj de la videbla regiono.
    fn haketi(&self) -> u64 {
        let mut haketo = 0xcbf2_9ce4_8422_2325u64;
        for y in 0..self.informo.alto as usize {
            let vico = self.informo.bazo + y as u64 * self.informo.paŝo as u64;
            for x in 0..self.informo.larĝo as usize {
                // SAFETY: la adreso kuŝas en la mapita frambufferio; la legado
                // estas flikema, por ke la kompililo ne forigu ĝin.
                let valoro = unsafe { ptr::read_volatile((vico + x as u64 * 4) as *const u32) };
                haketo ^= valoro as u64;
                haketo = haketo.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        haketo
    }

    fn testi_pikselojn(&mut self) -> PikselaRaporto {
        self.ŝablono();
        let haketo = self.haketi();
        // Restarigu puran paĝon, por ke la sekva teksto (la raporta linio)
        // aperu sur ordinara fono.
        self.malplenigi();
        PikselaRaporto {
            larĝo: self.informo.larĝo,
            alto: self.informo.alto,
            bpp: self.informo.bpp,
            tiparo: (self.tiparo.larĝo, self.tiparo.alto),
            haketo,
        }
    }
}

impl fmt::Write for Ekrano {
    fn write_str(&mut self, teksto: &str) -> fmt::Result {
        self.skribi(teksto, TEKSTA_RGB);
        Ok(())
    }
}
