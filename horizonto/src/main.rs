//! Horizonto: uzanta procezo (Ring 3).
//!
//! La kerno liveras ĉi tiun bildon kiel praŝarĝan modulon, muntas ĝin super
//! 4 GiB kaj transiras al `horizonto_start` per `iretq`. La vokprotokolo estas
//! `int 0x80` kun la registra aranĝo el `libmikerno`. La sama bildo plenumas
//! du rolojn: la viktimo kontrolas sian sekretan paĝon kaj finas normale, dum
//! la spiono legas la duan paĝon, kiun nur la viktimo mapis.

#![no_std]
#![no_main]

mod biblioteko;

use core::panic::PanicInfo;

use libmikerno::starto::{
    MAGIA_NUMERO, REĜIMO_SEKRETO, REĜIMO_SPIONO, SEKRETA_ADRESO, SEKRETA_DUA_ADRESO,
    SEKRETA_ID_DEŜOVO,
};
use libmikerno::syscall::{
    SENFINA_TEMPOLIMO, SYS_KREI_FINPUNKTON, SYS_REVOKI, SYS_RICEVI, SYS_RICEVI_TEMPO, SYS_SENDI,
    SYS_SKRIBI, invoki,
};
use libmikerno::{BOOTSTRAP_TENILO, Mesago, Permesoj, Tenilo};

/// La mesaĝa aranĝo devas kongrui bajte kun la kerno (kun `libmikerno`).
const _: () = {
    assert!(libmikerno::ABI_VERSIO == 2);
    assert!(core::mem::size_of::<Mesago>() == 96);
    assert!(core::mem::align_of::<Mesago>() == 8);
    assert!(core::mem::offset_of!(Mesago, tipo) == 0);
    assert!(core::mem::offset_of!(Mesago, sendinto) == 8);
    assert!(core::mem::offset_of!(Mesago, ricevonto) == 16);
    assert!(core::mem::offset_of!(Mesago, longo) == 24);
};

const SALUTO: &str = "horizonto: procezo vekis en Ring 3.\n";

/// Enirpunkto de uzanta procezo. `reĝimo` venas en `rdi` kaj `identigilo` en
/// `rsi` el la kerna `iretq`-transiro.
#[unsafe(no_mangle)]
pub extern "C" fn horizonto_start(reĝimo: u64, identigilo: u64) -> ! {
    biblioteko::skribi(SALUTO);
    testi_identon(identigilo);
    testi_finpunktojn(identigilo);
    testi_rajtojn(identigilo);
    testi_revokon(identigilo);
    testi_tempolimojn(identigilo);
    testi_negativajn();
    match reĝimo {
        REĜIMO_SEKRETO => sekreta_provo(identigilo),
        REĜIMO_SPIONO => spioni(identigilo),
        _ => biblioteko::fini(0),
    }
}

/// Legu la propran sekretan paĝon kaj kontrolu la magian numeron kaj la
/// procezan identigilon. Ĉiu procezo vidas la saman virtualan adreson, do
/// kongruo pruvas, ke la paĝo tradukiĝas al propra fizika kadro.
fn kontroli_sekretan(identigilo: u64) -> bool {
    // SAFETY: la kerno mapis la sekretan paĝon je `SEKRETA_ADRESO` en la
    // proceza spaco antaŭ `iretq`, kaj la legado estas unuobla kaj nurlega.
    let (magio, konservita) = unsafe {
        (
            core::ptr::read_volatile(SEKRETA_ADRESO as *const u64),
            core::ptr::read_volatile((SEKRETA_ADRESO + SEKRETA_ID_DEŜOVO) as *const u64),
        )
    };
    if magio == MAGIA_NUMERO && konservita == identigilo {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] procezo {}: sekreta paĝo kongruas (magio 0x{:x}, id {})\n",
            identigilo, magio, konservita
        ));
        true
    } else {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] procezo {}: sekreta paĝo MALBONE (magio 0x{:x}, id {})\n",
            identigilo, magio, konservita
        ));
        false
    }
}

