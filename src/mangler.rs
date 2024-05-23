//! Simple Name Mangler
//!
//! This simple mangler uses module name as prefix and underscored with function
//! names.

use crate::ast::Qast;
use crate::error::Result;

pub(crate) fn mangle(ast: &mut Qast) -> Result<()> {
    for module in ast.iter_mut() {
        let mod_name = module.get_name();
        for function in module.iter_mut() {
            function.set_name(format!("{}_{}", mod_name, function.get_name()).into());
        }
    }

    Ok(())
}
