//! Type inference mechanism for qcc.
use crate::ast::Qast;
use crate::error::{QccErrorKind, Result};

pub(crate) fn infer(ast: &mut Qast) -> Result<()> {

    for module in ast.iter_modules() {
        for function in module.iter() {
            for param in function.iter_params() {
            }
        }
    }

    Ok(())
}
