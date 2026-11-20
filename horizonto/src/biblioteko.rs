//! Uzanta baza biblioteko: maldikaj envolvaĵoj ĉirkaŭ la sistemvokoj.
//!
//! La protokolo venas el `libmikerno`; ĉi tiu modulo nur donas al la uzanta
//! kodo Esperantajn nomojn sen montri la registran aranĝon.

use core::fmt::{self, Write};
use core::sync::atomic::{AtomicUsize, Ordering};

pub use libmikerno::kapablo::{Permesoj, Tenilo};
pub use libmikerno::mesago::Mesago;
pub use libmikerno::registro::{Dosierero, ProcezoRekordo};

/// Alinio de la kupaj blokoj; sufiĉa por ĉiu skalara tipo kaj por `Mesago`.
const KUPO_ALINIO: usize = 16;

/// La sekva libera adreso kaj la nuna `brk`-limo. `0` ĉe la fino signifas, ke
/// la unua aloko ankoraŭ demandos la bazon per `brk(0)`.
static KUPO_NUNA: AtomicUsize = AtomicUsize::new(0);
static KUPO_FINO: AtomicUsize = AtomicUsize::new(0);

fn alinii(valoro: usize) -> usize {
    valoro.div_ceil(KUPO_ALINIO) * KUPO_ALINIO
}

/// Aloku `grando` bajtojn el la uzanta kupo per puŝa (bump) alokilo.
///
/// La unua voko demandas la bazon per `brk(0)`; ĉiu etendo petas novan limon
/// kaj malsukcesas, se la kerno ne donas sufiĉe. `None` signifas, ke la kupo
/// estas elĉerpita aŭ ke la kerno rifuzis la vokon.
pub fn aloki(grando: usize) -> Option<*mut u8> {
    if grando == 0 {
        return None;
    }
    if KUPO_FINO.load(Ordering::Relaxed) == 0 {
        let bazo = libmikerno::syscall::brk(0);
        if bazo < 0 {
            return None;
        }
        KUPO_NUNA.store(bazo as usize, Ordering::Relaxed);
        KUPO_FINO.store(bazo as usize, Ordering::Relaxed);
    }
    let komenco = alinii(KUPO_NUNA.load(Ordering::Relaxed));
    let fino = komenco.checked_add(grando)?;
    if fino > KUPO_FINO.load(Ordering::Relaxed) {
        let nova = alinii(fino);
        let rezulto = libmikerno::syscall::brk(nova);
        if rezulto < 0 || (rezulto as usize) < fino {
            return None;
        }
        KUPO_FINO.store(rezulto as usize, Ordering::Relaxed);
    }
    KUPO_NUNA.store(fino, Ordering::Relaxed);
    // SAFETY: la intervalo `[komenco, fino)` kuŝas en la kupo, kiun la kerno
    // mapis per `brk`; la montrilo restas valida ĝis la procezo finiĝas aŭ
    // `exec` anstataŭigas la spacon.
    Some(komenco as *mut u8)
}

/// Skribu tekston al la kerna konzolo.
pub fn skribi(teksto: &str) -> isize {
    libmikerno::syscall::skribi(teksto)
}

/// Sufiĉa por unu protokola linio; pli longa teksto estas simple ignorata.
const FORMATO_LONGO: usize = 256;

struct Bufro {
    bajtoj: [u8; FORMATO_LONGO],
    longo: usize,
}

impl Bufro {
    const fn nova() -> Self {
        Self {
            bajtoj: [0; FORMATO_LONGO],
            longo: 0,
        }
    }
}

impl Write for Bufro {
    fn write_str(&mut self, teksto: &str) -> fmt::Result {
        let fino = self.longo + teksto.len();
        if fino > self.bajtoj.len() {
            return Err(fmt::Error);
        }
        self.bajtoj[self.longo..fino].copy_from_slice(teksto.as_bytes());
        self.longo = fino;
        Ok(())
    }
}

