pub mod qasm;
use crate::error::Result;

/// A translator trait can be implemented by IRs to provide a translation
/// codegen to go from one IR to another. In this codebase, the compiler deals
/// with the following two IRs:
///     Quale IR   --------->   OpenQASM
/// translating from the higher-source to quantum assembly.
pub(crate) trait Translator<T>: Sized {
    fn translate(ir: T) -> Result<Self>;
}
