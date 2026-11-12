//! La komenca protokolo, kiun la kerno transdonas per `iretq` al la uzanta
//! enirpunkto: la reĝimo en `rdi` kaj la proceza identigilo en `rsi`.
//!
//! Ĝi estas parto de la kerna/uzanta protokolo: `horizonto` uzas la reĝimon
//! por elekti la testan scenaron kaj la identigilon por kontroli, ke la
//! sekreta paĝo vere apartenas al ĝi.

/// Kontrolu la propran sekretan paĝon kaj finu normale.
pub const REĜIMO_SEKRETO: u64 = 2;
/// Kontrolu la propran paĝon, poste legu la duan paĝon de la viktimo; tiu
/// legado devas kaŭzi `#PF`.
pub const REĜIMO_SPIONO: u64 = 3;

/// La komuna virtuala adreso de la sekreta paĝo. Ĉiu procezo vidas ĝin je la
/// sama adreso, sed ĝi tradukiĝas al propra fizika kadro.
pub const SEKRETA_ADRESO: u64 = 0x0000_0002_0000_0000;
/// La dua sekreta paĝo, kiun mapas nur la viktima procezo. La spionprocezo
/// konas la aranĝon kaj provas legi ĝin.
pub const SEKRETA_DUA_ADRESO: u64 = SEKRETA_ADRESO + 4096;
/// La marko, kiun la kerno skribas en la unuan okbajton de la sekreta paĝo.
pub const MAGIA_NUMERO: u64 = 0x4D69_4B65_726E_6F21;
/// La deŝovo de la proceza identigilo en la sekreta paĝo.
pub const SEKRETA_ID_DEŜOVO: u64 = 8;
