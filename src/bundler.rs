use std::env;
use std::fs;
use std::io::{Read, Seek, SeekFrom};

use crate::compiler::Compiler;
use crate::lexer::Lexer;
use crate::opcode::Chunk;
use crate::parser::Parser;

pub const MAGIC_TRAILER: &[u8; 20] = b"KELEW_STANDALONE_V1\0";

pub struct Bundler;

impl Bundler {
    /// Génère un fichier exécutable natif (.exe) autonome avec bytecode embarqué
    pub fn build_executable(source_path: &str, output_path: &str) -> Result<(), String> {
        // 1. Lire le code source Kelew
        let source = fs::read_to_string(source_path)
            .map_err(|e| format!("Impossible de lire le fichier source '{}': {}", source_path, e))?;

        // 2. Compiler en Bytecode
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        let compiler = Compiler::new();
        let chunk = compiler.compile(&program)?;

        let chunk_bytes = chunk.to_bytes();
        let chunk_len = chunk_bytes.len() as u64;

        // 3. Lire le binaire hôte actuel (kelew.exe)
        let current_exe = env::current_exe()
            .map_err(|e| format!("Impossible de déterminer le chemin du binaire hôte: {}", e))?;

        let mut host_bytes = fs::read(&current_exe)
            .map_err(|e| format!("Impossible de lire le binaire hôte '{:?}': {}", current_exe, e))?;

        // Nettoyer un éventuel payload précédent si le binaire hôte était déjà un bundle
        if let Some(clean_len) = Self::detect_embedded_offset(&host_bytes) {
            host_bytes.truncate(clean_len);
        }

        // 4. Assembler le binaire final
        // [HOST EXECUTABLE] + [BYTECODE CHUNK] + [CHUNK_LEN: 8 bytes] + [MAGIC_TRAILER: 20 bytes]
        let mut final_binary = host_bytes;
        final_binary.extend_from_slice(&chunk_bytes);
        final_binary.extend_from_slice(&chunk_len.to_le_bytes());
        final_binary.extend_from_slice(MAGIC_TRAILER);

        // 5. Écrire le binaire de sortie
        fs::write(output_path, &final_binary)
            .map_err(|e| format!("Échec de l'écriture de l'exécutable autonome '{}': {}", output_path, e))?;

        Ok(())
    }

    /// Tente d'extraire le bytecode embarqué dans le fichier exécutable en cours d'exécution
    pub fn extract_embedded_bytecode() -> Option<Chunk> {
        let exe_path = env::current_exe().ok()?;
        let mut file = fs::File::open(exe_path).ok()?;
        let file_len = file.metadata().ok()?.len() as usize;
        let trailer_size: usize = 28;
        if file_len < trailer_size {
            return None;
        }

        // Lire le trailer à la fin du fichier
        file.seek(SeekFrom::End(-(trailer_size as i64))).ok()?;
        let mut trailer = [0u8; 28];
        file.read_exact(&mut trailer).ok()?;

        let magic = &trailer[8..28];
        if magic != MAGIC_TRAILER {
            return None;
        }

        let chunk_len = u64::from_le_bytes(trailer[0..8].try_into().ok()?) as usize;
        if file_len < trailer_size + chunk_len {
            return None;
        }
        let chunk_offset = file_len - trailer_size - chunk_len;

        file.seek(SeekFrom::Start(chunk_offset as u64)).ok()?;
        let mut chunk_bytes = vec![0u8; chunk_len];
        file.read_exact(&mut chunk_bytes).ok()?;

        Chunk::from_bytes(&chunk_bytes).ok()
    }

    fn detect_embedded_offset(bytes: &[u8]) -> Option<usize> {
        let trailer_size = 28;
        if bytes.len() < trailer_size {
            return None;
        }
        let magic = &bytes[bytes.len() - 20..];
        if magic == MAGIC_TRAILER {
            let len_bytes = &bytes[bytes.len() - 28..bytes.len() - 20];
            let chunk_len = u64::from_le_bytes(len_bytes.try_into().ok()?) as usize;
            Some(bytes.len() - trailer_size - chunk_len)
        } else {
            None
        }
    }
}
