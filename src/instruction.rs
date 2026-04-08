use crate::object::NyaPrimitiveType;

pub enum Instruction {
    PushInt(i64),
    PushFloat(f64),
    Pop,
    Add,
    Jump(usize),
    JumpIf(usize),
    SetGlobal,
    GetGlobal,
    Print,
    Equal,
    Not,
    CollectGarbage,
    Halt,
}
