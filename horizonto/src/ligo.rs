//! Ligo: la objekta ŝelo de Horizonto.
//!
//! La dukto transdonas tipizitajn rekordojn, ne tekstajn liniojn: `listi` kaj
//! `procezoj` produktas rekordojn, la stadioj (`filtri`, `ordigi`, `unua`,
//! `sumi`, `nombri`, `montri`, `teksto`) ricevas kaj redonas tutan aron, kaj
//! nur la fina montro faras tabelon. `teksto` estas la sola eksplicita limo.
//!
//! La eligo sekvas la ĵetonojn de FluoU (ĉefa teksto, kaska teksto, informo,
//! sukceso, danĝero, oro); la koloroj nur akompanas la signifon, do la sama
//! informo restas legebla sen ili.

use core::cmp::Ordering;
use core::fmt::Write as _;
use core::sync::atomic::{AtomicU8, Ordering as AtomOrdering};

use libmikerno::registro::{
    Dosierero, NOMA_LONGO, PERMESOJ_LONGO, ProcezoRekordo, STATO_KURANTA, STATO_PRETA,
    TIPO_DOSIERUJO,
};

use crate::biblioteko;

/// La versio de la ŝela aspekto (FluoU); sendependa de la kerna versio.
const FLUOU_VERSIO: &str = "0.1";
/// La maksimuma nombro da rekordoj en unu dukta stadio (la kerna limo).
const MAKS_REKORDOJ: usize = libmikerno::registro::MAKS_REGISTROJ;
/// La maksimuma longo de unu eniga linio.
const LINIA_MAKSIMO: usize = 256;
/// La maksimuma dosiero, kiun `legi` montras en unu voko.
const MAKS_LEGO: usize = 4096;
/// La nombro da konservataj enigaj linioj por `↑`/`↓`.
const HISTORIA_LONGO: usize = 8;

const NULIGI: &str = "\x1b[0m";
const GRASA: &str = "\x1b[1m";
const FG_TEKSTO: &str = "\x1b[38;2;238;240;255m";
const FG_TEKSTO2: &str = "\x1b[38;2;167;173;204m";
const FG_TEKSTO3: &str = "\x1b[38;2;109;115;150m";
const FG_NOMO: &str = "\x1b[38;2;201;206;240m";
const FG_INFORMO: &str = "\x1b[38;2;76;201;255m";
const FG_SUKCESO: &str = "\x1b[38;2;87;227;137m";
const FG_DANĜERO: &str = "\x1b[38;2;255;107;107m";
const FG_ORO: &str = "\x1b[38;2;255;180;84m";
const BG_TENILO: &str = "\x1b[48;2;58;46;29m";
const BG_KAPO: &str = "\x1b[48;2;26;31;59m";

/// La ses akcentoj de FluoU; la elektita akcento kolorigas la prompton, la
/// skalaron kaj la karton de `legi`.
const AKCENTOJ: [(&str, &str); 6] = [
    ("oro", FG_ORO),
    ("koralo", "\x1b[38;2;255;92;138m"),
    ("ĉielo", FG_INFORMO),
    ("verdo", FG_SUKCESO),
    ("viola", "\x1b[38;2;167;139;250m"),
    ("bluo", "\x1b[38;2;91;124;250m"),
];

static AKCENTO: AtomicU8 = AtomicU8::new(0);

fn akcenta() -> &'static str {
    let indekso = AKCENTO.load(AtomOrdering::Relaxed) as usize % AKCENTOJ.len();
    AKCENTOJ[indekso].1
}

/// Malgranda teksta bufro por nombroj kaj unuopaj valoroj.
struct Bufro {
    bajtoj: [u8; 160],
    longo: usize,
}

impl Bufro {
    const fn nova() -> Self {
        Self {
            bajtoj: [0; 160],
            longo: 0,
        }
    }

    fn malplenigu(&mut self) {
        self.longo = 0;
    }

    fn teksto(&self) -> &str {
        core::str::from_utf8(&self.bajtoj[..self.longo]).unwrap_or("")
    }
}

impl core::fmt::Write for Bufro {
    fn write_str(&mut self, teksto: &str) -> core::fmt::Result {
        let fino = self.longo + teksto.len();
        if fino > self.bajtoj.len() {
            return Err(core::fmt::Error);
        }
        self.bajtoj[self.longo..fino].copy_from_slice(teksto.as_bytes());
        self.longo = fino;
        Ok(())
    }
}

/// La teksto antaŭ la unua nula bajto.
fn el_nula(bajtoj: &[u8]) -> &str {
    let longo = bajtoj
        .iter()
        .position(|bajto| *bajto == 0)
        .unwrap_or(bajtoj.len());
    core::str::from_utf8(&bajtoj[..longo]).unwrap_or("")
}

/// Kopiu `fonto` en `celo` sen tranĉi UTF-8-signon; redonas la kopiitan longon.
fn kopii_en(celo: &mut [u8], fonto: &str) -> usize {
    let mut longo = fonto.len().min(celo.len());
    while longo > 0 && !fonto.is_char_boundary(longo) {
        longo -= 1;
    }
    celo[..longo].copy_from_slice(&fonto.as_bytes()[..longo]);
    longo
}

/// La speco de rekordo decidas, kiujn kampojn la tabelo montras.
#[derive(Clone, Copy, PartialEq)]
enum Speco {
    Dosiero,
    Procezo,
    Mesago,
    Kampo,
}

impl Speco {
    fn kampoj(self) -> &'static [&'static str] {
        match self {
            Speco::Dosiero => &["nomo", "tipo", "grandeco", "tenilo"],
            Speco::Procezo => &["pid", "nomo", "stato", "memoro", "permesoj"],
            Speco::Mesago => &["mesaĝo"],
            Speco::Kampo => &["kampo", "valoro"],
        }
    }
}

/// Unu objekto en la dukto.
#[derive(Clone, Copy)]
struct Rekordo {
    speco: Speco,
    nomo: [u8; NOMA_LONGO],
    tipo: u8,
    grandeco: u32,
    inodo: u32,
    pid: u64,
    stato: u8,
    memoro: u64,
    permesoj: [u8; PERMESOJ_LONGO],
    teksto: [u8; 120],
    teksta_longo: usize,
}

impl Rekordo {
    const fn malplena(speco: Speco) -> Self {
        Self {
            speco,
            nomo: [0; NOMA_LONGO],
            tipo: 0,
            grandeco: 0,
            inodo: 0,
            pid: 0,
            stato: 0,
            memoro: 0,
            permesoj: [0; PERMESOJ_LONGO],
            teksto: [0; 120],
            teksta_longo: 0,
        }
    }

    fn el_dosierero(ero: &Dosierero) -> Self {
        let mut rekordo = Self::malplena(Speco::Dosiero);
        rekordo.nomo = ero.nomo;
        rekordo.tipo = ero.tipo;
        rekordo.grandeco = ero.grandeco;
        rekordo.inodo = ero.inodo;
        rekordo
    }

