//! Sistemvokoj: la `int 0x80`-interkonsento kaj ĝia kerna efektivigo.
//!
//! Registra ABI (vidu `libmikerno::syscall`): `rax` = numero,
//! `rdi`/`rsi`/`rdx`/`r10` = argumentoj, `rax` = rezulto. La arkitektura
//! tavolo legas la registrojn per `SistemvokaKadro` kaj skribas la rezulton
//! reen; ĉi tiu modulo nur decidas, kion la voko signifas.

use core::mem::size_of;

use libmikerno::registro::{Dosierero, MAKS_REGISTROJ, ProcezoRekordo};
use libmikerno::syscall::{
    SYS_BRK, SYS_CEDI, SYS_DONI, SYS_DORMI, SYS_ENIGI, SYS_EXEC, SYS_FERMI, SYS_FINI,
    SYS_KREI_FINPUNKTON, SYS_LANĜI, SYS_LEGI, SYS_LISTI, SYS_MORTIGI, SYS_PROCEZOJ, SYS_REVOKI,
    SYS_RICEVI, SYS_RICEVI_TEMPO, SYS_SENDI, SYS_SENDI_TEMPO, SYS_SKRIBI,
};
use libmikerno::{Mesago, Permesoj, Tenilo};

use crate::konservado::snds;
use crate::kunhavigita::Kunhavigita;
use crate::memoro::virtuala::{PaĝaTablo, UZANTA_STAKO_GRANDE, UZANTA_STAKO_SUPRO};

use super::procezo::MAKSAJ_PROCEZOJ;
use super::{ik, planisto};

/// `-1`: la argumentoj estas malvalidaj.
pub const MALĜUSTA_ARG: u64 = (-1i64) as u64;
/// `-2`: la numero ne apartenas al la interkonsento.
pub const NEKONATA_VOKO: u64 = (-2i64) as u64;
/// `-3`: la celo de mesaĝo ne estas fadeno.
pub const NEVALIDA_CELLOKO: u64 = (-3i64) as u64;
/// `-9`: la tenilo ne validas, la generacio malakordas aŭ la fendo fermiĝis.
pub const NEVALIDA_TENILO: u64 = (-9i64) as u64;
/// `-28`: la kapabla aŭ finpunkta tabelo estas plena.
pub const TABELO_PLENA: u64 = (-28i64) as u64;
/// `-32`: la finpunkto estis revokita aŭ ĝia posedanto finiĝis.
pub const FINPUNKTO_REVOKITA: u64 = (-32i64) as u64;
/// `-13`: la operacio estas malpermesata; la nukleo kaj la ŝelo ne mortigeblas.
pub const NEPERMITITA: u64 = (-13i64) as u64;
/// `-17`: la dosiero aŭ procezo ne troviĝas.
pub const NE_TROVITA: u64 = (-17i64) as u64;

const _: () = {
    assert!(NEVALIDA_TENILO == libmikerno::syscall::NEVALIDA_TENILO as u64);
    assert!(TABELO_PLENA == libmikerno::syscall::TABELO_PLENA as u64);
    assert!(FINPUNKTO_REVOKITA == libmikerno::syscall::FINPUNKTO_REVOKITA as u64);
    assert!(NEPERMITITA == libmikerno::syscall::NEPERMITITA as u64);
    assert!(NE_TROVITA == libmikerno::syscall::NE_TROVITA as u64);
};

/// La sistemvoka erar-kodo por IK-malsukceso.
fn kodo(eraro: ik::IkEraro) -> u64 {
    eraro.kodo() as u64
}

/// Maksimuma longo de unu `SYS_SKRIBI`-bufro.
const MAKS_SKRIBO: usize = 4096;
/// Maksimuma longo de `SYS_EXEC`-vojo.
const MAKS_VOJO: usize = 256;

/// Kerna kopiujo por `SYS_SKRIBI`: la uzanta bufro estas legata nur dum la
/// mallonga `STAC`-fenestro, kaj la formatado poste legas ĉi tiun bufron. La
/// aliro estas unuprocesora kaj okazas kun interrompoj malŝaltitaj.
static SKRIBA_BUFRO: Kunhavigita<[u8; MAKS_SKRIBO]> = Kunhavigita::nova([0; MAKS_SKRIBO]);

/// Kerna preparbufro por `SYS_LISTI`: la eroj estas plenigitaj ĉi tie kaj
/// poste kopiitaj al Ring 3 dum mallonga `STAC`-fenestro.
static LISTI_BUFRO: Kunhavigita<[Dosierero; MAKS_REGISTROJ]> =
    Kunhavigita::nova([Dosierero::nova(); MAKS_REGISTROJ]);
