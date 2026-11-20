//! Unuprocesora kunhavigita stato: aliro estas sekurigita per malŝaltitaj
//! interrompoj, ne per ŝloso.

use core::cell::UnsafeCell;

pub struct Kunhavigita<T>(UnsafeCell<T>);

unsafe impl<T> Sync for Kunhavigita<T> {}

impl<T> Kunhavigita<T> {
    pub const fn nova(valoro: T) -> Self {
        Self(UnsafeCell::new(valoro))
    }

    /// Kruda montrilo al la valoro.
    ///
    /// # Sekureco
    /// La alvokanto devas garantii, ke neniu alia aliro al la sama valoro
    /// okazas samtempe — en la nukleo tio signifas malŝaltitajn interrompojn
    /// dum la tuta uzo de la referenco. La `&self`-subskribo ne esprimas tiun
    /// disciplinon, do la derivo de referenco el la montrilo restas eksplicita
    /// `unsafe`-ago de la alvokanto.
    pub fn akiri(&self) -> *mut T {
        self.0.get()
    }
}