    fn el_procezrekordo(ero: &ProcezoRekordo) -> Self {
        let mut rekordo = Self::malplena(Speco::Procezo);
        rekordo.nomo = ero.nomo;
        rekordo.pid = ero.pid;
        rekordo.stato = ero.stato;
        rekordo.memoro = ero.memoro;
        rekordo.permesoj = ero.permesoj;
        rekordo
    }

    fn mesago(teksto: &str) -> Self {
        let mut rekordo = Self::malplena(Speco::Mesago);
        rekordo.teksta_longo = kopii_en(&mut rekordo.teksto, teksto);
        rekordo
    }

    fn kampo(kampo: &str, valoro: &str) -> Self {
        let mut rekordo = Self::malplena(Speco::Kampo);
        kopii_en(&mut rekordo.nomo, kampo);
        rekordo.teksta_longo = kopii_en(&mut rekordo.teksto, valoro);
        rekordo
    }

    /// Tekstaj kampoj; la nombraj kaj `tenilo` venas el `valoro`.
    fn teksta(&self, kampo: &str) -> Option<&str> {
        match (self.speco, kampo) {
            (Speco::Dosiero | Speco::Procezo, "nomo") => Some(el_nula(&self.nomo)),
            (Speco::Kampo, "kampo") => Some(el_nula(&self.nomo)),
            (Speco::Kampo, "valoro") => {
                Some(core::str::from_utf8(&self.teksto[..self.teksta_longo]).unwrap_or(""))
            }
            (Speco::Mesago, "mesaĝo") => {
                Some(core::str::from_utf8(&self.teksto[..self.teksta_longo]).unwrap_or(""))
            }
            (Speco::Dosiero, "tipo") => Some(if self.tipo == TIPO_DOSIERUJO {
                "dosierujo"
            } else {
                "dosiero"
            }),
            (Speco::Procezo, "stato") => Some(match self.stato {
                STATO_PRETA => "Preta",
                STATO_KURANTA => "Kuranta",
                _ => "Mortinta",
            }),
            (Speco::Procezo, "permesoj") => Some(if self.permesoj[0] == 0 {
                "—"
            } else {
                el_nula(&self.permesoj)
            }),
            _ => None,
        }
    }

    /// La nombra valoro en fundamentaj unuoj (bajtoj aŭ identigilo).
    fn nombro(&self, kampo: &str) -> Option<u64> {
        match (self.speco, kampo) {
            (Speco::Dosiero, "grandeco") => Some(self.grandeco as u64),
            (Speco::Procezo, "pid") => Some(self.pid),
            (Speco::Procezo, "memoro") => Some(self.memoro),
            _ => None,
        }
    }

    /// La montra valoro de `kampo`; nombroj uziĝas la unuojn de FluoU.
    fn valoro<'a>(&'a self, kampo: &str, bufro: &'a mut Bufro) -> Option<&'a str> {
        match kampo {
            "tenilo" if self.speco == Speco::Dosiero => {
                bufro.malplenigu();
                let _ = write!(bufro, "0x{:04x}", self.inodo);
                Some(bufro.teksto())
            }
            "grandeco" if self.speco == Speco::Dosiero => {
                if self.grandeco < 1024 {
                    bufro.malplenigu();
                    let _ = write!(bufro, "{} B", self.grandeco);
                } else {
                    kvanto(bufro, self.grandeco as u64, 1024, " KiB");
                }
                Some(bufro.teksto())
            }
            "memoro" if self.speco == Speco::Procezo => {
                kvanto(bufro, self.memoro, 1024 * 1024, " MiB");
                Some(bufro.teksto())
            }
            "pid" if self.speco == Speco::Procezo => {
                bufro.malplenigu();
                let _ = write!(bufro, "{}", self.pid);
                Some(bufro.teksto())
            }
            _ => self.teksta(kampo),
        }
    }
}

/// Skribu `valoro / dividanto` kun unu dekuma cifero kaj la unuo.
fn kvanto(bufro: &mut Bufro, valoro: u64, dividanto: u64, unuo: &str) {
    bufro.malplenigu();
    let entjero = valoro / dividanto;
    let dekimo = (valoro % dividanto) * 10 / dividanto;
    let _ = write!(bufro, "{entjero}.{dekimo}{unuo}");
}

/// Aro da samspecaj rekordoj, kiun la dukto transdonas inter stadioj.
#[derive(Clone, Copy)]
struct Vico {
    speco: Speco,
    eroj: [Rekordo; MAKS_REKORDOJ],
    nombro: usize,
}

impl Vico {
    fn nova(speco: Speco) -> Self {
        Self {
            speco,
            eroj: [Rekordo::malplena(speco); MAKS_REKORDOJ],
            nombro: 0,
        }
    }

    fn puŝi(&mut self, rekordo: Rekordo) {
        if self.nombro < self.eroj.len() {
            self.eroj[self.nombro] = rekordo;
            self.nombro += 1;
        }
    }
}

/// La majuskla kapo de tabela kolumno, kiel en FluoU.
fn majuskla(kampo: &str) -> &'static str {
    match kampo {
        "nomo" => "NOMO",
        "tipo" => "TIPO",
        "grandeco" => "GRANDECO",
        "tenilo" => "TENILO",
        "pid" => "PID",
        "stato" => "STATO",
        "memoro" => "MEMORO",
        "permesoj" => "PERMESOJ",
        "mesaĝo" => "MESAĜO",
        "kampo" => "KAMPO",
        "valoro" => "VALORO",
        _ => "KAMPO",
    }
}

fn koloro_de(kampo: &str, valoro: &str) -> &'static str {
    match kampo {
        "nomo" | "kampo" => FG_NOMO,
        "tipo" => {
            if valoro == "dosierujo" {
                akcenta()
            } else {
                FG_INFORMO
            }
        }
        "stato" => match valoro {
            "Kuranta" => FG_SUKCESO,
            "Preta" => FG_INFORMO,
            _ => FG_TEKSTO3,
        },
        "grandeco" | "memoro" | "pid" => FG_TEKSTO,
        "permesoj" => FG_TEKSTO3,
        _ => FG_TEKSTO2,
    }
}

/// Ĉu la kampo estas dekstre aliniiĝinta nombro?
fn nombra_kampo(kampo: &str) -> bool {
    matches!(kampo, "grandeco" | "memoro" | "pid")
}

/// La larĝo de la `TENILO`-ŝildeto: spaco, ses literoj, spaco.
const ŜILDETO: usize = 8;

