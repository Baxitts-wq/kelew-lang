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

    OpReturn,
    OpBuildList(usize),
    OpBuildMap(usize),
    OpGetIndex,
    OpSetIndex,
    // Fin d'exécution
    OpHalt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
    pub file_ids: Vec<usize>,
    pub files: Vec<String>,
}

pub const BYTECODE_MAGIC: &[u8; 8] = b"KLW_BC01";
pub const BYTECODE_VERSION: u8 = 1;

/// Calcul de somme de contrôle Adler-32 (100% natif, zéro dépendance externe)
pub fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    const MOD_ADLER: u32 = 65521;

    for &byte in data {
        a = (a + byte as u32) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }

    (b << 16) | a
}

#[inline]
fn read_u8(buf: &[u8], cur: &mut usize) -> Result<u8, String> {
    if *cur >= buf.len() {
        return Err("Fin inattendue des données (octet manquant)".to_string());
    }
    let val = buf[*cur];
    *cur += 1;
    Ok(val)
}

#[inline]
fn read_u32(buf: &[u8], cur: &mut usize) -> Result<u32, String> {
    if *cur + 4 > buf.len() {
        return Err("Fin inattendue des données (u32 manquant)".to_string());
    }
    let val = u32::from_le_bytes([buf[*cur], buf[*cur + 1], buf[*cur + 2], buf[*cur + 3]]);
    *cur += 4;
    Ok(val)
}

#[inline]
fn read_f64(buf: &[u8], cur: &mut usize) -> Result<f64, String> {
    if *cur + 8 > buf.len() {
        return Err("Fin inattendue des données (f64 manquant)".to_string());
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&buf[*cur..*cur + 8]);
    *cur += 8;
    Ok(f64::from_le_bytes(bytes))
}

