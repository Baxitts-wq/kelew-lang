use crate::ast::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    // Manipulation de pile et constantes
    OpConstant(usize),   // Charge une constante depuis la table par index
    OpNil,               // Empile Walou (nil)
    OpTrue,              // Empile Sah (true)
    OpFalse,             // Empile K7hba (false)
    OpPop,               // Dépile la valeur au sommet

    // Opérations arithmétiques
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpModulo,
    OpNegate,

    // Comparaisons et logique
    OpEqual,
    OpNotEqual,
    OpGreater,
    OpGreaterEqual,
    OpLess,
    OpLessEqual,
    OpNot,

    // Variables globales
    OpDefineGlobal(usize),
    OpGetGlobal(usize),
    OpSetGlobal(usize),

    // Contrôle de flux (Sauts / Jumps)
    OpJumpIfFalse(usize),
    OpJump(usize),
    OpLoop(usize),

    // Entrées / Sorties et Appels de fonctions natives
    OpPrint,
    OpInput(usize),
    OpCall(usize, usize), // (name_idx, arg_count)

    // Fin d'exécution
    OpHalt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_op(&mut self, op: OpCode, line: usize) -> usize {
        let offset = self.code.len();
        self.code.push(op);
        self.lines.push(line);
        offset
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        for (idx, c) in self.constants.iter().enumerate() {
            if c == &value {
                return idx;
            }
        }
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// Sérialise le Chunk en format binaire portable
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // 1. Constantes
        let const_count = self.constants.len() as u32;
        buf.extend_from_slice(&const_count.to_le_bytes());

        for c in &self.constants {
            match c {
                Value::Number(n) => {
                    buf.push(0x01);
                    buf.extend_from_slice(&n.to_le_bytes());
                }
                Value::String(s) => {
                    buf.push(0x02);
                    let bytes = s.as_bytes();
                    let len = bytes.len() as u32;
                    buf.extend_from_slice(&len.to_le_bytes());
                    buf.extend_from_slice(bytes);
                }
                Value::Bool(b) => {
                    buf.push(0x03);
                    buf.push(if *b { 1 } else { 0 });
                }
                Value::Nil => {
                    buf.push(0x04);
                }
            }
        }

        // 2. Instructions (Opcodes)
        let code_count = self.code.len() as u32;
        buf.extend_from_slice(&code_count.to_le_bytes());

        for (i, op) in self.code.iter().enumerate() {
            let line = (self.lines.get(i).copied().unwrap_or(0)) as u32;
            buf.extend_from_slice(&line.to_le_bytes());

            match op {
                OpCode::OpConstant(idx) => {
                    buf.push(1);
                    buf.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpNil => buf.push(2),
                OpCode::OpTrue => buf.push(3),
                OpCode::OpFalse => buf.push(4),
                OpCode::OpPop => buf.push(5),
                OpCode::OpAdd => buf.push(6),
                OpCode::OpSubtract => buf.push(7),
                OpCode::OpMultiply => buf.push(8),
                OpCode::OpDivide => buf.push(9),
                OpCode::OpModulo => buf.push(10),
                OpCode::OpNegate => buf.push(11),
                OpCode::OpEqual => buf.push(12),
                OpCode::OpNotEqual => buf.push(13),
                OpCode::OpGreater => buf.push(14),
                OpCode::OpGreaterEqual => buf.push(15),
                OpCode::OpLess => buf.push(16),
                OpCode::OpLessEqual => buf.push(17),
                OpCode::OpNot => buf.push(18),
                OpCode::OpDefineGlobal(idx) => {
                    buf.push(19);
                    buf.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpGetGlobal(idx) => {
                    buf.push(20);
                    buf.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpSetGlobal(idx) => {
                    buf.push(21);
                    buf.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpJumpIfFalse(target) => {
                    buf.push(22);
                    buf.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                OpCode::OpJump(target) => {
                    buf.push(23);
                    buf.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                OpCode::OpLoop(target) => {
                    buf.push(24);
                    buf.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                OpCode::OpPrint => buf.push(25),
                OpCode::OpInput(idx) => {
                    buf.push(26);
                    buf.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpCall(name_idx, argc) => {
                    buf.push(27);
                    buf.extend_from_slice(&(*name_idx as u32).to_le_bytes());
                    buf.extend_from_slice(&(*argc as u32).to_le_bytes());
                }
                OpCode::OpHalt => buf.push(28),
            }
        }

        buf
    }

    /// Désérialise le format binaire en Chunk prêt à l'exécution
    pub fn from_bytes(bytes: &[u8]) -> Result<Chunk, String> {
        let mut cur = 0;

        // 1. Lire les constantes
        if bytes.len() < 4 {
            return Err("Format de bytecode tronqué.".to_string());
        }
        let const_count = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
        cur += 4;

        let mut constants = Vec::with_capacity(const_count);
        for _ in 0..const_count {
            if cur >= bytes.len() {
                return Err("Format de bytecode invalide (constantes).".to_string());
            }
            let tag = bytes[cur];
            cur += 1;
            match tag {
                0x01 => {
                    if cur + 8 > bytes.len() { return Err("Erreur float".to_string()); }
                    let n = f64::from_le_bytes(bytes[cur..cur + 8].try_into().unwrap());
                    cur += 8;
                    constants.push(Value::Number(n));
                }
                0x02 => {
                    if cur + 4 > bytes.len() { return Err("Erreur string len".to_string()); }
                    let len = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    if cur + len > bytes.len() { return Err("Erreur string data".to_string()); }
                    let s = String::from_utf8_lossy(&bytes[cur..cur + len]).to_string();
                    cur += len;
                    constants.push(Value::String(s));
                }
                0x03 => {
                    if cur >= bytes.len() { return Err("Erreur bool".to_string()); }
                    let b = bytes[cur] != 0;
                    cur += 1;
                    constants.push(Value::Bool(b));
                }
                0x04 => {
                    constants.push(Value::Nil);
                }
                _ => return Err(format!("Tag de constante inconnu: 0x{:02X}", tag)),
            }
        }

        // 2. Lire le code
        if cur + 4 > bytes.len() {
            return Err("Format de bytecode tronqué (code count).".to_string());
        }
        let code_count = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
        cur += 4;

        let mut code = Vec::with_capacity(code_count);
        let mut lines = Vec::with_capacity(code_count);

        for _ in 0..code_count {
            if cur + 5 > bytes.len() {
                return Err("Format de bytecode tronqué (opcode).".to_string());
            }
            let line = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
            cur += 4;
            lines.push(line);

            let op_tag = bytes[cur];
            cur += 1;

            let op = match op_tag {
                1 => {
                    let idx = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpConstant(idx)
                }
                2 => OpCode::OpNil,
                3 => OpCode::OpTrue,
                4 => OpCode::OpFalse,
                5 => OpCode::OpPop,
                6 => OpCode::OpAdd,
                7 => OpCode::OpSubtract,
                8 => OpCode::OpMultiply,
                9 => OpCode::OpDivide,
                10 => OpCode::OpModulo,
                11 => OpCode::OpNegate,
                12 => OpCode::OpEqual,
                13 => OpCode::OpNotEqual,
                14 => OpCode::OpGreater,
                15 => OpCode::OpGreaterEqual,
                16 => OpCode::OpLess,
                17 => OpCode::OpLessEqual,
                18 => OpCode::OpNot,
                19 => {
                    let idx = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpDefineGlobal(idx)
                }
                20 => {
                    let idx = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpGetGlobal(idx)
                }
                21 => {
                    let idx = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpSetGlobal(idx)
                }
                22 => {
                    let target = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpJumpIfFalse(target)
                }
                23 => {
                    let target = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpJump(target)
                }
                24 => {
                    let target = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpLoop(target)
                }
                25 => OpCode::OpPrint,
                26 => {
                    let idx = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpInput(idx)
                }
                27 => {
                    let name_idx = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    let argc = u32::from_le_bytes(bytes[cur..cur + 4].try_into().unwrap()) as usize;
                    cur += 4;
                    OpCode::OpCall(name_idx, argc)
                }
                28 => OpCode::OpHalt,
                _ => return Err(format!("Opcode inconnu: {}", op_tag)),
            };

            code.push(op);
        }

        Ok(Chunk { code, constants, lines })
    }
}