/// Kerna preparbufro por `SYS_PROCEZOJ`.
static PROCEZOJ_BUFRO: Kunhavigita<[ProcezoRekordo; MAKS_REGISTROJ]> =
    Kunhavigita::nova([ProcezoRekordo::nova(); MAKS_REGISTROJ]);
/// La plej granda dosiero, kiun `SYS_LEGI` transdonas en unu voko.
const MAKS_LEGO: usize = 4096;
static LEGA_BUFRO: Kunhavigita<[u8; MAKS_LEGO]> = Kunhavigita::nova([0; MAKS_LEGO]);

/// Efektivigu unu sistemvokon.
///
/// `el_ring3` estas vera nur por vokoj el Ring 3: la kerno mem estas fidinda
/// kaj rajtas uzi kernajn montrilojn, dum uzantaj montriloj estas limigitaj al
/// la anoncitaj bildaj kaj stakaj regionoj.
pub fn voki(numero: usize, a0: u64, a1: u64, a2: u64, a3: u64, el_ring3: bool) -> u64 {
    match numero {
        SYS_SKRIBI => skribi(a0, a1, el_ring3),
        SYS_FINI => fini(),
        SYS_CEDI => cedi(),
        SYS_SENDI => sendi(a0, a1, el_ring3),
        SYS_SENDI_TEMPO => sendi_tempa(a0, a1, a3, el_ring3),
        SYS_RICEVI => ricevi(a0, a1, el_ring3),
        SYS_RICEVI_TEMPO => ricevi_tempa(a0, a1, a2, a3, el_ring3),
        SYS_KREI_FINPUNKTON => krei_finpunkton(a0),
        SYS_DONI => doni(a0, a1, a2),
        SYS_REVOKI => revoki(a0, a1),
        SYS_FERMI => fermi(a0),
        SYS_DORMI => dormi(a0),
        SYS_BRK => brk(a0, el_ring3),
        SYS_EXEC => exec(a0, a1, el_ring3),
        SYS_ENIGI => enigi(el_ring3),
        SYS_LISTI => listi(a0, a1, a2, a3, el_ring3),
        SYS_LEGI => legi(a0, a1, a2, a3, el_ring3),
        SYS_PROCEZOJ => procezoj(a0, a1, el_ring3),
        SYS_MORTIGI => mortigi(a0, el_ring3),
        SYS_LANĜI => lanĉi(a0, a1, el_ring3),
        _ => NEKONATA_VOKO,
    }
}

/// `rdi` = nova kupo-limo (`0` nur demandas).
fn brk(nova: u64, el_ring3: bool) -> u64 {
    if !el_ring3 {
        return MALĜUSTA_ARG;
    }
    let id = planisto::nuna_procezo();
    if id == 0 {
        return MALĜUSTA_ARG;
    }
    let _gardo = crate::interrompo::InterrompaGardo::nova();
    let Some(procezo) = super::procezo::akiri(id) else {
        return MALĜUSTA_ARG;
    };
    if nova == 0 {
        return procezo.kupaj_limoj().1;
    }
    match procezo.etendi_kupon(nova, super::kadra_alokilo()) {
        Ok(limo) => limo,
        Err(_) => MALĜUSTA_ARG,
    }
}

/// `rdi` = montrilo al la vojo, `rsi` = longo. Sukceso neniam revenas.
fn exec(montrilo: u64, longo: u64, el_ring3: bool) -> u64 {
    let Some((bufro, longo)) = kopii_vojon(montrilo, longo, el_ring3) else {
        return MALĜUSTA_ARG;
    };
    let id = planisto::nuna_procezo();
    if id == 0 {
        return MALĜUSTA_ARG;
    }
    let Ok(vojo) = core::str::from_utf8(&bufro[..longo]) else {
        return MALĜUSTA_ARG;
    };

    // `anstataŭigi` redonas nur malsukceson; sukceso transiras al Ring 3.
    let eraro = super::exec::anstataŭigi(id, vojo);
    crate::skribu!("[kerno] exec: procezo {} malsukcesis: {}\n", id, eraro);
    MALĜUSTA_ARG
}

