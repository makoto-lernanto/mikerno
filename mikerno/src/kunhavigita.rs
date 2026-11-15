//! Unuprocesora kunhavigita stato: aliro estas sekurigita per malŝaltitaj
//! interrompoj, ne per ŝloso.

use core::cell::UnsafeCell;

pub struct Kunhavigita<T>(UnsafeCell<T>);

unsafe impl<T> Sync for Kunhavigita<T> {}

impl<T> Kunhavigita<T> {
    pub const fn nova(valoro: T) -> Self {
        Self(UnsafeCell::new(valoro))
    }

    /// Aliru la valoron per mutable reference.
    ///
    /// # Sekureco
    /// La alvokanto devas garantii, ke neniu alia aliro al la sama valoro
    /// okazas samtempe — en la nukleo tio signifas malŝaltitajn interrompojn
    /// dum la tuta uzo de la referenco. La `&self`-subskribo ne esprimas tiun
    /// disciplinon, do ĝi restas respondeco de la alvokanto.
    #[allow(clippy::mut_from_ref)]
    pub fn akiri(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}