/// Montru la aron kiel koloran tabelon (O13).
fn tabelo(vico: &Vico, elektitaj: &[&'static str]) {
    if vico.nombro == 0 {
        biblioteko::skribi_formaton(format_args!("{FG_TEKSTO3}Neniu rekordo.{NULIGI}\n"));
        return;
    }
    let kolumnoj: &[&'static str] = if elektitaj.is_empty() {
        vico.speco.kampoj()
    } else {
        elektitaj
    };
    let mut larĝoj = [0usize; 8];
    let mut bufro = Bufro::nova();
    for (indekso, kampo) in kolumnoj.iter().enumerate() {
        larĝoj[indekso] = majuskla(kampo).chars().count();
    }
    for rekordo in &vico.eroj[..vico.nombro] {
        for (indekso, kampo) in kolumnoj.iter().enumerate() {
            let teksto = rekordo.valoro(kampo, &mut bufro).unwrap_or("—");
            let kroma = if *kampo == "tenilo" { ŜILDETO } else { 0 };
            larĝoj[indekso] = larĝoj[indekso].max(teksto.chars().count() + kroma);
        }
    }

    biblioteko::skribi_formaton(format_args!("{FG_TEKSTO3}{GRASA}"));
    for (indekso, kampo) in kolumnoj.iter().enumerate() {
        if indekso > 0 {
            biblioteko::skribi_formaton(format_args!("  "));
        }
        biblioteko::skribi_formaton(format_args!(
            "{:<larĝo$}",
            majuskla(kampo),
            larĝo = larĝoj[indekso]
        ));
    }
    biblioteko::skribi_formaton(format_args!("{NULIGI}\n"));
    biblioteko::skribi_formaton(format_args!(
        "{FG_TEKSTO3}────────────────────────────────────────────────────────────{NULIGI}\n"
    ));

    for rekordo in &vico.eroj[..vico.nombro] {
        for (indekso, kampo) in kolumnoj.iter().enumerate() {
            if indekso > 0 {
                biblioteko::skribi_formaton(format_args!("  "));
            }
            let teksto = rekordo.valoro(kampo, &mut bufro).unwrap_or("—");
            if *kampo == "tenilo" {
                let uzata = teksto.chars().count() + ŜILDETO;
                let plenigo = larĝoj[indekso].saturating_sub(uzata);
                biblioteko::skribi_formaton(format_args!(
                    "{BG_TENILO}{FG_ORO} TENILO {teksto}{NULIGI}{:<plenigo$}",
                    "",
                    plenigo = plenigo
                ));
            } else if nombra_kampo(kampo) {
                biblioteko::skribi_formaton(format_args!(
                    "{}{:>larĝo$}{NULIGI}",
                    koloro_de(kampo, teksto),
                    teksto,
                    larĝo = larĝoj[indekso]
                ));
            } else {
                biblioteko::skribi_formaton(format_args!(
                    "{}{:<larĝo$}{NULIGI}",
                    koloro_de(kampo, teksto),
                    teksto,
                    larĝo = larĝoj[indekso]
                ));
            }
        }
        biblioteko::skribi("\n");
    }
}

fn eraro(komando: &str, kaŭzo: &str, kodo: i32) {
    biblioteko::skribi_formaton(format_args!(
        "{FG_DANĜERO}✕ {komando}: {kaŭzo} ({kodo}){NULIGI}\n"
    ));
}

fn eraro_senprefikse(teksto: &str) {
    biblioteko::skribi_formaton(format_args!("{FG_DANĜERO}{teksto}{NULIGI}\n"));
}

/// Eraro, kies kaŭzo enhavas unu valoron (vojon, kampon, operacion).
fn eraro_kun(komando: &str, prefikso: &str, valoro: &str, kodo: i32) {
    let mut mesaĝo = Bufro::nova();
    let _ = write!(mesaĝo, "{prefikso}{valoro}");
    eraro(komando, mesaĝo.teksto(), kodo);
}

fn sukceso(teksto: &str) {
    biblioteko::skribi_formaton(format_args!("{FG_SUKCESO}{teksto}{NULIGI}\n"));
}

fn averti(teksto: &str) {
    biblioteko::skribi_formaton(format_args!("{FG_TEKSTO3}{teksto}{NULIGI}\n"));
}

/// Forigu ĉirkaŭajn citilojn de argumento.
fn sen_citiloj(argumento: &str) -> &str {
    let argumento = argumento.trim();
    if argumento.len() >= 2 {
        let unua = argumento.as_bytes()[0];
        let lasta = argumento.as_bytes()[argumento.len() - 1];
        if (unua == b'"' && lasta == b'"') || (unua == b'\'' && lasta == b'\'') {
            return &argumento[1..argumento.len() - 1];
        }
    }
    argumento
}

/// Ĉu la teksto estas la komando aŭ stadio `nomo`? La komandnomoj estas
/// Esperantaj; nur la askiaj literoj ignoras usklecon.
fn estas(teksto: &str, nomo: &str) -> bool {
    teksto.eq_ignore_ascii_case(nomo)
}

/// La komandoj kaj stadioj de Ligo, por la helpa tabelo.
const KOMANDOJ: [(&str, &str); 19] = [
    ("helpu", "Montri la komandtabelon kaj la helpon"),
    (
        "listi",
        "Listigi dosierojn kun nomo, tipo, grandeco kaj tenilo",
    ),
    ("legi", "Legi dosieron kiel rekordon kun tenilo"),
    ("procezoj", "Montri la procezan tabelon"),
    ("mortigi", "Mortigi procezon (ne protektan)"),
    ("lanĉi", "Lanĉi bildon el SNDS kiel novan procezon"),
    ("eldiri", "Krei mesaĝan rekordon"),
    ("pri", "Montri sistemajn informojn"),
    ("temo", "Ŝanĝi la temon: malluma aŭ hela"),
    ("akcento", "Ŝanĝi la akcenton de la interfaco"),
    ("eniro", "Malfermi la komandan paletron"),
    ("filtri", "Filtri rekordojn laŭ kampo"),
    ("ordigi", "Ordigi rekordojn laŭ kampo"),
    ("unua", "Preni la unuajn rekordojn"),
    ("sumi", "Sumi nombran kampon"),
    ("nombri", "Nombri rekordojn"),
    ("montri", "Projekcii kolumnojn"),
    ("teksto", "Transiri al la eksplicita teksta limo"),
    ("purigi", "Malplenigi la terminalon"),
];

fn estas_stadio(teksto: &str) -> bool {
    [
        "filtri", "ordigi", "unua", "sumi", "nombri", "montri", "teksto",
    ]
    .iter()
    .any(|nomo| estas(teksto, nomo))
}

fn helpo() {
    biblioteko::skribi_formaton(format_args!(
        "{FG_INFORMO}{GRASA}Komandaro de Ligo{NULIGI}\n"
    ));
    for (nomo, priskribo) in KOMANDOJ {
        biblioteko::skribi_formaton(format_args!(
            "  {FG_ORO}{nomo:<10}{NULIGI} {FG_TEKSTO2}{priskribo}{NULIGI}\n"
        ));
    }
    biblioteko::skribi_formaton(format_args!(
        "  {FG_ORO}{:<10}{NULIGI} {FG_TEKSTO2}Fini la ŝelon (ankaŭ Ctrl+D){NULIGI}\n",
        "fini"
    ));
    biblioteko::skribi_formaton(format_args!(
        "{FG_TEKSTO3}Duktoj: {NULIGI}{FG_TEKSTO2}listi | filtri grandeco > 100 | ordigi grandeco malkreske | montri{NULIGI}\n"
    ));
}

fn pri() {
    let mut vico = Vico::nova(Speco::Kampo);
    vico.puŝi(Rekordo::kampo("FluoU", FLUOU_VERSIO));
    vico.puŝi(Rekordo::kampo("ABI", "2"));
    let mut bufro = Bufro::nova();
    let _ = write!(bufro, "MiKerno {}", env!("CARGO_PKG_VERSION"));
    vico.puŝi(Rekordo::kampo("Kerno", bufro.teksto()));
    vico.puŝi(Rekordo::kampo("Arkitekturoj", "x86_64, arm64"));
    vico.puŝi(Rekordo::kampo(
        "Kompozitoro",
        "izolita servo · nur FENESTRO-teniloj",
    ));
    vico.puŝi(Rekordo::kampo("Komandoj", "helpu por la plena tabelo"));
    tabelo(&vico, &[]);
}

fn listi(argumento: &str) -> Option<Vico> {
    let mut vojo = sen_citiloj(argumento);
    if vojo.is_empty() {
        vojo = "/";
    }
    let mut eroj = [Dosierero::nova(); MAKS_REKORDOJ];
    let rezulto = biblioteko::listi(vojo, &mut eroj);
    if rezulto < 0 {
        if rezulto == libmikerno::syscall::NE_TROVITA {
            eraro_kun("listi", "la dosierujo ne ekzistas: ", vojo, -3);
        } else {
            eraro_kun("listi", "nevalida vojo: ", vojo, -2);
        }
        return None;
    }
    let mut vico = Vico::nova(Speco::Dosiero);
    for ero in &eroj[..rezulto as usize] {
        vico.puŝi(Rekordo::el_dosierero(ero));
    }
    Some(vico)
}

fn trovi_tenilon(vojo: &str) -> Option<u32> {
    let (dosierujo, nomo) = match vojo.rfind('/') {
        Some(indekso) => (&vojo[..indekso], &vojo[indekso + 1..]),
        None => ("", vojo),
    };
    let dosierujo = if dosierujo.is_empty() { "/" } else { dosierujo };
    let mut eroj = [Dosierero::nova(); MAKS_REKORDOJ];
    let rezulto = biblioteko::listi(dosierujo, &mut eroj);
    if rezulto < 0 {
        return None;
    }
    eroj[..rezulto as usize]
        .iter()
        .find(|ero| el_nula(&ero.nomo) == nomo)
        .map(|ero| ero.inodo)
}

fn legi(argumento: &str) {
    let vojo = sen_citiloj(argumento);
    if vojo.is_empty() {
        eraro("legi", "mankas dosiernomo", -2);
        return;
    }
    let mut bufro = [0u8; MAKS_LEGO];
    let rezulto = biblioteko::legi(vojo, &mut bufro);
    if rezulto < 0 {
        if rezulto == libmikerno::syscall::NE_TROVITA {
            eraro_kun("legi", "dosiero ne ekzistas: ", vojo, -3);
        } else if rezulto == libmikerno::syscall::TABELO_PLENA {
            eraro("legi", "la dosiero tro grandas aŭ ne estas teksto", -90);
        } else {
            eraro_kun("legi", "nevalida vojo: ", vojo, -2);
        }
        return;
    }
    let Ok(teksto) = core::str::from_utf8(&bufro[..rezulto as usize]) else {
        eraro_kun("legi", "ne estas teksto: ", vojo, -90);
        return;
    };
    karto(vojo, trovi_tenilon(vojo), teksto);
}

fn karto(vojo: &str, tenilo: Option<u32>, teksto: &str) {
    biblioteko::skribi_formaton(format_args!("{BG_KAPO}{FG_TEKSTO2} {vojo} {NULIGI}\n"));
    for linio in teksto.split('\n') {
        biblioteko::skribi(akcenta());
        biblioteko::skribi("│");
        biblioteko::skribi(NULIGI);
        if !linio.is_empty() {
            biblioteko::skribi(" ");
            biblioteko::skribi(FG_TEKSTO2);
            biblioteko::skribi(linio);
            biblioteko::skribi(NULIGI);
        }
        biblioteko::skribi("\n");
    }
    if let Some(inodo) = tenilo {
        biblioteko::skribi_formaton(format_args!(
            "{BG_TENILO}{FG_ORO} TENILO 0x{inodo:04x} {NULIGI}\n"
        ));
    }
}

fn procezoj() -> Vico {
    let mut eroj = [ProcezoRekordo::nova(); MAKS_REKORDOJ];
    let nombro = biblioteko::procezoj(&mut eroj);
    let mut vico = Vico::nova(Speco::Procezo);
    if nombro < 0 {
        return vico;
    }
    for ero in &eroj[..nombro as usize] {
        vico.puŝi(Rekordo::el_procezrekordo(ero));
    }
    vico
}

fn mortigi(argumento: &str) {
    let argumento = argumento.trim();
    let Ok(pid) = argumento.parse::<u64>() else {
        eraro("mortigi", "nevalida proceza identigilo", -3);
        return;
    };
    if pid <= 1 {
        let mut mesaĝo = Bufro::nova();
        let _ = write!(mesaĝo, "procezo {pid} estas protekta sistema procezo (-1)");
        eraro_senprefikse(mesaĝo.teksto());
        return;
    }
    let mut eroj = [ProcezoRekordo::nova(); MAKS_REKORDOJ];
    let nombro = biblioteko::procezoj(&mut eroj);
    let nomo = if nombro > 0 {
        eroj[..nombro as usize]
            .iter()
            .find(|ero| ero.pid == pid)
            .map(|ero| el_nula(&ero.nomo))
            .unwrap_or("procezo")
    } else {
        "procezo"
    };
    let mut mesaĝo = Bufro::nova();
    let _ = write!(mesaĝo, "Procezo {pid} ({nomo}) finiĝis.");
    if biblioteko::mortigi(pid) == 0 {
        sukceso(mesaĝo.teksto());
    } else {
        eraro_kun("mortigi", "la procezo ne ekzistas: ", argumento, -3);
    }
}

fn lanĉi(argumento: &str) {
    let vojo = sen_citiloj(argumento);
    if vojo.is_empty() {
        eraro("lanĉi", "mankas dosiernomo", -2);
        return;
    }
    let rezulto = biblioteko::lanĉi(vojo);
    if rezulto > 0 {
        let mut mesaĝo = Bufro::nova();
        let _ = write!(mesaĝo, "Procezo {rezulto} lanĉita: {vojo}");
        sukceso(mesaĝo.teksto());
        return;
    }
    if rezulto == libmikerno::syscall::NE_TROVITA {
        eraro_kun("lanĉi", "dosiero ne ekzistas: ", vojo, -3);
    } else if rezulto == libmikerno::syscall::TABELO_PLENA {
        eraro("lanĉi", "la proceza tabelo estas plena", -28);
    } else {
        eraro_kun("lanĉi", "nevalida bildo: ", vojo, -90);
    }
}

fn eldiri(argumento: &str) {
    let teksto = argumento.trim();
    if teksto.is_empty() {
        eraro("eldiri", "mankas teksto", -2);
        return;
    }
    let mut vico = Vico::nova(Speco::Mesago);
    vico.puŝi(Rekordo::mesago(teksto));
    tabelo(&vico, &[]);
}

fn temo(argumento: &str) {
    let valoro = argumento.trim();
    if !estas(valoro, "malluma") && !estas(valoro, "hela") {
        eraro("temo", "uzu \"malluma\" aŭ \"hela\"", -3);
        return;
    }
    let mut mesaĝo = Bufro::nova();
    let _ = write!(mesaĝo, "Temo ŝanĝita al {valoro}.");
    sukceso(mesaĝo.teksto());
}

fn akcento(argumento: &str) {
    let kruda = argumento.trim();
    let Some(indekso) = AKCENTOJ.iter().position(|(nomo, _)| estas(kruda, nomo)) else {
        eraro(
            "akcento",
            "uzu oro, koralo, ĉielo, verdo, viola aŭ bluo",
            -3,
        );
        return;
    };
    AKCENTO.store(indekso as u8, AtomOrdering::Relaxed);
    let mut mesaĝo = Bufro::nova();
    let _ = write!(mesaĝo, "Akcento ŝanĝita al {}.", AKCENTOJ[indekso].0);
    biblioteko::skribi_formaton(format_args!(
        "{}{GRASA}{}{NULIGI}\n",
        akcenta(),
        mesaĝo.teksto()
    ));
}

fn eniro() {
    sukceso("Eniro malfermita. Premu Ctrl+E (provizore) ie ajn; estonte Mito.");
}

fn purigi() {
    biblioteko::skribi("\x1b[2J\x1b[H");
    bonvenon();
}

/// La dekuma valoro en milonoj (`1,5` → `1500`), por kompari kun la
/// fundamentaj unuoj de la kampo.
fn nombro_el(teksto: &str) -> Option<u64> {
    let teksto = teksto.trim();
    if teksto.is_empty() || teksto.starts_with('-') {
        return None;
    }
    let (entjero, dekimo) = match teksto.find(['.', ',']) {
        Some(indekso) => (&teksto[..indekso], &teksto[indekso + 1..]),
        None => (teksto, ""),
    };
    let entjero: u64 = entjero.parse().ok()?;
    if dekimo.len() > 3 || !dekimo.bytes().all(|bajto| bajto.is_ascii_digit()) {
        return None;
    }
    let mut milonoj = 0u64;
    for (indekso, bajto) in dekimo.bytes().enumerate() {
        let cifero = (bajto - b'0') as u64;
        milonoj += cifero * [100, 10, 1][indekso];
    }
    Some(entjero * 1000 + milonoj)
}

fn unuo_de(kampo: &str) -> u64 {
    match kampo {
        "grandeco" => 1024,
        "memoro" => 1024 * 1024,
        _ => 1,
    }
}

fn egalas_ignorante(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn entenas_ignorante(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    b.is_empty()
        || a.windows(b.len()).any(|fenestro| {
            fenestro
                .iter()
                .zip(b)
                .all(|(x, y)| x.eq_ignore_ascii_case(y))
        })
}

fn filtri(fonto: &Vico, argumentoj: &[&str]) -> Option<Vico> {
    if argumentoj.len() < 3 {
        eraro("filtri", "mankas kampo, operacio aŭ valoro", -2);
        return None;
    }
    let kampo = argumentoj[0];
    let operacio = argumentoj[1];
    let mut valora_bufro = [0u8; 64];
    let celo = sen_citiloj(kunigu(&argumentoj[2..], &mut valora_bufro));
    if !fonto.speco.kampoj().contains(&kampo) {
        eraro_kun("filtri", "kampo ne ekzistas: ", kampo, -2);
        return None;
    }
    let sojlo = if nombra_kampo(kampo) {
        let Some(valoro) = nombro_el(celo) else {
            eraro_kun("filtri", "nevalida nombro: ", celo, -3);
            return None;
        };
        Some(valoro)
    } else {
        None
    };
    let operacio_valida = if sojlo.is_some() {
        matches!(operacio, ">" | "<" | ">=" | "<=" | "==" | "=" | "!=")
    } else {
        matches!(operacio, "==" | "=" | "!=" | "entenas")
    };
    if !operacio_valida {
        eraro_kun("filtri", "nekonata operacio: ", operacio, -2);
        return None;
    }
    let mut rezulto = Vico::nova(fonto.speco);
    for ero in &fonto.eroj[..fonto.nombro] {
        let konservu = if let (Some(valoro), Some(sojlo)) = (ero.nombro(kampo), sojlo) {
            let skalo = valoro.saturating_mul(1000) / unuo_de(kampo);
            match operacio {
                ">" => skalo > sojlo,
                "<" => skalo < sojlo,
                ">=" => skalo >= sojlo,
                "<=" => skalo <= sojlo,
                "==" | "=" => skalo == sojlo,
                _ => skalo != sojlo,
            }
        } else {
            let valoro = ero.teksta(kampo).unwrap_or("");
            match operacio {
                "==" | "=" => egalas_ignorante(valoro, celo),
                "!=" => !egalas_ignorante(valoro, celo),
                _ => entenas_ignorante(valoro, celo),
            }
        };
        if konservu {
            rezulto.puŝi(*ero);
        }
    }
    Some(rezulto)
}

/// Kunigu la ceterajn argumentojn per unu spaco; la celo estas bufraĵo, por ke
/// la valoro restu valida dum la filtrado.
fn kunigu<'a>(tokoj: &[&str], bufro: &'a mut [u8; 64]) -> &'a str {
    let mut longo = 0;
    for (indekso, toko) in tokoj.iter().enumerate() {
        if indekso > 0 && longo < bufro.len() {
            bufro[longo] = b' ';
            longo += 1;
        }
        longo += kopii_en(&mut bufro[longo..], toko);
    }
    core::str::from_utf8(&bufro[..longo]).unwrap_or("")
}