/// La viktimo: kontrolu sian paĝon, cedu plurfoje (por ke la spiono ruliĝu
/// intere) kaj finu per `SYS_FINI`.
fn sekreta_provo(identigilo: u64) -> ! {
    if !kontroli_sekretan(identigilo) {
        biblioteko::fini(1)
    }
    for fojo in 1..=6 {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] procezo {}: laboras (fojo {})\n",
            identigilo, fojo
        ));
        biblioteko::cedi();
        for n in 0..100_000u64 {
            core::hint::black_box(n);
        }
        biblioteko::cedi();
    }
    biblioteko::skribi_formaton(format_args!(
        "[ring3] procezo {}: pluvivas post la spiono; finas per SYS_FINI\n",
        identigilo
    ));
    biblioteko::fini(0)
}

/// La spiono: kontrolu sian paĝon, poste legu la duan paĝon de la viktimo.
/// Tiu adreso mankas en la spaca paĝtabelo, do la legado kaŭzas `#PF`.
fn spioni(identigilo: u64) -> ! {
    if !kontroli_sekretan(identigilo) {
        biblioteko::fini(1)
    }
    biblioteko::skribi_formaton(format_args!(
        "[ring3] procezo {}: spionas la viktimon\n",
        identigilo
    ));
    biblioteko::cedi();
    for n in 0..100_000u64 {
        core::hint::black_box(n);
    }
    biblioteko::cedi();
    biblioteko::skribi_formaton(format_args!(
        "[ring3] procezo {}: legas la duan sekretan paĝon ĉe 0x{:x}\n",
        identigilo, SEKRETA_DUA_ADRESO
    ));
    // SAFETY: la adreso intence mankas en la spaco de la spiono; la legado
    // devas kaŭzi Ring 3 `#PF`, kiun la kerno kaptas kaj mortigas la procezon.
    let fremda = unsafe { core::ptr::read_volatile(SEKRETA_DUA_ADRESO as *const u64) };
    core::hint::black_box(fremda);
    biblioteko::skribi("[ring3] ERARO: la fremda legado sukcesis; izolado mankas\n");
    biblioteko::fini(1)
}

/// A1: la kerno, ne la uzanto, plenigas `sendinto`/`ricevonto`.
///
/// La procezo sendas al si mem tra sia defaŭlta kapablo (`BOOTSTRAP_TENILO`,
/// fendo `0`). La du identigiloj estas intence falsitaj (`999`); post la sendo
/// la kerno anstataŭigas ilin per la vera sendinto kaj la celo; la neuzata
/// resto de la datumaro devas resti nula.
fn testi_identon(identigilo: u64) {
    let sendata = Mesago::nova(0x4901, 999, 999, b"idento");
    let sendo = biblioteko::sendi(BOOTSTRAP_TENILO, &sendata);
    if sendo != 0 {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] procezo {}: idento (MALBONE): sendo={}\n",
            identigilo, sendo
        ));
        return;
    }
    let mut ricevita = Mesago::malplena();
    let ricevo = biblioteko::ricevi(BOOTSTRAP_TENILO, &mut ricevita);
    let resto_nula = ricevita.datumaro()[sendata.longo()..]
        .iter()
        .all(|bajto| *bajto == 0);
    let bone = ricevo == 0
        && ricevita.tipo == sendata.tipo
        && ricevita.sendinto == identigilo
        && ricevita.ricevonto == identigilo
        && ricevita.enhavo() == sendata.enhavo()
        && resto_nula;
    biblioteko::skribi_formaton(format_args!(
        "[ring3] procezo {}: idento ({}): sendinto={} (uzanto asertis 999), ricevonto={}, resto_nula={}\n",
        identigilo,
        if bone { "bone" } else { "MALBONE" },
        ricevita.sendinto,
        ricevita.ricevonto,
        if resto_nula { "jes" } else { "ne" },
    ));
}

