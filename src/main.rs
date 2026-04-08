use crate::{instruction::Instruction, state::NyaState};

pub mod instruction;
pub mod object;
pub mod state;

fn main() {
    let mut ns = NyaState::new();
    let program_basic: Vec<Instruction> = vec![
        Instruction::PushInt(2),
        Instruction::PushInt(3),
        Instruction::Add,
        Instruction::Print,
        Instruction::Halt,
    ];
    println!("basic program(adds 2 and 3)");
    ns.run_instructions(&program_basic);
    let program1: Vec<Instruction> = vec![
        Instruction::PushInt(69),
        Instruction::Jump(3),
        Instruction::PushInt(2),
        Instruction::PushInt(3),
        Instruction::Add,
        Instruction::Print,
        Instruction::Halt,
    ];
    println!("basic program with jump(skips the addition and just prints)");
    ns.run_instructions(&program1);

    let program2: Vec<Instruction> = vec![
        Instruction::PushInt(2),
        Instruction::PushInt(3),
        Instruction::Add,
        Instruction::PushInt(5),
        Instruction::Equal,
        Instruction::JumpIf(1),
        Instruction::PushFloat(420.69),
        Instruction::PushInt(8008),
        Instruction::Print,
        Instruction::Halt,
    ];
    println!("basic program with jump and equality(skips adding float to stack)");
    ns.run_instructions(&program2);
}
