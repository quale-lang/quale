//! Parser for quale language.
//! It translates the given code into an AST.
use crate::ast::QAST;
use crate::error::Result;

pub(crate) fn parse_src(src: &String) -> Result<QAST> {
    let lines = std::fs::read(src)?;

    for byte in lines {
        print!("{} ", std::ascii::escape_default(byte));
    }
    print!("\n");
    return Ok(QAST {});
}