/// Kopiu kaj validigu Ring 3-vojon: ne-nula, limigita, ene de la anoncita
/// regiono, sen interna NUL.
fn kopii_vojon(montrilo: u64, longo: u64, el_ring3: bool) -> Option<([u8; MAKS_VOJO], usize)> {
    let longo = usize::try_from(longo).ok()?;
    if longo == 0 || longo > MAKS_VOJO || !kontroli_regionon(montrilo, longo, false, el_ring3) {
        return None;
    }
    let mut bufro = [0u8; MAKS_VOJO];
    {
        let _gardo = el_ring3.then(crate::arkitekturo::StacGardo::nova);
        // SAFETY: la regiona kontrolo certigas, ke la Ring 3-montrilo estas
        // mapita kaj legebla; `longo` ne superas la kernan bukron. Ring
        // 0-montriloj estas fidindaj.
        unsafe {
            core::ptr::copy_nonoverlapping(montrilo as *const u8, bufro.as_mut_ptr(), longo);
        }
    }
    if bufro[..longo].contains(&0) {
        return None;
    }
    Some((bufro, longo))
}

/// `SYS_ENIGI`: legu unubajton el la seria konzolo; bloku ĝis unu alvenas.
fn enigi(el_ring3: bool) -> u64 {
    if !el_ring3 || planisto::nuna_procezo() == 0 {
        return MALĜUSTA_ARG;
    }
    loop {
        if let Some(bajto) = crate::arkitekturo::legi_konzolan() {
            return bajto as u64;
        }
        planisto::dormi(10);
    }
}

