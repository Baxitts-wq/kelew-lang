use crate::opcode::{Chunk, OpCode};

pub struct Disassembler<'a> {
    chunk: &'a Chunk,
    name: &'a str,
}

impl<'a> Disassembler<'a> {
    pub fn new(chunk: &'a Chunk, name: &'a str) -> Self {
        Disassembler { chunk, name }
    }

    pub fn disassemble(&self) {
        println!("┌──────────────────────────────────────────────────────────────┐");
        println!("│  KELEW BYTECODE DISASSEMBLY : {:<30} │", self.name);
        println!("└──────────────────────────────────────────────────────────────┘");
        println!("{:<6} {:<6} {:<18} {:<24}", "OFFSET", "LIGNE", "OPCODE", "OPERANDES / DETAILS");
        println!("{:-<6} {:-<6} {:-<18} {:-<24}", "", "", "", "");

        let mut offset = 0;
        while offset < self.chunk.code.len() {
            offset = self.disassemble_instruction(offset);
        }
        println!();
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        let line = self.chunk.lines[offset];
        let line_str = if offset > 0 && line == self.chunk.lines[offset - 1] {
            "  |".to_string()
        } else {
            format!("{:>3}", line)
        };

        print!("{:04X}   {:<6} ", offset, line_str);

        let op = &self.chunk.code[offset];
        match op {
            OpCode::OpConstant(idx) => {
                let val = &self.chunk.constants[*idx];
                println!("{:<18} #{:<4} ({})", "OP_CONSTANT", idx, val);
            }
            OpCode::OpNil => println!("{:<18}", "OP_NIL"),
            OpCode::OpTrue => println!("{:<18} (Sah)", "OP_TRUE"),
            OpCode::OpFalse => println!("{:<18} (K7hba)", "OP_FALSE"),
            OpCode::OpPop => println!("{:<18}", "OP_POP"),
            OpCode::OpAdd => println!("{:<18} (+)", "OP_ADD"),
            OpCode::OpSubtract => println!("{:<18} (-)", "OP_SUBTRACT"),
            OpCode::OpMultiply => println!("{:<18} (*)", "OP_MULTIPLY"),
            OpCode::OpDivide => println!("{:<18} (/)", "OP_DIVIDE"),
            OpCode::OpModulo => println!("{:<18} (%)", "OP_MODULO"),
            OpCode::OpNegate => println!("{:<18} (- unitaire)", "OP_NEGATE"),
            OpCode::OpEqual => println!("{:<18} (==)", "OP_EQUAL"),
            OpCode::OpNotEqual => println!("{:<18} (!=)", "OP_NOT_EQUAL"),
            OpCode::OpGreater => println!("{:<18} (>)", "OP_GREATER"),
            OpCode::OpGreaterEqual => println!("{:<18} (>=)", "OP_GREATER_EQUAL"),
            OpCode::OpLess => println!("{:<18} (<)", "OP_LESS"),
            OpCode::OpLessEqual => println!("{:<18} (<=)", "OP_LESS_EQUAL"),
            OpCode::OpNot => println!("{:<18} (!)", "OP_NOT"),
            OpCode::OpDefineGlobal(idx) => {
                let name = &self.chunk.constants[*idx];
                println!("{:<18} #{:<4} ({})", "OP_DEFINE_GLOBAL", idx, name);
            }
            OpCode::OpGetGlobal(idx) => {
                let name = &self.chunk.constants[*idx];
                println!("{:<18} #{:<4} ({})", "OP_GET_GLOBAL", idx, name);
            }
            OpCode::OpSetGlobal(idx) => {
                let name = &self.chunk.constants[*idx];
                println!("{:<18} #{:<4} ({})", "OP_SET_GLOBAL", idx, name);
            }
            OpCode::OpJumpIfFalse(target) => {
                println!("{:<18} -> {:04X}", "OP_JUMP_IF_FALSE", target);
            }
            OpCode::OpJump(target) => {
                println!("{:<18} -> {:04X}", "OP_JUMP", target);
            }
            OpCode::OpLoop(target) => {
                println!("{:<18} -> {:04X} (retour)", "OP_LOOP", target);
            }
            OpCode::OpPrint => println!("{:<18} (Ejacule)", "OP_PRINT"),
            OpCode::OpInput(idx) => {
                let name = &self.chunk.constants[*idx];
                println!("{:<18} #{:<4} (Jib -> {})", "OP_INPUT", idx, name);
            }
            OpCode::OpCall(name_idx, argc) => {
                let name = &self.chunk.constants[*name_idx];
                println!("{:<18} #{:<4} ({} avec {} args)", "OP_CALL", name_idx, name, argc);
            }
            OpCode::OpHalt => println!("{:<18} (Baraka/FioDelPuta)", "OP_HALT"),
        }

        offset + 1
    }
}