fn kompari(a: &Rekordo, b: &Rekordo, kampo: &str) -> Ordering {
    if let (Some(x), Some(y)) = (a.nombro(kampo), b.nombro(kampo)) {
        return x.cmp(&y);
    }
    a.teksta(kampo)
        .unwrap_or("")
        .cmp(b.teksta(kampo).unwrap_or(""))
}

fn ordigi(vico: &mut Vico, argumentoj: &[&str]) {
    let Some(kampo) = argumentoj.first() else {
        eraro("ordigi", "mankas kampo", -2);
        vico.nombro = 0;
        return;
    };
    if !vico.speco.kampoj().contains(kampo) {
        eraro_kun("ordigi", "kampo ne ekzistas: ", kampo, -2);
        vico.nombro = 0;
        return;
    }
    let malkreske = argumentoj.get(1).is_some_and(|a| estas(a, "malkreske"));
    for indekso in 1..vico.nombro {
        let nuna = vico.eroj[indekso];
        let mut pozicio = indekso;
        while pozicio > 0 {
            let antaŭa = vico.eroj[pozicio - 1];
            let ordo = kompari(&nuna, &antaŭa, kampo);
            let ordo = if malkreske { ordo.reverse() } else { ordo };
            if ordo != Ordering::Less {
                break;
            }
            vico.eroj[pozicio] = antaŭa;
            pozicio -= 1;
        }
        vico.eroj[pozicio] = nuna;
    }
}

