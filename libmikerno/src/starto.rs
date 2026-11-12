//! La komenca reĝimo, kiun la kerno transdonas en `rdi` al la uzanta
//! enirpunkto.
//!
//! Ĝi estas parto de la kerna/uzanta protokolo: `horizonto` uzas ĝin por
//! elekti la testan scenaron.

/// Normala plenumo: la bildo finiĝas per `SYS_FINI`.
pub const REĜIMO_FINI: u64 = 0;
/// Intenca legado de nemapita adreso, por pruvi la kapton de `#PF`.
pub const REĜIMO_KRUI: u64 = 1;