/// B1: veraj teniloj el la kapablotabelo de la procezo.
///
/// Fendo `0` portas la defaŭltan finpunkton, do `KREI_FINPUNKTON` uzas la
/// unuan liberan fendon `1`. Post `FERMI` la fendo reuziĝas kun pli alta
/// generacio; la malnova tenilo restas malvalida (kontraŭ-ABA).
fn testi_finpunktojn(identigilo: u64) {
    let kreo = biblioteko::krei_finpunkton(Permesoj::PLENAJ);
    let tenilo = if kreo > 0 {
        Tenilo::el_nombro(kreo as u32)
    } else {
        None
    };
    let Some(tenilo) = tenilo else {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] procezo {}: finpunktoj (MALBONE): kreo={}\n",
            identigilo, kreo
        ));
        return;
    };
    let fermo = biblioteko::fermi(tenilo);
    let duobla = biblioteko::fermi(tenilo);
    let malvalida = biblioteko::fermi(Tenilo::MALVALIDA);
    // SAFETY: neniuj montriloj; la kerno nur legas la permes-bitojn.
    let nulaj = unsafe { invoki(SYS_KREI_FINPUNKTON, [0; 4]) };
    let nekonataj = unsafe { invoki(SYS_KREI_FINPUNKTON, [8, 0, 0, 0]) };
    let rekrea = biblioteko::krei_finpunkton(Permesoj::SENDI);
    let rekrea_tenilo = if rekrea > 0 {
        Tenilo::el_nombro(rekrea as u32)
    } else {
        None
    };
    // La fendo `1` nun portas la novan tenilon, do la malnova restas rifuzata.
    let malnova = biblioteko::fermi(tenilo);
    let fermo2 = rekrea_tenilo.map_or(-1, biblioteko::fermi);
    let bone = fermo == 0
        && duobla == -9
        && malvalida == -9
        && nulaj == -1
        && nekonataj == -1
        && rekrea > 0
        && malnova == -9
        && fermo2 == 0;
    biblioteko::skribi_formaton(format_args!(
        "[ring3] procezo {}: finpunktoj ({}): tenilo={}, fermo={}, duobla={}, malvalida={}, nulaj_permesoj={}, nekonataj_permesoj={}, rekrea={}, malnova={}\n",
        identigilo,
        if bone { "bone" } else { "MALBONE" },
        tenilo.nombro(),
        fermo,
        duobla,
        malvalida,
        nulaj,
        nekonataj,
        rekrea,
        malnova
    ));
}

/// C10/C11: la permes-bitoj de la kapablo limigas la operacion.
///
/// Finpunkto kreita nur kun `RICEVI` rifuzas `SENDI` per `-1`, kaj finpunkto
/// kun nur `SENDI` rifuzas `RICEVI` per `-1`. La montriloj estas validaj, do la
/// rifuzo venas de la permesa kontrolo, ne de la argumenta kontrolo.
fn testi_rajtojn(identigilo: u64) {
    let riceva = biblioteko::krei_finpunkton(Permesoj::RICEVI);
    let senda = biblioteko::krei_finpunkton(Permesoj::SENDI);
    let (Some(riceva), Some(senda)) = (
        Tenilo::el_nombro(riceva as u32),
        Tenilo::el_nombro(senda as u32),
    ) else {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] procezo {}: rajtoj (MALBONE): kreo={}, {}\n",
            identigilo, riceva, senda
        ));
        return;
    };
    let mesaĝo = Mesago::nova(0x4A01, 0, 0, b"rajtoj");
    let sendo = biblioteko::sendi(riceva, &mesaĝo);
    let mut celo = Mesago::malplena();
    let ricevo = biblioteko::ricevi(senda, &mut celo);
    let fermo_riceva = biblioteko::fermi(riceva);
    let fermo_senda = biblioteko::fermi(senda);
    let bone = sendo == -1 && ricevo == -1 && fermo_riceva == 0 && fermo_senda == 0;
    biblioteko::skribi_formaton(format_args!(
        "[ring3] procezo {}: rajtoj ({}): sendi_per_ricevi={}, ricevi_per_sendi={}\n",
        identigilo,
        if bone { "bone" } else { "MALBONE" },
        sendo,
        ricevo
    ));
}

