use askama::Template;
use serde::Deserialize;
use std::{
    io::{BufReader, Write},
    path::Path,
};

const OPCODES_FILE: &str = "opcodes.yaml";
const CODEGEN_FILE: &str = "opcodes.rs";

mod filters {
    pub fn hex(n: &u8) -> askama::Result<String> {
        Ok(format!("0x{n:02x}"))
    }
}

const fn true_() -> bool {
    true
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Opcode {
    name: String,
    description: String,
    #[serde(default)]
    context: Context,
    #[serde(default = "true_")]
    supports_alternate: bool,
    request: RequestResponse,
    response: Option<RequestResponse>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Context {
    #[serde(default)]
    request: bool,
    #[serde(default)]
    response: bool,
    #[serde(default)]
    instruction: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            request: true,
            response: true,
            instruction: true,
        }
    }
}

#[derive(Deserialize)]
struct RequestResponse {
    opcode: u8,
    #[serde(default)]
    params: Vec<Param>,
}

fn u8_() -> String {
    "u8".into()
}

#[derive(Deserialize)]
struct Param {
    name: String,
    #[serde(default = "u8_")]
    ty: String,
    #[serde(default)]
    branch_target: bool,
}

#[derive(Template)]
#[template(path = "opcodes.rs", escape = "none")]
struct OpcodesTemplate {
    opcodes: Vec<Opcode>,
}

fn main() {
    println!("cargo:rerun-if-changed={OPCODES_FILE}");
    println!("cargo:rerun-if-changed=templates/opcodes.rs");

    let opcodes_file = Path::new(env!("CARGO_MANIFEST_DIR")).join(OPCODES_FILE);
    let file = std::fs::File::open(opcodes_file).unwrap();
    let reader = BufReader::new(file);
    let opcodes: Vec<Opcode> = serde_yaml::from_reader(reader).unwrap();
    let templ = OpcodesTemplate { opcodes };

    let codegen = templ.render().unwrap();

    let codegen_file =
        Path::new(&std::env::var_os("OUT_DIR").unwrap()).join(CODEGEN_FILE);
    let mut codegen_file = std::fs::File::create(codegen_file).unwrap();
    codegen_file.write_all(codegen.as_bytes()).unwrap();
}
