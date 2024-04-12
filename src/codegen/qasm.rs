//! OpenQASM Codegen Backend
use crate::ast::{FunctionAST, Ident, Qast};
use crate::attributes::Attribute;
use crate::codegen::Translator;
use crate::error::Result;
use std::fmt;

use std::io::Write;

pub(crate) enum QasmVersion {
    V2_0,
}

impl From<&str> for QasmVersion {
    fn from(value: &str) -> Self {
        use QasmVersion::*;
        match value {
            "2.0" => V2_0,
            _ => panic!("Qasm: Unexpected version number"),
        }
    }
}

impl fmt::Display for QasmVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QasmVersion::*;
        match self {
            V2_0 => write!(f, "2.0"),
        }
    }
}

/// An OpenQASM module.
/// NOTE: Does the Sea of Nodes IR work here? Because we only have to worry
/// about `barrier` and `measure` operations. So, ideally control-flow
/// in-between should not bother us.
pub(crate) struct QasmModule {
    version: QasmVersion,
    includes: Vec<QasmInclude>,
    gates: Vec<QasmGate>,
}

impl QasmModule {
    pub(crate) fn new(version: &str) -> Self {
        Self {
            version: version.into(),
            includes: vec![],
            gates: vec![],
        }
    }

    /// It outputs the translated `QasmModule` to a file at `path`.
    pub(crate) fn generate(&self, path: &str) -> Result<()> {
        let mut asm_path = std::fs::File::create(path)?;
        asm_path.write(self.to_string().as_bytes())?;
        Ok(())
    }
}

impl Translator<Qast> for QasmModule {
    /// Translator for qasm codegen.
    /// It takes a `Qast` object and translates it recursively into a
    /// `QasmModule`.
    fn translate(ast: Qast) -> Result<Self> {
        let mut gates: Vec<QasmGate> = vec![];
        for f in ast.iter() {
            let attrs = f.get_attrs();
            if !attrs.is_empty() && attrs.0.contains(&Attribute::NonDeter) {
                gates.push(f.into());
            }
        }
        Ok(gates.into())
    }
}

impl From<Vec<QasmGate>> for QasmModule {
    fn from(gates: Vec<QasmGate>) -> Self {
        Self {
            version: QasmVersion::V2_0,
            includes: vec![],
            gates,
        }
    }
}

impl Default for QasmModule {
    fn default() -> Self {
        Self {
            version: QasmVersion::V2_0,
            includes: vec![QasmInclude(
                "/home/manas/workspace/quale/openqasm-examples/qelib1.inc",
            )],
            gates: vec![QasmGate::new(
                "def",
                &["lambda", "theta"],
                vec![Qreg::new("a", 8), Qreg::new("b", 8)],
            )],
        }
    }
}

impl fmt::Display for QasmModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "OPENQASM {};", self.version)?;

        for include in &self.includes {
            writeln!(f, "{}", include)?;
        }

        for gate in &self.gates {
            write!(f, "{}", gate)?;
        }
        Ok(())
    }
}

/// A qubit representation.
/// It is a linear combination of |0〉 and |1〉.
#[derive(Debug, Clone)]
// I know about no-cloning property of qubits, and this is quite ironic, but hey
// it is just a classical world representation of qubit, not an actual qubit.
pub(crate) struct Qubit([f32; 2]);

impl Qubit {
    pub(crate) fn new(bit: Cbit) -> Self {
        if bit {
            Qubit::one()
        } else {
            Qubit::zero()
        }
    }

    pub(crate) fn zero() -> Self {
        Self([1.0, 0.0])
    }

    pub(crate) fn one() -> Self {
        Self([0.0, 1.0])
    }
}

/// A quantum register representation.
pub(crate) struct Qreg {
    name: Ident,
    len: QregSize,
    // qubits: Vec<Qubit>,
}

type QregSize = usize;