#[inline]
fn read_string(buf: &[u8], cur: &mut usize) -> Result<String, String> {
    let len = read_u32(buf, cur)? as usize;
    if *cur + len > buf.len() {
        return Err(format!("Fin inattendue des données (chaîne de longueur {} tronquée)", len));
    }
    let s = String::from_utf8_lossy(&buf[*cur..*cur + len]).to_string();
    *cur += len;
    Ok(s)
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
            file_ids: Vec::new(),
            files: vec!["<source>".to_string()],
        }
    }

    pub fn add_file(&mut self, filename: &str) -> usize {
        for (idx, f) in self.files.iter().enumerate() {
            if f == filename {
                return idx;
            }
        }
        self.files.push(filename.to_string());
        self.files.len() - 1
    }

    pub fn write_op(&mut self, op: OpCode, line: usize) -> usize {
        self.write_op_loc(op, 0, line)
    }

    pub fn write_op_loc(&mut self, op: OpCode, file_id: usize, line: usize) -> usize {
        let offset = self.code.len();
        self.code.push(op);
        self.lines.push(line);
        self.file_ids.push(file_id);
        offset
    }

    pub fn get_location(&self, offset: usize) -> (&str, usize) {
        let line = self.lines.get(offset).copied().unwrap_or(0);
        let file_id = self.file_ids.get(offset).copied().unwrap_or(0);
        let file = self.files.get(file_id).map(|s| s.as_str()).unwrap_or("<source>");
        (file, line)
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

    /// Sérialise le Chunk en format binaire portable avec magic header KLW_BC01 et checksum Adler-32
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut payload = Vec::new();

        // 1. Constantes
        let const_count = self.constants.len() as u32;
        payload.extend_from_slice(&const_count.to_le_bytes());

        for c in &self.constants {
            match c {
                Value::Number(n) => {
                    payload.push(0x01);
                    payload.extend_from_slice(&n.to_le_bytes());
                }
                Value::String(s) => {
                    payload.push(0x02);
                    let bytes = s.as_bytes();
                    let len = bytes.len() as u32;
                    payload.extend_from_slice(&len.to_le_bytes());
                    payload.extend_from_slice(bytes);
                }
                Value::Bool(b) => {
                    payload.push(0x03);
                    payload.push(if *b { 1 } else { 0 });
                }
                Value::Nil => {
                    payload.push(0x04);
                }
                Value::Function(f_val) => {
                    payload.push(0x05);
                    let name_bytes = f_val.name.as_bytes();
                    payload.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
                    payload.extend_from_slice(name_bytes);

                    payload.extend_from_slice(&(f_val.params.len() as u32).to_le_bytes());
                    for p in &f_val.params {
                        let p_bytes = p.as_bytes();
                        payload.extend_from_slice(&(p_bytes.len() as u32).to_le_bytes());
                        payload.extend_from_slice(p_bytes);
                    }

                    let chunk_bytes = f_val.chunk.to_bytes();
                    payload.extend_from_slice(&(chunk_bytes.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&chunk_bytes);
                }
                Value::List(l) => {
                    payload.push(0x06);
                    let borrowed = l.borrow();
                    let count = borrowed.len() as u32;
                    payload.extend_from_slice(&count.to_le_bytes());
                    let mut sub_chunk = Chunk::new();
                    for elem in borrowed.iter() {
                        sub_chunk.add_constant(elem.clone());
                    }
                    let sub_bytes = sub_chunk.to_bytes();
                    payload.extend_from_slice(&(sub_bytes.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&sub_bytes);
                }
                Value::Map(m) => {
                    payload.push(0x07);
                    let borrowed = m.borrow();
                    let count = borrowed.len() as u32;
                    payload.extend_from_slice(&count.to_le_bytes());
                    let mut sub_chunk = Chunk::new();
                    for (k, v) in borrowed.iter() {
                        sub_chunk.add_constant(Value::String(k.clone()));
                        sub_chunk.add_constant(v.clone());
                    }
                    let sub_bytes = sub_chunk.to_bytes();
                    payload.extend_from_slice(&(sub_bytes.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&sub_bytes);
                }
            }
        }

        // 2. Fichiers sources
        let file_count = self.files.len() as u32;
        payload.extend_from_slice(&file_count.to_le_bytes());
        for f in &self.files {
            let bytes = f.as_bytes();
            payload.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            payload.extend_from_slice(bytes);
        }

        // 3. Instructions (Opcodes)
        let code_count = self.code.len() as u32;
        payload.extend_from_slice(&code_count.to_le_bytes());

        for (i, op) in self.code.iter().enumerate() {
            let line = (self.lines.get(i).copied().unwrap_or(0)) as u32;
            payload.extend_from_slice(&line.to_le_bytes());

            let file_id = (self.file_ids.get(i).copied().unwrap_or(0)) as u32;
            payload.extend_from_slice(&file_id.to_le_bytes());

            match op {
                OpCode::OpConstant(idx) => {
                    payload.push(1);
                    payload.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpNil => payload.push(2),
                OpCode::OpTrue => payload.push(3),
                OpCode::OpFalse => payload.push(4),
                OpCode::OpPop => payload.push(5),
                OpCode::OpAdd => payload.push(6),
                OpCode::OpSubtract => payload.push(7),
                OpCode::OpMultiply => payload.push(8),
                OpCode::OpDivide => payload.push(9),
                OpCode::OpModulo => payload.push(10),
                OpCode::OpNegate => payload.push(11),
                OpCode::OpEqual => payload.push(12),
                OpCode::OpNotEqual => payload.push(13),
                OpCode::OpGreater => payload.push(14),
                OpCode::OpGreaterEqual => payload.push(15),
                OpCode::OpLess => payload.push(16),
                OpCode::OpLessEqual => payload.push(17),
                OpCode::OpNot => payload.push(18),
                OpCode::OpDefineGlobal(idx) => {
                    payload.push(19);
                    payload.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpGetGlobal(idx) => {
                    payload.push(20);
                    payload.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpSetGlobal(idx) => {
                    payload.push(21);
                    payload.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpJumpIfFalse(target) => {
                    payload.push(22);
                    payload.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                OpCode::OpJump(target) => {
                    payload.push(23);
                    payload.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                OpCode::OpLoop(target) => {
                    payload.push(24);
                    payload.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                OpCode::OpPrint => payload.push(25),
                OpCode::OpInput(idx) => {
                    payload.push(26);
                    payload.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                OpCode::OpCall(name_idx, argc) => {
                    payload.push(27);
                    payload.extend_from_slice(&(*name_idx as u32).to_le_bytes());
                    payload.extend_from_slice(&(*argc as u32).to_le_bytes());
                }
                OpCode::OpHalt => payload.push(28),
                OpCode::OpReturn => payload.push(29),
                OpCode::OpBuildList(count) => {
                    payload.push(30);
                    payload.extend_from_slice(&(*count as u32).to_le_bytes());
                }
                OpCode::OpBuildMap(count) => {
                    payload.push(31);
                    payload.extend_from_slice(&(*count as u32).to_le_bytes());
                }
                OpCode::OpGetIndex => payload.push(32),
                OpCode::OpSetIndex => payload.push(33),
            }
        }

        // Assemblage final : [MAGIC: 8 octets] [CHECKSUM: 4 octets] [PAYLOAD...]
        let checksum = adler32(&payload);
        let mut container = Vec::with_capacity(12 + payload.len());
        container.extend_from_slice(BYTECODE_MAGIC);
        container.extend_from_slice(&checksum.to_le_bytes());
        container.extend_from_slice(&payload);
        container
    }

    /// Désérialise le format binaire en Chunk prêt à l'exécution avec vérification stricte
    pub fn from_bytes(bytes: &[u8]) -> Result<Chunk, String> {
        if bytes.len() < 12 {
            return Err("Format de bytecode tronqué: en-tête manquant (taille < 12 octets)".to_string());
        }

        let magic = &bytes[0..8];
        if magic != BYTECODE_MAGIC {
            return Err(format!(
                "En-tête de bytecode invalide: attendu {:?}, reçu {:?}",
                BYTECODE_MAGIC, magic
            ));
        }

        let expected_checksum = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let payload = &bytes[12..];
        let computed_checksum = adler32(payload);

        if computed_checksum != expected_checksum {
            return Err(format!(
                "Échec d'intégrité du bytecode (Adler-32): attendu 0x{:08X}, calculé 0x{:08X}",
                expected_checksum, computed_checksum
            ));
        }

        let mut cur = 0;

        // 1. Lire les constantes
        let const_count = read_u32(payload, &mut cur)? as usize;
        let mut constants = Vec::with_capacity(const_count);
        for _ in 0..const_count {
            let tag = read_u8(payload, &mut cur)?;
            match tag {
                0x01 => {
                    let n = read_f64(payload, &mut cur)?;
                    constants.push(Value::Number(n));
                }
                0x02 => {
                    let s = read_string(payload, &mut cur)?;
                    constants.push(Value::String(s));
                }
                0x03 => {
                    let b = read_u8(payload, &mut cur)? != 0;
                    constants.push(Value::Bool(b));
                }
                0x04 => {
                    constants.push(Value::Nil);
                }
                0x05 => {
                    let name = read_string(payload, &mut cur)?;
                    let p_count = read_u32(payload, &mut cur)? as usize;
                    let mut params = Vec::with_capacity(p_count);
                    for _ in 0..p_count {
                        params.push(read_string(payload, &mut cur)?);
                    }
                    let chk_len = read_u32(payload, &mut cur)? as usize;
                    if cur + chk_len > payload.len() {
                        return Err("Données de fonction tronquées".to_string());
                    }
                    let sub_chunk = Chunk::from_bytes(&payload[cur..cur + chk_len])?;
                    cur += chk_len;

                    constants.push(Value::Function(Box::new(crate::ast::UserFunction {
                        name,
                        params,
                        chunk: sub_chunk,
                    })));
                }
                0x06 => {
                    let _count = read_u32(payload, &mut cur)? as usize;
                    let chk_len = read_u32(payload, &mut cur)? as usize;
                    if cur + chk_len > payload.len() {
                        return Err("Données de liste tronquées".to_string());
                    }
                    let sub = Chunk::from_bytes(&payload[cur..cur + chk_len])?;
                    cur += chk_len;
                    constants.push(Value::List(std::rc::Rc::new(std::cell::RefCell::new(sub.constants))));
                }
                0x07 => {
                    let count = read_u32(payload, &mut cur)? as usize;
                    let chk_len = read_u32(payload, &mut cur)? as usize;
                    if cur + chk_len > payload.len() {
                        return Err("Données de map tronquées".to_string());
                    }
                    let sub = Chunk::from_bytes(&payload[cur..cur + chk_len])?;
                    cur += chk_len;
                    let mut map = std::collections::HashMap::new();
                    for i in 0..count {
                        if let (Some(Value::String(k)), Some(v)) = (sub.constants.get(i * 2), sub.constants.get(i * 2 + 1)) {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                    constants.push(Value::Map(std::rc::Rc::new(std::cell::RefCell::new(map))));
                }
                _ => return Err(format!("Tag de constante inconnu: 0x{:02X}", tag)),
            }
        }

        // 2. Lire les fichiers sources
        let file_count = read_u32(payload, &mut cur)? as usize;
        let mut files = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            files.push(read_string(payload, &mut cur)?);
        }

        // 3. Lire le code
        let code_count = read_u32(payload, &mut cur)? as usize;
        let mut code = Vec::with_capacity(code_count);
        let mut lines = Vec::with_capacity(code_count);
        let mut file_ids = Vec::with_capacity(code_count);

        for _ in 0..code_count {
            let line = read_u32(payload, &mut cur)? as usize;
            lines.push(line);

            let file_id = read_u32(payload, &mut cur)? as usize;
            file_ids.push(file_id);

            let op_tag = read_u8(payload, &mut cur)?;
            let op = match op_tag {
                1 => OpCode::OpConstant(read_u32(payload, &mut cur)? as usize),
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
                19 => OpCode::OpDefineGlobal(read_u32(payload, &mut cur)? as usize),
                20 => OpCode::OpGetGlobal(read_u32(payload, &mut cur)? as usize),
                21 => OpCode::OpSetGlobal(read_u32(payload, &mut cur)? as usize),
                22 => OpCode::OpJumpIfFalse(read_u32(payload, &mut cur)? as usize),
                23 => OpCode::OpJump(read_u32(payload, &mut cur)? as usize),
                24 => OpCode::OpLoop(read_u32(payload, &mut cur)? as usize),
                25 => OpCode::OpPrint,
                26 => OpCode::OpInput(read_u32(payload, &mut cur)? as usize),
                27 => {
                    let name_idx = read_u32(payload, &mut cur)? as usize;
                    let argc = read_u32(payload, &mut cur)? as usize;
                    OpCode::OpCall(name_idx, argc)
                }
                28 => OpCode::OpHalt,
                29 => OpCode::OpReturn,
                30 => OpCode::OpBuildList(read_u32(payload, &mut cur)? as usize),
                31 => OpCode::OpBuildMap(read_u32(payload, &mut cur)? as usize),
                32 => OpCode::OpGetIndex,
                33 => OpCode::OpSetIndex,
                _ => return Err(format!("Opcode inconnu: {}", op_tag)),
            };

            code.push(op);
        }

        // Validation défensive des indices de fichiers
        for fid in &file_ids {
            if *fid >= files.len() {
                return Err(format!("ID de fichier invalide dans le bytecode: {}", fid));
            }
        }

        Ok(Chunk { code, constants, lines, file_ids, files })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adler32_known_vector() {
        // Vecteur standard Adler-32 pour "123456789" = 0x091E01DE
        let sum = adler32(b"123456789");
        assert_eq!(sum, 0x091E01DE);
    }

    #[test]
    fn test_chunk_serialization_magic_and_checksum() {
        let mut chunk = Chunk::new();
        chunk.add_file("test.klw");
        let c0 = chunk.add_constant(Value::Number(42.0));
        chunk.write_op_loc(OpCode::OpConstant(c0), 0, 10);
        chunk.write_op_loc(OpCode::OpHalt, 0, 11);

        let bytes = chunk.to_bytes();
        assert!(bytes.len() >= 12);
        assert_eq!(&bytes[0..8], BYTECODE_MAGIC);

        let restored = Chunk::from_bytes(&bytes).expect("Désérialisation valide");
        assert_eq!(chunk, restored);
    }

    #[test]
    fn test_bytecode_corrupted_header_detection() {
        // En-tête trop court
        let short = vec![0u8; 8];
        assert!(Chunk::from_bytes(&short).is_err());

        // Magic incorrect
        let mut wrong_magic = vec![0u8; 16];
        wrong_magic[0..8].copy_from_slice(b"BADMAGIC");
        assert!(Chunk::from_bytes(&wrong_magic).is_err());
    }

    #[test]
    fn test_bytecode_corrupted_checksum_detection() {
        let mut chunk = Chunk::new();
        chunk.write_op(OpCode::OpNil, 1);
        let mut bytes = chunk.to_bytes();

        // Altération d'un octet de payload
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;

        let res = Chunk::from_bytes(&bytes);
        assert!(res.is_err());
        assert!(res.err().unwrap().contains("Adler-32"));
    }

    #[test]
    fn test_bytecode_truncated_payload_never_panics() {
        let mut chunk = Chunk::new();
        let c = chunk.add_constant(Value::String("Chaîne longue pour test".to_string()));
        chunk.write_op(OpCode::OpConstant(c), 1);
        chunk.write_op(OpCode::OpHalt, 2);
        let bytes = chunk.to_bytes();

        // Tronquer à toutes les longueurs possibles de 0 à len
        for cut in 0..bytes.len() {
            let truncated = &bytes[0..cut];
            let res = Chunk::from_bytes(truncated);
            assert!(res.is_err(), "Ne doit jamais paniquer à la coupe {}", cut);
        }
    }
}