/// D1/C14: `REVOKI` revokas la tutan finpunkton kaj postulas la biton.
///
/// Finpunkto kun `PLENAJ` revokiĝas; ĉiuj ĝiaj teniloj fariĝas malvalidaj
/// (`-9`), ĉu por sendo, fermo aŭ dua revoko. Finpunkto sen la `REVOKI`-bito
/// rifuzas la vokon per `-1`, kaj ne-nula rezervita `rsi` same per `-1`.
fn testi_revokon(identigilo: u64) {
    let revokebla = biblioteko::krei_finpunkton(Permesoj::PLENAJ);
    let senrevoka = biblioteko::krei_finpunkton(Permesoj::ĈIUJ);
    let (Some(revokebla), Some(senrevoka)) = (
        Tenilo::el_nombro(revokebla as u32),
        Tenilo::el_nombro(senrevoka as u32),
    ) else {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] procezo {}: revoko (MALBONE): kreo={}, {}\n",
            identigilo, revokebla, senrevoka
        ));
        return;
    };
    // SAFETY: neniuj montriloj; `rsi` intence ne estas nula.
    let argumento = unsafe { invoki(SYS_REVOKI, [senrevoka.nombro() as usize, 1, 0, 0]) };
    let rifuzo = biblioteko::revoki(senrevoka);
    let fermo_sen = biblioteko::fermi(senrevoka);
    let revoko = biblioteko::revoki(revokebla);
    let mesaĝo = Mesago::nova(0x4B01, 0, 0, b"revoko");
    let sendo = biblioteko::sendi(revokebla, &mesaĝo);
    let fermo = biblioteko::fermi(revokebla);
    let duobla = biblioteko::revoki(revokebla);
    let bone = argumento == -1
        && rifuzo == -1
        && fermo_sen == 0
        && revoko == 0
        && sendo == -9
        && fermo == -9
        && duobla == -9;
    biblioteko::skribi_formaton(format_args!(
        "[ring3] procezo {}: revoko ({}): argumento={}, sen_revoki={}, fermo_sen={}, revoko={}, sendo={}, fermo={}, duobla={}\n",
        identigilo,
        if bone { "bone" } else { "MALBONE" },
        argumento,
        rifuzo,
        fermo_sen,
        revoko,
        sendo,
        fermo,
        duobla
    ));
}

/// E2: la tempolima politiko de `RICEVI_TEMPO`/`SENDI_TEMPO` el Ring 3.
///
/// `r10 = 0` ne blokas (`-11`), `50 ms` tempolimas post `ceil(50/10) = 5`
/// tikoj (`-110`), `u32::MAX` estas senfina atendo, kaj valoro super `u32::MAX`
/// estas malĝusta argumento (`-1`). Mesaĝo, kiu jam atendas, venkas la
/// tempolimon.
fn testi_tempolimojn(identigilo: u64) {
    let mut celo = Mesago::malplena();
    let nebloka = biblioteko::ricevi_kun_tempolimo(BOOTSTRAP_TENILO, &mut celo, 1, 0);
    let tempolimo = biblioteko::ricevi_kun_tempolimo(BOOTSTRAP_TENILO, &mut celo, 1, 50);
    let mesaĝo = Mesago::nova(0x4C01, 0, 0, b"tempolimo");
    let sendo = biblioteko::sendi_kun_tempolimo(BOOTSTRAP_TENILO, &mesaĝo, 0);
    let mut ricevita = Mesago::malplena();
    let senfina =
        biblioteko::ricevi_kun_tempolimo(BOOTSTRAP_TENILO, &mut ricevita, 0, SENFINA_TEMPOLIMO);
    // SAFETY: la tenilo kaj la celo estas validaj; la tempolimo intence superas
    // `u32::MAX`, do la kerno rifuzas ĝin antaŭ ol legi aŭ skribi la mesaĝon.
    let troa = unsafe {
        invoki(
            SYS_RICEVI_TEMPO,
            [
                BOOTSTRAP_TENILO.nombro() as usize,
                &mut celo as *mut Mesago as usize,
                0,
                u32::MAX as usize + 1,
            ],
        )
    };
    let bone = nebloka == -11
        && tempolimo == -110
        && sendo == 0
        && senfina == 0
        && ricevita.tipo == mesaĝo.tipo
        && troa == -1;
    biblioteko::skribi_formaton(format_args!(
        "[ring3] procezo {}: tempolimoj ({}): nebloka={}, tempolimo={}, sendo={}, senfina={}, troa={}\n",
        identigilo,
        if bone { "bone" } else { "MALBONE" },
        nebloka,
        tempolimo,
        sendo,
        senfina,
        troa
    ));
}