/// Skribu formatitan tekston per unu `SYS_SKRIBI`. Formati aparte kaj sendi la
/// tutan linion unufoje tenas la eliron atomia eĉ kiam la planisto cedas al la
/// alia procezo inter du linioj.
pub fn skribi_formaton(argumentoj: fmt::Arguments<'_>) {
    let mut bufro = Bufro::nova();
    if bufro.write_fmt(argumentoj).is_ok()
        && let Ok(teksto) = core::str::from_utf8(&bufro.bajtoj[..bufro.longo])
    {
        skribi(teksto);
    }
}

/// Sendu mesaĝon tra la tenilo.
pub fn sendi(tenilo: Tenilo, mesago: &Mesago) -> isize {
    libmikerno::syscall::sendi(tenilo, mesago)
}

/// Ricevu mesaĝon tra la tenilo en `celo`.
pub fn ricevi(tenilo: Tenilo, celo: &mut Mesago) -> isize {
    libmikerno::syscall::ricevi(tenilo, celo)
}

/// Ricevu mesaĝon kun fonta filtro (`0` = ajna) kaj tempolimo.
pub fn ricevi_kun_tempolimo(
    tenilo: Tenilo,
    celo: &mut Mesago,
    fonto: u64,
    tempolimo_ms: u64,
) -> isize {
    libmikerno::syscall::ricevi_kun_tempolimo(tenilo, celo, fonto, tempolimo_ms)
}

/// Sendu mesaĝon tra la tenilo kun tempolimo.
pub fn sendi_kun_tempolimo(tenilo: Tenilo, mesago: &Mesago, tempolimo_ms: u64) -> isize {
    libmikerno::syscall::sendi_kun_tempolimo(tenilo, mesago, tempolimo_ms)
}

/// Kreu finpunkton kun la donitaj permesoj; `> 0` estas la tenila nombro.
pub fn krei_finpunkton(permesoj: Permesoj) -> isize {
    libmikerno::syscall::krei_finpunkton(permesoj)
}

/// Fermu la lokan tenilon de finpunkto.
pub fn fermi(tenilo: Tenilo) -> isize {
    libmikerno::syscall::fermi(tenilo)
}

/// Revoku la tutan finpunkton de la tenilo.
pub fn revoki(tenilo: Tenilo) -> isize {
    libmikerno::syscall::revoki(tenilo)
}

/// Cedu la procesoron al la planisto.
pub fn cedi() -> isize {
    libmikerno::syscall::cedi()
}

/// Dormu almenaŭ la donitajn milisekundojn.
pub fn dormi(millisekundoj: u64) -> isize {
    libmikerno::syscall::dormi(millisekundoj)
}

/// Anstataŭigu la nunan procezon per la ELF-bildo `vojo` el SNDS.
///
/// Sukceso neniam revenas; malsukceso redonas negativan eraron.
pub fn exec(vojo: &str) -> isize {
    libmikerno::syscall::exec(vojo)
}

/// Legu unubajton el la seria konzolo; blokas ĝis bajto alvenas.
pub fn enigi() -> isize {
    libmikerno::syscall::enigi()
}

/// Listigu la erojn de `vojo` en `celo`; redonas la nombron da eroj.
pub fn listi(vojo: &str, celo: &mut [Dosierero]) -> isize {
    libmikerno::syscall::listi(vojo, celo)
}

/// Legu la tutan dosieron `vojo` en `celo`; redonas la nombron da bajtoj.
pub fn legi(vojo: &str, celo: &mut [u8]) -> isize {
    libmikerno::syscall::legi(vojo, celo)
}

/// Plenigu `celo` per la vivaj procezoj; redonas la nombron da eroj.
pub fn procezoj(celo: &mut [ProcezoRekordo]) -> isize {
    libmikerno::syscall::procezoj(celo)
}

/// Finigu la procezon `pid`; `0` aŭ negativa eraro.
pub fn mortigi(pid: u64) -> isize {
    libmikerno::syscall::mortigi(pid)
}

/// Kreu procezon el la ELF-bildo `vojo`; redonas la novan identigilon.
pub fn ruli(vojo: &str) -> isize {
    libmikerno::syscall::ruli(vojo)
}

/// Finu la nunan procezon. Se la kerno rifuzas, la procezo restas blokita.
pub fn fini(kodo: usize) -> ! {
    let _ = libmikerno::syscall::fini(kodo);
    loop {
        core::hint::spin_loop();
    }
}
