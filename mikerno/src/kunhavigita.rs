//! Unuprocesora kunhavigita stato: aliro estas sekurigita per malŝaltitaj
//! interrompoj, ne per ŝloso.

use core::cell::UnsafeCell;

pub struct Kunhavigita<T>(UnsafeCell<T>);

unsafe impl<T> Sync for Kunhavigita<T> {}

impl<T> Kunhavigita<T> {
    pub const fn nova(valoro: T) -> Self {
        Self(UnsafeCell::new(valoro))
    }

    #[allow(clippy::mut_from_ref)]
    pub fn akiri(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}
