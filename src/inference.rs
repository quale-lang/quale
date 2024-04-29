//! Type inference mechanism for qcc.
use crate::ast::Qast;
use crate::error::{QccErrorKind, Result};
use crate::types::Type;

pub(crate) fn infer(ast: &mut Qast) -> Result<()> {
    for module in ast.iter_mut() {
        for function in module.iter_mut() {
            for each in function.get_input_type() {
                if *each == Type::Bottom {
                    println!("{}", function.get_name());
                }
            }

            if *function.get_output_type() == Type::Bottom && function.get_name() == "main" {
                // println!("{function}");
            }
        }
    }

    Ok(())
}