impl Qreg {
    pub(crate) fn new<T>(name: T, len: QregSize) -> Self
    where
        String: From<T>,
    {
        Self {
            name: name.into(),
            len,
            // qubits: vec![Qubit::zero(); len],
        }
    }

    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }

    pub(crate) fn len(&self) -> QregSize {
        self.len
    }

    pub(crate) fn get_type(&self) -> String {
        "q".to_string() + &self.len.to_string()
    }
}

impl fmt::Display for Qreg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "qreg {}[{}]", self.name, self.len)
    }
}

pub(crate) struct QregDef {
    info: Qreg,
    qubits: Vec<Qubit>,
}

impl QregDef {
    pub(crate) fn new<T>(name: T, len: QregSize) -> Self
    where
        String: From<T>,
    {
        Self {
            info: Qreg::new(name, len),
            qubits: vec![Qubit::zero(); len],
        }
    }
}

/// A classical bit representation.
type Cbit = bool;

/// A classical register representation.
pub(crate) struct Creg {
    name: Ident,
    len: CregSize,
    bits: Vec<Cbit>,
}

type CregSize = usize;

impl Creg {
    pub(crate) fn new<T>(name: T, len: CregSize) -> Self
    where
        String: From<T>,
    {
        Self {
            name: name.into(),
            len,
            bits: vec![false; len],
        }
    }

    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }

    pub(crate) fn len(&self) -> QregSize {
        self.len
    }
}

/// A qasm gate is a simple function-like structure.
/// // comment
/// gate name(params) qargs
/// {
///   body
/// }
pub(crate) struct QasmGate {
    name: Ident,
    params: Vec<Ident>,
    qargs: Vec<Qreg>,
}

impl QasmGate {
    pub(crate) fn new(name: &str, params: &[&str], qargs: Vec<Qreg>) -> Self {
        Self {
            name: name.into(),
            params: params.to_vec().iter().map(|p| p.to_string()).collect(),
            qargs,
        }
    }
}

impl From<&FunctionAST> for QasmGate {
    fn from(f: &FunctionAST) -> Self {
        Self {
            name: f.get_name().clone(),
            params: vec![],
            qargs: vec![],
        }
    }
}

impl fmt::Display for QasmGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let qargs_s: String = self
            .qargs
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        if self.params.len() > 0 {
            let params_s: String = self
                .params
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            write!(
                f,
                "
gate {}({}) {}
{{
    // body: feature to be implemented
}}
",
                self.name, params_s, qargs_s
            )
        } else {
            write!(
                f,
                "
gate {} {}
{{
    // body: feature to be implemented
}}
",
                self.name, qargs_s
            )
        }
    }
}

pub(crate) struct QasmInclude(&'static str);

impl fmt::Display for QasmInclude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "include \"{}\";", self.0)
    }
}

pub(crate) struct QasmComments<'a>(Vec<&'a str>);

impl<'a> fmt::Display for QasmComments<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.0 {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_qasm_backend() {
        let q = Qreg::new("a", 8);
        assert!(q.get_type() == "q8");
        // gate cu1(lambda) a,b
        // {
        //   U(0,0,theta/2) a;
        //   CX a,b;
        //   U(0,0,-theta/2) b;
        //   CX a,b;
        //   U(0,0,theta/2) b;
        // }
        // cu1(pi/2) q[0],q[1];
        let qreg_a = Qreg::new("a", 8);
        let qreg_b = Qreg::new("b", 8);
        let qgate = QasmGate::new("cu1", &["lambda", "theta"], vec![qreg_a, qreg_b]);
        println!("{qgate}");

        // let qmod: QasmModule = Default::default();
        // println!("{qmod}");
    }

    use crate::error::Result;
    use crate::parser::Parser;

    #[test]
    fn check_qasm_translate() -> Result<()> {
        let parser = Parser::new(vec!["tests/test1.ql"])?.unwrap();
        let config = parser.get_config();
        let ast = parser.parse(&config.analyzer.src)?;
        let ir = QasmModule::translate(ast)?;
        println!("{ir}");

        Ok(())
    }
}