/// Kontrolu, ke la kerno rifuzas malvalidajn Ring 3-argumentojn anstataŭ
/// dereferenci ilin.
fn testi_negativajn() {
    // La kvara provo celas sian propran enirpaĝon, kiu estas RX: la regiona
    // kontrolo trapasas, sed la folia permeso por skribo mankas, do ankaŭ la
    // paĝnivela kontrolo devas rifuzi ĝin.
    // SAFETY: la argumentoj estas intence malvalidaj aŭ malpermesitaj; la kerna
    // dispetĉilo validigas montrilojn, longojn kaj foliajn permesojn antaŭ ol
    // legi aŭ skribi ion ajn.
    let (nekonata, nula, kerna, troa, nurlega) = unsafe {
        (
            invoki(0x7FFF, [0; 4]),
            invoki(SYS_SKRIBI, [0, 8, 0, 0]),
            invoki(SYS_SKRIBI, [0x1000, 8, 0, 0]),
            invoki(SYS_SKRIBI, [SALUTO.as_ptr() as usize, 4097, 0, 0]),
            invoki(SYS_RICEVI, [0, horizonto_start as *const () as usize, 0, 0]),
        )
    };
    if nekonata == -2 && nula == -1 && kerna == -1 && troa == -1 && nurlega == -1 {
        biblioteko::skribi(
            "[ring3] negativaj: nekonata=-2, nula=-1, kerna=-1, troa=-1, nurlega=-1\n",
        );
    } else {
        biblioteko::skribi("[ring3] negativaj: MALBONE\n");
    }

    // Ekde Fazo 3 `rdi` estas kapablo-tenilo, ne fadeno-identigilo. La valoroj
    // `6` (fendo kun ne-kongrua generacio) kaj `0` (ĉiam malvalida) devas esti
    // rifuzataj per `-9`. La montrilo restas valida, do la tenila kontrolo
    // estas tiu, kiu rifuzas (C11, C16).
    let celo = Mesago::malplena();
    let celo_ptr = &celo as *const Mesago as usize;
    // SAFETY: `celo` estas valida staka `Mesago`; la tenilaj valoroj estas
    // intence nevalidaj.
    let (tenilo6, tenilo0) = unsafe {
        (
            invoki(SYS_SENDI, [6, celo_ptr, 0, 0]),
            invoki(SYS_SENDI, [0, celo_ptr, 0, 0]),
        )
    };
    if tenilo6 == -9 && tenilo0 == -9 {
        biblioteko::skribi("[ring3] negativaj (F3): tenilo6=-9, tenilo0=-9\n");
    } else {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] negativaj (F3): MALBONE (tenilo6={}, tenilo0={})\n",
            tenilo6, tenilo0
        ));
    }
}

#[panic_handler]
fn paniko(_informo: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