fn unua(vico: &mut Vico, argumentoj: &[&str]) {
    let Some(toko) = argumentoj.first() else {
        eraro("unua", "mankas valida nombro", -2);
        vico.nombro = 0;
        return;
    };
    let Ok(n) = toko.parse::<usize>() else {
        eraro("unua", "mankas valida nombro", -2);
        vico.nombro = 0;
        return;
    };
    vico.nombro = vico.nombro.min(n);
}

fn sumi(vico: &Vico, argumentoj: &[&str]) {
    let Some(kampo) = argumentoj.first() else {
        eraro("sumi", "mankas kampo", -2);
        return;
    };
    if !vico.speco.kampoj().contains(kampo) {
        eraro_kun("sumi", "kampo ne ekzistas: ", kampo, -2);
        return;
    }
    let mut sumo = 0u64;
    for ero in &vico.eroj[..vico.nombro] {
        let Some(valoro) = ero.nombro(kampo) else {
            eraro_kun("sumi", "ne estas nombra kampo: ", kampo, -2);
            return;
        };
        sumo = sumo.saturating_add(valoro);
    }
    let (dividanto, unuo) = match *kampo {
        "grandeco" => (1024, " KiB"),
        "memoro" => (1024 * 1024, " MiB"),
        _ => (1, ""),
    };
    let mut etikedo = Bufro::nova();
    let _ = write!(etikedo, "SUMO DE {kampo}");
    let mut valoro = Bufro::nova();
    kvanto(&mut valoro, sumo, dividanto, unuo);
    skalaro(etikedo.teksto(), valoro.teksto());
}