/// `SYS_LISTI`: `rdi` = vojo, `rsi` = longo, `rdx` = celo, `r10` = grando.
fn listi(montrilo: u64, longo: u64, celo: u64, celgrando: u64, el_ring3: bool) -> u64 {
    let Some((bufro, longo)) = kopii_vojon(montrilo, longo, el_ring3) else {
        return MALĜUSTA_ARG;
    };
    let Ok(vojo) = core::str::from_utf8(&bufro[..longo]) else {
        return MALĜUSTA_ARG;
    };
    let Ok(celgrando) = usize::try_from(celgrando) else {
        return MALĜUSTA_ARG;
    };
    let eroj = (celgrando / size_of::<Dosierero>()).min(MAKS_REGISTROJ);
    if eroj == 0 || !kontroli_regionon(celo, eroj * size_of::<Dosierero>(), true, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let bufro = LISTI_BUFRO.akiri();
    match snds::listi_registrojn(vojo, &mut bufro[..eroj]) {
        Ok(nombro) => {
            kopii_al_uzanto(
                celo,
                bufro.as_ptr().cast(),
                nombro * size_of::<Dosierero>(),
                el_ring3,
            );
            nombro as u64
        }
        Err(eraro) => kodo_de_snds(eraro),
    }
}

/// `SYS_LEGI`: `rdi` = vojo, `rsi` = longo, `rdx` = celo, `r10` = grando.
fn legi(montrilo: u64, longo: u64, celo: u64, celgrando: u64, el_ring3: bool) -> u64 {
    let Some((bufro, longo)) = kopii_vojon(montrilo, longo, el_ring3) else {
        return MALĜUSTA_ARG;
    };
    let Ok(vojo) = core::str::from_utf8(&bufro[..longo]) else {
        return MALĜUSTA_ARG;
    };
    let Ok(celgrando) = usize::try_from(celgrando) else {
        return MALĜUSTA_ARG;
    };
    if celgrando == 0 || !kontroli_regionon(celo, celgrando, true, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let dosiero = match snds::malfermi(vojo) {
        Ok(dosiero) => dosiero,
        Err(eraro) => return kodo_de_snds(eraro),
    };
    if dosiero.grando() as usize > celgrando || dosiero.grando() as usize > MAKS_LEGO {
        return TABELO_PLENA;
    }
    let bufro = LEGA_BUFRO.akiri();
    match snds::legi(&dosiero, &mut bufro[..celgrando.min(MAKS_LEGO)]) {
        Ok(nombro) => {
            kopii_al_uzanto(celo, bufro.as_ptr(), nombro, el_ring3);
            nombro as u64
        }
        Err(eraro) => kodo_de_snds(eraro),
    }
}

/// `SYS_PROCEZOJ`: `rdi` = celo, `rsi` = grando.
fn procezoj(celo: u64, celgrando: u64, el_ring3: bool) -> u64 {
    let Ok(celgrando) = usize::try_from(celgrando) else {
        return MALĜUSTA_ARG;
    };
    let eroj = (celgrando / size_of::<ProcezoRekordo>()).min(MAKS_REGISTROJ);
    if eroj == 0 || !kontroli_regionon(celo, eroj * size_of::<ProcezoRekordo>(), true, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let bufro = PROCEZOJ_BUFRO.akiri();
    let nombro = super::procezo::listi(&mut bufro[..eroj]);
    kopii_al_uzanto(
        celo,
        bufro.as_ptr().cast(),
        nombro * size_of::<ProcezoRekordo>(),
        el_ring3,
    );
    nombro as u64
}

/// `SYS_MORTIGI`: `rdi` = proceza identigilo. La nukleo (`0`) kaj la ŝelo
/// (`1`) estas nemortigeblaj.
fn mortigi(pid: u64, el_ring3: bool) -> u64 {
    if !el_ring3 || planisto::nuna_procezo() == 0 {
        return MALĜUSTA_ARG;
    }
    if pid == 0 || pid == super::ŜELA_PROCEZA_ID {
        return NEPERMITITA;
    }
    if planisto::mortigi_procezon(pid) {
        0
    } else {
        NE_TROVITA
    }
}

/// `SYS_LANĜI`: `rdi` = vojo, `rsi` = longo. Kreu novan procezon el SNDS.
fn lanĉi(montrilo: u64, longo: u64, el_ring3: bool) -> u64 {
    if !el_ring3 {
        return MALĜUSTA_ARG;
    }
    let Some((bufro, longo)) = kopii_vojon(montrilo, longo, el_ring3) else {
        return MALĜUSTA_ARG;
    };
    let Ok(vojo) = core::str::from_utf8(&bufro[..longo]) else {
        return MALĜUSTA_ARG;
    };
    match super::exec::lanĉi(vojo) {
        Ok(id) => id,
        Err(eraro) => {
            crate::skribu!("[kerno] lanĉi: {eraro}\n");
            match eraro {
                super::exec::ExecEraro::Dosiersistemo(snds::Eraro::NeTrovo) => NE_TROVITA,
                super::exec::ExecEraro::Elĉerpita => TABELO_PLENA,
                _ => MALĜUSTA_ARG,
            }
        }
    }
}

/// Kopiu kernan bufron al Ring 3 dum `STAC`-fenestro.
fn kopii_al_uzanto(celo: u64, fonto: *const u8, longo: usize, el_ring3: bool) {
    let _gardo = el_ring3.then(crate::arkitekturo::StacGardo::nova);
    // SAFETY: la celo trapasis `kontroli_regionon` por skribo; la fonto estas
    // privata kerna bufro; la intervaloj ne interkovriĝas.
    unsafe {
        core::ptr::copy_nonoverlapping(fonto, celo as *mut u8, longo);
    }
}

/// La sistemvoka kodo por SNDS-malsukceso.
fn kodo_de_snds(eraro: snds::Eraro) -> u64 {
    match eraro {
        snds::Eraro::NeTrovo | snds::Eraro::NeDosiero => NE_TROVITA,
        snds::Eraro::BufroTroMalgranda => TABELO_PLENA,
        _ => MALĜUSTA_ARG,
    }
}

/// `rdi` = daŭro en milisekundoj.
fn dormi(millisekundoj: u64) -> u64 {
    if planisto::dormi(millisekundoj) {
        0
    } else {
        MALĜUSTA_ARG
    }
}

fn skribi(montrilo: u64, longo: u64, el_ring3: bool) -> u64 {
    let Ok(longo) = usize::try_from(longo) else {
        return MALĜUSTA_ARG;
    };
    if !kontroli_bufron(montrilo, longo, false, el_ring3) {
        return MALĜUSTA_ARG;
    }
    // Ring 3-bufro estas kopiata en kernan bufron dum mallonga `STAC`-fenestro;
    // la formatado poste legas nur la kernan kopion. Ring 0-montriloj estas
    // fidindaj kaj ne bezonas fenestron.
    let bufro: &[u8] = if el_ring3 {
        let celo = SKRIBA_BUFRO.akiri();
        {
            let _gardo = crate::arkitekturo::StacGardo::nova();
            // SAFETY: Ring 3-montriloj trapasis la regionan kontrolon, do la
            // bufro estas ne-nula, maplitan kaj ene de la anoncita bildo aŭ
            // stako; `longo` estas limigita per `MAKS_SKRIBO`. La celo estas
            // privata kerna bufro de sama grando, kaj interrompoj estas
            // malŝaltitaj, do neniu alia kopio samtempe uzas ĝin.
            unsafe {
                core::ptr::copy_nonoverlapping(montrilo as *const u8, celo.as_mut_ptr(), longo);
            }
        }
        &celo[..longo]
    } else {
        // SAFETY: Ring 0-montriloj estas fidindaj kaj la longo estas limigita.
        unsafe { core::slice::from_raw_parts(montrilo as *const u8, longo) }
    };
    match core::str::from_utf8(bufro) {
        Ok(teksto) => {
            crate::skribu!("{}", teksto);
            0
        }
        Err(_) => MALĜUSTA_ARG,
    }
}

fn fini() -> u64 {
    // `morti` neniam revenas; post ĝi ne ekzistas resuriga kodo.
    planisto::morti()
}

fn cedi() -> u64 {
    planisto::cedi();
    0
}

fn sendi(tenilo: u64, montrilo: u64, el_ring3: bool) -> u64 {
    // La montrila kontrolo venas antaŭ la tenila (C11): malnova Ring 3-provo
    // kun malvalida celo devas ankoraŭ redoni `-1`, ne `-9`.
    if !kontroli_mesagon(montrilo, false, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let mesago = {
        let _gardo = el_ring3.then(crate::arkitekturo::StacGardo::nova);
        // SAFETY: la montrilo estas ne-nula kaj (por Ring 3) en la anoncita
        // regiono; `Mesago` estas `Copy`, do unufoja legado sufiĉas. La
        // `STAC`-fenestro kovras nur la legadon, ne la IK-vokon.
        unsafe { core::ptr::read_unaligned(montrilo as *const Mesago) }
    };
    if mesago.longo() > ik::MAKSA_MESAGA_LONGO {
        // C2: `MesagujoPlena` (-4) restas nur por la ne-blokebla kerna vojo;
        // tro longa mesaĝo havas sian propran kodon ekde Fazo 5.
        return kodo(ik::IkEraro::TroLongaMesago);
    }
    let Some(tenilo) = el_tenilo(tenilo) else {
        return NEVALIDA_TENILO;
    };
    match ik::sendi_kapable(
        planisto::nuna_procezo(),
        tenilo,
        &mesago,
        ik::Atendo::Senfina,
    ) {
        Ok(()) => 0,
        Err(eraro) => kodo(eraro),
    }
}

/// `rdi` = fonta tenilo; `rsi` = celo-procezo; `rdx` = novaj permesoj.
///
/// La novaj permesoj devas esti subaro de la fontaj; `0` aŭ nekonata bito estas
/// malĝusta argumento.
/// Malpakigu permesojn el la 8 malaltaj bitoj; `None` se iu pli alta bito
/// estas ŝaltita aŭ se neniu konata permeso restas.
fn el_permesoj(valoro: u64) -> Option<Permesoj> {
    if valoro >> 8 != 0 {
        return None;
    }
    Permesoj::el_bitoj(valoro as u8).filter(|permesoj| permesoj.bitoj() != 0)
}

fn doni(fonto: u64, celo: u64, permesoj: u64) -> u64 {
    let Some(permesoj) = el_permesoj(permesoj) else {
        return MALĜUSTA_ARG;
    };
    let Some(fonto) = el_tenilo(fonto) else {
        return NEVALIDA_TENILO;
    };
    if celo == 0 || celo as usize >= MAKSAJ_PROCEZOJ {
        return NEVALIDA_CELLOKO;
    }
    match ik::doni(planisto::nuna_procezo(), fonto, celo, permesoj) {
        Ok(tenilo) => tenilo.nombro() as u64,
        Err(eraro) => kodo(eraro),
    }
}

fn ricevi(tenilo: u64, celo: u64, el_ring3: bool) -> u64 {
    // C11: la celo-montrilo estas kontrolata antaŭ la tenilo.
    if !kontroli_mesagon(celo, true, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let Some(tenilo) = el_tenilo(tenilo) else {
        return NEVALIDA_TENILO;
    };
    // `ricevi_kapable` rajtas bloki kaj cedi la procesoron, do ĝi restas
    // ekster la `STAC`-fenestro.
    let mesago =
        match ik::ricevi_kapable(planisto::nuna_procezo(), tenilo, None, ik::Atendo::Senfina) {
            Ok(mesago) => mesago,
            Err(eraro) => return kodo(eraro),
        };
    {
        let _gardo = el_ring3.then(crate::arkitekturo::StacGardo::nova);
        // SAFETY: `celo` estas ne-nula kaj (por Ring 3) en la anoncita regiono;
        // `Mesago` estas `Copy` kaj la skribo okazas unufoje, sen referenco
        // restanta. La fenestro kovras nur la skribon.
        unsafe {
            core::ptr::write_unaligned(celo as *mut Mesago, mesago);
        }
    }
    0
}

/// `rdi` = tenilo; `rsi` = celo; `rdx` = fonta fadeno (`0` = ajna);
/// `r10` = tempolimo en milisekundoj (C4, C15).
fn ricevi_tempa(tenilo: u64, celo: u64, fonto: u64, tempolimo: u64, el_ring3: bool) -> u64 {
    // C11: la celo-montrilo estas kontrolata antaŭ la tenilo.
    if !kontroli_mesagon(celo, true, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let Some(tenilo) = el_tenilo(tenilo) else {
        return NEVALIDA_TENILO;
    };
    let Some(atendo) = ik::el_tempolimo(tempolimo) else {
        return MALĜUSTA_ARG;
    };
    let fonto = if fonto == 0 {
        None
    } else if fonto <= planisto::MAKSAJ_FADENOJ as u64 {
        Some(fonto)
    } else {
        return MALĜUSTA_ARG;
    };
    let mesago = match ik::ricevi_kapable(planisto::nuna_procezo(), tenilo, fonto, atendo) {
        Ok(mesago) => mesago,
        Err(eraro) => return kodo(eraro),
    };
    {
        let _gardo = el_ring3.then(crate::arkitekturo::StacGardo::nova);
        // SAFETY: sama kiel ĉe `ricevi`: la celo estas kontrolata supre kaj la
        // `Mesago` estas skribata unufoje dum la `STAC`-fenestro.
        unsafe {
            core::ptr::write_unaligned(celo as *mut Mesago, mesago);
        }
    }
    0
}

/// `rdi` = tenilo; `rsi` = mesaĝo; `rdx` estas rezervita; `r10` = tempolimo en
/// milisekundoj (C15).
fn sendi_tempa(tenilo: u64, montrilo: u64, tempolimo: u64, el_ring3: bool) -> u64 {
    // C11: la montrila kontrolo venas antaŭ la tenila.
    if !kontroli_mesagon(montrilo, false, el_ring3) {
        return MALĜUSTA_ARG;
    }
    let mesago = {
        let _gardo = el_ring3.then(crate::arkitekturo::StacGardo::nova);
        // SAFETY: sama kiel ĉe `sendi`: la montrilo estas ne-nula kaj (por
        // Ring 3) en la anoncita regiono; la `STAC`-fenestro kovras nur la
        // legadon.
        unsafe { core::ptr::read_unaligned(montrilo as *const Mesago) }
    };
    if mesago.longo() > ik::MAKSA_MESAGA_LONGO {
        return kodo(ik::IkEraro::TroLongaMesago);
    }
    let Some(tenilo) = el_tenilo(tenilo) else {
        return NEVALIDA_TENILO;
    };
    let Some(atendo) = ik::el_tempolimo(tempolimo) else {
        return MALĜUSTA_ARG;
    };
    match ik::sendi_kapable(planisto::nuna_procezo(), tenilo, &mesago, atendo) {
        Ok(()) => 0,
        Err(eraro) => kodo(eraro),
    }
}

/// Malpakigu tenilon el la sistemvoka argumento; `None` se ĝi ne eniras 32
/// bitojn aŭ se la kodigo estas malvalida.
fn el_tenilo(valoro: u64) -> Option<Tenilo> {
    let valoro = u32::try_from(valoro).ok()?;
    Tenilo::el_nombro(valoro)
}

/// `rdi` = permesoj; `0` aŭ nekonata bito estas malĝusta argumento.
fn krei_finpunkton(permesoj: u64) -> u64 {
    let Some(permesoj) = el_permesoj(permesoj) else {
        return MALĜUSTA_ARG;
    };
    match ik::krei_finpunkton(planisto::nuna_procezo(), permesoj) {
        Ok(tenilo) => tenilo.nombro() as u64,
        Err(eraro) => kodo(eraro),
    }
}

/// `rdi` = tenilo; fermas nur la lokan kapablofendon de la nuna procezo.
fn fermi(tenilo: u64) -> u64 {
    let Some(tenilo) = el_tenilo(tenilo) else {
        return NEVALIDA_TENILO;
    };
    match ik::fermi(planisto::nuna_procezo(), tenilo) {
        Ok(()) => 0,
        Err(eraro) => kodo(eraro),
    }
}

/// `rdi` = tenilo; `rsi` devas esti `0` (C14). Revoko estas ĉiam tutfinpunkta.
fn revoki(tenilo: u64, rezervita: u64) -> u64 {
    if rezervita != 0 {
        return MALĜUSTA_ARG;
    }
    let Some(tenilo) = el_tenilo(tenilo) else {
        return NEVALIDA_TENILO;
    };
    match ik::revoki(planisto::nuna_procezo(), tenilo) {
        Ok(()) => 0,
        Err(eraro) => kodo(eraro),
    }
}

/// Ne-nula, limigita kaj (por Ring 3) ene de la anoncita uzanta regiono.
fn kontroli_bufron(montrilo: u64, longo: usize, skriba: bool, el_ring3: bool) -> bool {
    if montrilo == 0 || longo == 0 || longo > MAKS_SKRIBO {
        return false;
    }
    kontroli_regionon(montrilo, longo, skriba, el_ring3)
}

fn kontroli_mesagon(montrilo: u64, skriba: bool, el_ring3: bool) -> bool {
    if montrilo == 0 {
        return false;
    }
    kontroli_regionon(montrilo, size_of::<Mesago>(), skriba, el_ring3)
}

/// Kontrolu Ring 3-montrilon: sen ciferpremo, ene de la anoncita bildo, stako
/// aŭ kupo, kaj kun la postulata folia permeso — `skriba` por skribo, alie por
/// legado. Ring 0-montriloj estas fidindaj kaj trapasas sen kontrolo.
fn kontroli_regionon(montrilo: u64, longo: usize, skriba: bool, el_ring3: bool) -> bool {
    let Some(fino) = montrilo.checked_add(longo as u64) else {
        return false;
    };
    if !el_ring3 {
        return true;
    }
    let id = super::planisto::nuna_procezo();
    let Some(procezo) = super::procezo::akiri(id) else {
        return false;
    };
    let (bilda_komenco, bilda_fino) = procezo.bildaj_limoj();
    let en_bildo = bilda_fino > bilda_komenco && montrilo >= bilda_komenco && fino <= bilda_fino;
    let staka_komenco = UZANTA_STAKO_SUPRO - UZANTA_STAKO_GRANDE as u64;
    let en_stako = montrilo >= staka_komenco && fino <= UZANTA_STAKO_SUPRO;
    let (kupa_komenco, kupa_fino) = procezo.kupaj_limoj();
    let en_kupo = kupa_fino > kupa_komenco && montrilo >= kupa_komenco && fino <= kupa_fino;
    if !en_bildo && !en_stako && !en_kupo {
        crate::skribu!(
            "[eraro] procezo {}: montrilo 0x{:x} (longo {}) estas ekster la anoncita regiono\n",
            id,
            montrilo,
            longo
        );
        return false;
    }
    if !procezo.spaco().kontroli_uzantan(montrilo, longo, skriba) {
        crate::skribu!(
            "[eraro] procezo {}: montrilo 0x{:x} (longo {}, skriba {}) ne havas la postulatan folian permeson\n",
            id,
            montrilo,
            longo,
            skriba
        );
        return false;
    }
    true
}

// Limdata veko ne signifas tujan kuradon: la preta vico povas postuli unu rondon.
pub fn dorma_provo(id: u64, millisekundoj: u64) {
    let antaŭ = crate::arkitekturo::temporizilaj_tikoj();
    let kodo = invoki(SYS_DORMI, [millisekundoj, 0, 0, 0]) as i64;
    let pasintaj = crate::arkitekturo::temporizilaj_tikoj() - antaŭ;
    let petitaj = millisekundoj.div_ceil(10).max(1);
    let tolero = planisto::MAKSA_PRETA_ATENDO + 1;
    let bone = kodo == 0 && (petitaj..=petitaj + tolero).contains(&pasintaj);
    crate::skribu!(
        "[fadeno {}] dormo ({}): petitaj {} ms, pasintaj {} tikoj (kodo {})\n",
        id,
        if bone { "bone" } else { "MALBONE" },
        millisekundoj,
        pasintaj,
        kodo
    );
}

/// Ring 0-memtesto de la dispetĉo kaj de la argumenta kontrolo.
///
/// Vokata de la unua fadeno: `SYS_CEDI` tie vere cedas la procesoron, dum en la
/// praŝarĝa kunteksto ĝi nur revenus sen ŝanĝo.
pub fn testi_ring0() {
    #[cfg(target_arch = "x86_64")]
    let teksto = "saluton el Ring 0 tra int 0x80";
    #[cfg(target_arch = "aarch64")]
    let teksto = "saluton el Ring 0 tra svc #0";
    let skribo = invoki(
        SYS_SKRIBI,
        [teksto.as_ptr() as u64, teksto.len() as u64, 0, 0],
    );
    let nekonata = invoki(0x7FFF, [0; 4]);
    let nula = invoki(SYS_SKRIBI, [0, 8, 0, 0]);
    let troa = invoki(
        SYS_SKRIBI,
        [teksto.as_ptr() as u64, (MAKS_SKRIBO + 1) as u64, 0, 0],
    );
    let cedo = invoki(SYS_CEDI, [0; 4]);
    // Mesaĝo, kies `longo` mensoge superas la buferon; nur la longa kontrolo
    // antaŭas la tenilan, do `-90` (ne `-1` aŭ `-9`).
    let mut tro_longa_mesago = ik::Mesago::malplena();
    tro_longa_mesago.longo = ik::MAKSA_MESAGA_LONGO + 1;
    let tro_longa = invoki(SYS_SENDI, [0, &tro_longa_mesago as *const _ as u64, 0, 0]);
    // `int 0x80` estas programa interrompo: la dispetĉo ne rajtas sendi EOI-on.
    // La mezuro okazas kun interrompoj malŝaltitaj, do tiko ne ŝajnigas EOI-on.
    // `arm64` ne havas tiun vojon: `svc #0` ankaŭ estas programa escepto.
    #[cfg(target_arch = "x86_64")]
    let sen_eoi = {
        let _gardo = crate::interrompo::InterrompaGardo::nova();
        let antaŭ = crate::arkitekturo::konfirmoj_interrompoj();
        let _ = invoki(SYS_SKRIBI, [teksto.as_ptr() as u64, 0, 0, 0]);
        crate::arkitekturo::konfirmoj_interrompoj() == antaŭ
    };
    // La kerno ne havas kapablolinion (C8), do `KREI_FINPUNKTON` rifuzas per
    // `-28`, kaj `DONI`/`FERMI`/`REVOKI` kun valida-aspekta tenilo redonas
    // `-9`; ne-nula `rsi` ĉe `REVOKI` estas malĝusta argumento (`-1`).
    let krei = invoki(
        SYS_KREI_FINPUNKTON,
        [Permesoj::PLENAJ.bitoj() as u64, 0, 0, 0],
    );
    let doni = invoki(SYS_DONI, [0x100, 1, Permesoj::RICEVI.bitoj() as u64, 0]);
    let fermi = invoki(SYS_FERMI, [0; 4]);
    let revoki = invoki(SYS_REVOKI, [0x100, 0, 0, 0]);
    let revok_argumento = invoki(SYS_REVOKI, [0x100, 1, 0, 0]);
    #[cfg(target_arch = "x86_64")]
    crate::skribu!(
        "[sistemvoko] Ring 0-memtesto: skribo={}, nekonata={}, nula={}, troa={}, cedo={}, krei={}, doni={}, fermi={}, revoki={}, revok_argumento={}, tro_longa={}, sen_eoi={}\n",
        skribo as i64,
        nekonata as i64,
        nula as i64,
        troa as i64,
        cedo as i64,
        krei as i64,
        doni as i64,
        fermi as i64,
        revoki as i64,
        revok_argumento as i64,
        tro_longa as i64,
        sen_eoi
    );
    #[cfg(target_arch = "aarch64")]
    crate::skribu!(
        "[sistemvoko] Ring 0-memtesto: skribo={}, nekonata={}, nula={}, troa={}, cedo={}, krei={}, doni={}, fermi={}, revoki={}, revok_argumento={}, tro_longa={}\n",
        skribo as i64,
        nekonata as i64,
        nula as i64,
        troa as i64,
        cedo as i64,
        krei as i64,
        doni as i64,
        fermi as i64,
        revoki as i64,
        revok_argumento as i64,
        tro_longa as i64
    );
}

#[cfg(target_arch = "x86_64")]
fn invoki(numero: usize, argumentoj: [u64; 4]) -> u64 {
    let rezulto: u64;
    // SAFETY: `int 0x80` eniras la IDT-pordon instalitan de ĉi tiu modulo; la
    // registra ABI kongruas kun `libmikerno::syscall`. La voko ne tuŝas la
    // stakon (`nostack`).
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("rax") numero => rezulto,
            in("rdi") argumentoj[0],
            in("rsi") argumentoj[1],
            in("rdx") argumentoj[2],
            in("r10") argumentoj[3],
            options(nostack),
        );
    }
    rezulto
}

#[cfg(target_arch = "aarch64")]
fn invoki(numero: usize, argumentoj: [u64; 4]) -> u64 {
    crate::arkitekturo::invoki_sistemvokon(numero, argumentoj)
}
