use askama::Template;
use serde::Deserialize;
use std::{
    io::{BufReader, Write},
    path::Path,
};

const OPCODES_FILE: &str = "opcodes.yaml";
const CODEGEN_FILE: &str = "opcodes.rs";

#[derive(Deserialize)]
struct Opcode {
    name: String,
    request: RequestResponse,
    response: RequestResponse,
}

#[derive(Deserialize)]
struct RequestResponse {
    opcode: u8,
    #[serde(default)]
    params: Vec<Param>,
}

#[derive(Deserialize)]
struct Param {
    name: String,
    ty: String,
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