fn nombri(vico: &Vico) {
    let mut valoro = Bufro::nova();
    let _ = write!(valoro, "{}", vico.nombro);
    skalaro("NOMBRO DE REKORDOJ", valoro.teksto());
}

/// Skalara rezulto (O12): unu klara valoro kun etikedo, ne ŝajniga tabelo.
fn skalaro(etikedo: &str, valoro: &str) {
    let mut etikedo_majuskla = Bufro::nova();
    for signo in etikedo.chars() {
        let _ = write!(etikedo_majuskla, "{}", signo.to_ascii_uppercase());
    }
    biblioteko::skribi_formaton(format_args!(
        "{FG_TEKSTO3}{GRASA}{}{NULIGI}  {}{GRASA}{valoro}{NULIGI}\n",
        etikedo_majuskla.teksto(),
        akcenta()
    ));
}

fn montri(vico: &Vico, argumentoj: &[&str], elektitaj: &mut [&'static str; 8]) -> usize {
    let mut uzataj = 0;
    for kampo in argumentoj {
        let Some(nomo) = vico
            .speco
            .kampoj()
            .iter()
            .copied()
            .find(|nomo| estas(kampo, nomo))
        else {
            eraro_kun("montri", "kampo ne ekzistas: ", kampo, -2);
            return 0;
        };
        if uzataj < elektitaj.len() {
            elektitaj[uzataj] = nomo;
            uzataj += 1;
        }
    }
    uzataj
}

fn teksta_limo(vico: &Vico) {
    biblioteko::skribi_formaton(format_args!(
        "{FG_TEKSTO3}{GRASA}Eksplicita teksta limo{NULIGI}\n"
    ));
    let mut bufro = Bufro::nova();
    for ero in &vico.eroj[..vico.nombro] {
        biblioteko::skribi(FG_TEKSTO2);
        for (indekso, kampo) in vico.speco.kampoj().iter().enumerate() {
            if indekso > 0 {
                biblioteko::skribi(" · ");
                biblioteko::skribi(kampo);
                biblioteko::skribi(": ");
            }
            biblioteko::skribi(ero.valoro(kampo, &mut bufro).unwrap_or("—"));
        }
        biblioteko::skribi(NULIGI);
        biblioteko::skribi("\n");
    }
}

fn nekonata(vorto: &str) {
    biblioteko::skribi_formaton(format_args!(
        "{FG_DANĜERO}Nekonata komando: \"{vorto}\" (-2) — tajpu helpu{NULIGI}\n"
    ));
}

/// Analizu kaj plenumu unu linion; la dukto portas rekordojn, ne tekston.
fn plenumi(linio: &str) {
    let mut partoj = linio.split('|');
    let unua_parto = partoj.next().unwrap_or("").trim();
    if unua_parto.is_empty() {
        return;
    }
    let (vorto, argumento) = disigu(unua_parto);
    if estas(vorto, "fini") {
        let kodo = argumento.trim().parse::<usize>().unwrap_or(0);
        biblioteko::fini(kodo);
    }

    let vico = if estas(vorto, "helpu") {
        helpo();
        return;
    } else if estas(vorto, "legi") {
        legi(argumento);
        return;
    } else if estas(vorto, "mortigi") {
        mortigi(argumento);
        return;
    } else if estas(vorto, "lanĉi") {
        lanĉi(argumento);
        return;
    } else if estas(vorto, "eldiri") {
        eldiri(argumento);
        return;
    } else if estas(vorto, "pri") {
        pri();
        return;
    } else if estas(vorto, "temo") {
        temo(argumento);
        return;
    } else if estas(vorto, "akcento") {
        akcento(argumento);
        return;
    } else if estas(vorto, "eniro") {
        eniro();
        return;
    } else if estas(vorto, "purigi") {
        purigi();
        return;
    } else if estas(vorto, "listi") {
        listi(argumento)
    } else if estas(vorto, "procezoj") {
        let tabelo_vico = procezoj();
        if tabelo_vico.nombro == 0 {
            averti("La proceza tabelo estas malplena.");
        }
        Some(tabelo_vico)
    } else if estas_stadio(vorto) {
        eraro("Ligo", "neniuj rekordoj; komencu per listi aŭ procezoj", -2);
        return;
    } else {
        nekonata(vorto);
        return;
    };
    let mut vico = vico;

    let mut elektitaj = [""; 8];
    let mut elektitaj_nombro = 0usize;
    let mut tokaj = [""; 8];
    for parto in partoj {
        let Some(mut nuna) = vico.take() else {
            return;
        };
        let (stadio, argumenta_teksto) = disigu(parto.trim());
        if stadio.is_empty() {
            vico = Some(nuna);
            continue;
        }
        let mut nombro = 0usize;
        for toko in argumenta_teksto.split_whitespace() {
            if nombro < tokaj.len() {
                tokaj[nombro] = toko;
                nombro += 1;
            }
        }
        let argumentoj = &tokaj[..nombro];
        if estas(stadio, "filtri") {
            vico = filtri(&nuna, argumentoj);
        } else if estas(stadio, "ordigi") {
            ordigi(&mut nuna, argumentoj);
            vico = Some(nuna);
        } else if estas(stadio, "unua") {
            unua(&mut nuna, argumentoj);
            vico = Some(nuna);
        } else if estas(stadio, "sumi") {
            sumi(&nuna, argumentoj);
            vico = None;
        } else if estas(stadio, "nombri") {
            nombri(&nuna);
            vico = None;
        } else if estas(stadio, "montri") {
            elektitaj_nombro = montri(&nuna, argumentoj, &mut elektitaj);
            vico = Some(nuna);
        } else if estas(stadio, "teksto") {
            teksta_limo(&nuna);
            vico = None;
        } else {
            let mut mesaĝo = Bufro::nova();
            let _ = write!(mesaĝo, "Nekonata stadio: \"{stadio}\" (-2)");
            eraro_senprefikse(mesaĝo.teksto());
            vico = None;
        }
        if vico.is_none() {
            return;
        }
    }
    if let Some(fina) = vico {
        tabelo(&fina, &elektitaj[..elektitaj_nombro]);
    }
}

/// Disigu la unuan vorton de la cetero.
fn disigu(teksto: &str) -> (&str, &str) {
    match teksto.find(char::is_whitespace) {
        Some(indekso) => (&teksto[..indekso], teksto[indekso..].trim()),
        None => (teksto, ""),
    }
}

fn bonvenon() {
    biblioteko::skribi_formaton(format_args!(
        "{FG_INFORMO}{GRASA}Ligo {FLUOU_VERSIO}{NULIGI} — objekta ŝelo de Horizonto · ABI 2\n\
         {FG_TEKSTO3}Tajpu helpu por la komandaro. Anglaj nomoj kiel ls ne ekzistas.{NULIGI}\n"
    ));
}

fn skribi_prompton() {
    biblioteko::skribi_formaton(format_args!("{}{GRASA}ligo:~>{NULIGI} ", akcenta()));
}

/// Redesegnu la aktivan linion post ĉiu eniga bajto (O09).
fn desegni(linio: &[u8]) {
    biblioteko::skribi("\r\x1b[K");
    skribi_prompton();
    if let Ok(teksto) = core::str::from_utf8(linio) {
        biblioteko::skribi(teksto);
    }
}

/// La historio de la ŝelo: la lastaj `HISTORIA_LONGO` linioj.
struct Historio {
    eroj: [[u8; 64]; HISTORIA_LONGO],
    longoj: [u8; HISTORIA_LONGO],
    sekva: usize,
    nombro: usize,
}

impl Historio {
    const fn nova() -> Self {
        Self {
            eroj: [[0; 64]; HISTORIA_LONGO],
            longoj: [0; HISTORIA_LONGO],
            sekva: 0,
            nombro: 0,
        }
    }

    fn aldoni(&mut self, linio: &str) {
        let linio = linio.trim();
        if linio.is_empty() {
            return;
        }
        if self.nombro > 0 {
            let lasta = (self.sekva + HISTORIA_LONGO - 1) % HISTORIA_LONGO;
            let lasta_longo = self.longoj[lasta] as usize;
            if core::str::from_utf8(&self.eroj[lasta][..lasta_longo]).ok() == Some(linio) {
                return;
            }
        }
        let longo = kopii_en(&mut self.eroj[self.sekva], linio);
        self.longoj[self.sekva] = longo as u8;
        self.sekva = (self.sekva + 1) % HISTORIA_LONGO;
        if self.nombro < HISTORIA_LONGO {
            self.nombro += 1;
        }
    }

    /// La rikordo `de` poziciojn malantaŭ la plej nova; redonas la novan
    /// vizitindekson kaj la tekston.
    fn antaŭa(&self, de: usize) -> Option<(usize, &[u8])> {
        if de >= self.nombro {
            return None;
        }
        let indekso = (self.sekva + HISTORIA_LONGO - 1 - de) % HISTORIA_LONGO;
        Some((de + 1, &self.eroj[indekso][..self.longoj[indekso] as usize]))
    }
}

/// La historio kaj la akcento estas statikaj; Ligo havas unu fadenon, do la
/// aliroj estas sinsekvaj kaj ne bezonas ŝlosilon.
struct Kunhavigita<T>(core::cell::UnsafeCell<T>);

// SAFETY: unuprocesora ŝelo kun unu fadeno; neniu samtempa aliro okazas.
unsafe impl<T> Sync for Kunhavigita<T> {}

static HISTORIO: Kunhavigita<Historio> = Kunhavigita(core::cell::UnsafeCell::new(Historio::nova()));

fn historio() -> &'static mut Historio {
    // SAFETY: la ŝelo estas la sola aliranto kaj la referencoj ne interkovras.
    unsafe { &mut *HISTORIO.0.get() }
}

