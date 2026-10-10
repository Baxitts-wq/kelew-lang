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
    OpDefineGlobal(usize), // Nom de la variable stocké dans la table des constantes
    OpGetGlobal(usize),
    OpSetGlobal(usize),

    // Contrôle de flux (Sauts / Jumps)
    OpJumpIfFalse(usize),  // Saute à l'offset si le sommet est faux
    OpJump(usize),         // Saut inconditionnel avant
    OpLoop(usize),         // Saut inconditionnel arrière (boucle)

    // Entrées / Sorties
    OpPrint,               // Ejacule (affiche la valeur au sommet)
    OpInput(usize),        // Jib (lit une entrée et l'assigne à la variable)

    // Fin d'exécution
    OpHalt,
}

#[derive(Debug, Clone)]
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
        // Déduplication des constantes pour compacter le bytecode
        for (idx, c) in self.constants.iter().enumerate() {
            if c == &value {
                return idx;
            }
        }
        self.constants.push(value);
        self.constants.len() - 1
    }
}
