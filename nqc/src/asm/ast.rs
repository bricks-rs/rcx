#[derive(Debug, PartialEq, Eq)]
pub enum LabelOrOpcode<'input> {
    Label(&'input str),
    Jmp(&'input str),
    Opcode(&'input str, Vec<u8>),
}