/// Ĉu `nomo` komenciĝas per `vorto` (askia uskleco ignorata)?
fn antaŭas(nomo: &str, vorto: &str) -> bool {
    nomo.len() >= vorto.len()
        && nomo
            .as_bytes()
            .iter()
            .zip(vorto.as_bytes())
            .all(|(noma, vorta)| noma.eq_ignore_ascii_case(vorta))
}

fn komuna_prefikso<'a>(a: &'a str, b: &str) -> &'a str {
    let mut fino = 0;
    for (x, y) in a.chars().zip(b.chars()) {
        if x != y {
            break;
        }
        fino += x.len_utf8();
    }
    &a[..fino]
}

/// Kompletigu la unuan vorton per `Tab` (O09). Redonas `true`, se la listo de
/// kandidatoj estis preskribita sur nova linio.
fn kompletigi(linio: &mut [u8], longo: &mut usize) -> bool {
    let (gvidaj, vorta_longo, nombro, sola, komuna) = {
        let Ok(teksto) = core::str::from_utf8(&linio[..*longo]) else {
            return false;
        };
        let gvidaj = teksto.len() - teksto.trim_start().len();
        let vorto = teksto.trim_start();
        if vorto.is_empty() || vorto.contains(char::is_whitespace) {
            return false;
        }
        let mut sola: Option<&'static str> = None;
        let mut komuna: Option<&'static str> = None;
        let mut nombro = 0usize;
        for (nomo, _) in KOMANDOJ {
            if antaŭas(nomo, vorto) {
                nombro += 1;
                sola = Some(nomo);
                komuna = Some(match komuna {
                    None => nomo,
                    Some(antaua) => komuna_prefikso(antaua, nomo),
                });
            }
        }
        (gvidaj, vorto.len(), nombro, sola, komuna)
    };
    if nombro == 0 {
        return false;
    }
    if nombro == 1 {
        let nomo = sola.unwrap_or("");
        let fino = gvidaj + nomo.len();
        if fino < linio.len() {
            linio[gvidaj..fino].copy_from_slice(nomo.as_bytes());
            linio[fino] = b' ';
            *longo = fino + 1;
        }
        return false;
    }
    let komuna = komuna.unwrap_or("");
    if komuna.len() > vorta_longo && gvidaj + komuna.len() <= linio.len() {
        linio[gvidaj..gvidaj + komuna.len()].copy_from_slice(komuna.as_bytes());
        *longo = gvidaj + komuna.len();
        return false;
    }
    biblioteko::skribi("\n");
    if let Ok(teksto) = core::str::from_utf8(&linio[..*longo]) {
        let vorto = teksto.trim_start();
        for (nomo, _) in KOMANDOJ {
            if antaŭas(nomo, vorto) {
                biblioteko::skribi(nomo);
                biblioteko::skribi("  ");
            }
        }
    }
    biblioteko::skribi("\n");
    true
}

