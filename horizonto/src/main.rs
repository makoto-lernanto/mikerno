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
use libmikerno::syscall::{SYS_RICEVI, SYS_SKRIBI, invoki};
use libmikerno::{Mesago, Permesoj};

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
/// La procezo sendas al si mem mesaĝon, kies du identigiloj estas intence
/// falsitaj (`999`). Post la sendo la kerno anstataŭigas ilin per la vera
/// sendinto kaj la celo; la neuzata resto de la datumaro devas resti nula.
fn testi_identon(identigilo: u64) {
    let kapablo = libmikerno::FinaPunkto::nova(identigilo, Permesoj::ĈIUJ).kapablo();
    let sendata = Mesago::nova(0x4901, 999, 999, b"idento");
    let sendo = biblioteko::sendi(kapablo, &sendata);
    if sendo != 0 {
        biblioteko::skribi_formaton(format_args!(
            "[ring3] procezo {}: idento (MALBONE): sendo={}\n",
            identigilo, sendo
        ));
        return;
    }
    let mut ricevita = Mesago::malplena();
    let ricevo = biblioteko::ricevi(kapablo, &mut ricevita);
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
}

#[panic_handler]
fn paniko(_informo: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