enum Eskapo {
    Supra,
    Malsupra,
    Alia,
}

/// Konsumu ANSI-sekvencon post `ESC` kaj raportu la sagoklavojn.
fn skipo_eskapon() -> Eskapo {
    let unua = biblioteko::enigi();
    if unua < 0 {
        return Eskapo::Alia;
    }
    let unua = unua as u8;
    if unua != b'[' && unua != b'O' {
        return Eskapo::Alia;
    }
    let dua = biblioteko::enigi();
    if dua < 0 {
        return Eskapo::Alia;
    }
    let dua = dua as u8;
    if (0x40..=0x7e).contains(&dua) {
        return match dua {
            b'A' => Eskapo::Supra,
            b'B' => Eskapo::Malsupra,
            _ => Eskapo::Alia,
        };
    }
    loop {
        let kodo = biblioteko::enigi();
        if kodo < 0 {
            return Eskapo::Alia;
        }
        if (0x40..=0x7e).contains(&(kodo as u8)) {
            return Eskapo::Alia;
        }
    }
}

enum Legaĵo {
    Teksto(usize),
    Interrompo,
    Fino,
}

/// Legu unu linion el la seria konzolo; la seria terminalo ne faras lokan
/// reson, do la ŝelo redesegnas la linion post ĉiu bajto (C17.6).
fn legi_linion(bufro: &mut [u8]) -> Legaĵo {
    let historio = historio();
    let mut longo = 0usize;
    let mut vizito = 0usize;
    let mut provizora = [0u8; LINIA_MAKSIMO];
    let mut provizora_longo = 0usize;
    loop {
        let kodo = biblioteko::enigi();
        if kodo < 0 {
            return Legaĵo::Fino;
        }
        let bajto = kodo as u8;
        match bajto {
            b'\r' | b'\n' => {
                if let Ok(teksto) = core::str::from_utf8(&bufro[..longo]) {
                    historio.aldoni(teksto);
                }
                biblioteko::skribi("\n");
                return Legaĵo::Teksto(longo);
            }
            0x7f | 0x08 => {
                if longo > 0 {
                    longo -= 1;
                    while longo > 0 && (bufro[longo] & 0xC0) == 0x80 {
                        longo -= 1;
                    }
                    desegni(&bufro[..longo]);
                }
            }
            0x03 => {
                biblioteko::skribi("^C\n");
                return Legaĵo::Interrompo;
            }
            0x04 => {
                if longo == 0 {
                    biblioteko::skribi("\n");
                    return Legaĵo::Fino;
                }
            }
            0x09 => {
                // La kandidataro, se preskribita, jam finiĝas per nova linio.
                kompletigi(bufro, &mut longo);
                desegni(&bufro[..longo]);
            }
            0x0c => {
                purigi();
                desegni(&bufro[..longo]);
            }
            0x1b => match skipo_eskapon() {
                Eskapo::Supra => {
                    if let Some((nova, teksto)) = historio.antaŭa(vizito) {
                        if vizito == 0 {
                            provizora[..longo].copy_from_slice(&bufro[..longo]);
                            provizora_longo = longo;
                        }
                        longo = teksto.len();
                        bufro[..longo].copy_from_slice(teksto);
                        vizito = nova;
                        desegni(&bufro[..longo]);
                    }
                }
                Eskapo::Malsupra => {
                    if vizito > 0 {
                        vizito -= 1;
                        if vizito == 0 {
                            longo = provizora_longo;
                            bufro[..longo].copy_from_slice(&provizora[..longo]);
                        } else if let Some((_, teksto)) = historio.antaŭa(vizito - 1) {
                            longo = teksto.len();
                            bufro[..longo].copy_from_slice(teksto);
                        }
                        desegni(&bufro[..longo]);
                    }
                }
                Eskapo::Alia => {}
            },
            0x20..=0xff if longo < bufro.len() => {
                bufro[longo] = bajto;
                longo += 1;
                desegni(&bufro[..longo]);
            }
            _ => {}
        }
    }
}

/// Enirpunkto de la ŝela procezo; ĝi vivas ĝis `fini` aŭ Ctrl+D.
pub fn starti() -> ! {
    let Some(kupo) = biblioteko::aloki(LINIA_MAKSIMO) else {
        biblioteko::skribi("[ligo] la kupo ne estas disponebla\n");
        biblioteko::fini(3)
    };
    // SAFETY: `aloki` redonas blokon el la kupo, kiun `SYS_BRK` mapis en la
    // proceza spaco; la bufro estas 256 bajtoj kaj la bloko restas valida ĝis
    // la procezo finiĝas per `fini`, do ĝia vivo kovras la tutan ŝelon.
    let bufro = unsafe { core::slice::from_raw_parts_mut(kupo, LINIA_MAKSIMO) };
    bonvenon();
    loop {
        skribi_prompton();
        match legi_linion(bufro) {
            Legaĵo::Teksto(longo) => {
                if let Ok(teksto) = core::str::from_utf8(&bufro[..longo]) {
                    plenumi(teksto);
                }
            }
            Legaĵo::Interrompo => {}
            Legaĵo::Fino => biblioteko::fini(0),
        }
    }
}
