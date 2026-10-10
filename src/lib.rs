pub mod ast;
pub mod compiler;
pub mod disassembler;
pub mod lexer;
pub mod opcode;
pub mod parser;
pub mod stdlib;
pub mod token;
pub mod vm;

pub use ast::Value;
pub use compiler::Compiler;
pub use disassembler::Disassembler;
pub use lexer::Lexer;
pub use opcode::{Chunk, OpCode};
pub use parser::Parser;
pub use vm::{InterpretResult, VM};

/// Moteur de scripting embarqué Kelew
pub struct Engine {
    vm: VM,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            vm: VM::new(),
        }
    }

    /// Exécute une chaîne de code Kelew
    pub fn eval(&mut self, source: &str) -> Result<(), String> {
        self.eval_internal(source, false)
    }

    /// Exécute avec le traçage dynamique de la pile de la VM
    pub fn eval_with_trace(&mut self, source: &str) -> Result<(), String> {
        self.eval_internal(source, true)
    }

    fn eval_internal(&mut self, source: &str, trace: bool) -> Result<(), String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        let compiler = Compiler::new();
        let chunk = compiler.compile(&program)?;

        self.vm.set_trace(trace);
        match self.vm.interpret(chunk) {
            InterpretResult::Ok => Ok(()),
            InterpretResult::CompileError => Err("Erreur de compilation du bytecode.".to_string()),
            InterpretResult::RuntimeError => Err("Erreur d'exécution de la machine virtuelle.".to_string()),
        }
    }

    /// Récupère la valeur d'une variable globale
    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.vm.get_globals().get(name)
    }

    /// Définit la valeur d'une variable globale
    pub fn set_global(&mut self, name: &str, value: Value) {
        self.vm.get_globals_mut().insert(name.to_string(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_engine_api() {
        let mut engine = Engine::new();
        let code = r#"
            T3boun dymek
            Kelew score = 40 + 2
            Kelew nom = "KelewVM"
            Fio Del puta
        "#;
        assert!(engine.eval(code).is_ok());
        assert_eq!(engine.get_global("score"), Some(&Value::Number(42.0)));
        assert_eq!(engine.get_global("nom"), Some(&Value::String("KelewVM".to_string())));
    }

    #[test]
    fn test_stdlib_file_io() {
        let mut engine = Engine::new();
        let code = r#"
            T3boun dymek
            EcrireFichier("test_output.txt", "Contenu de test")
            Kelew existe = FichierExiste("test_output.txt")
            Kelew lu = LireFichier("test_output.txt")
            SupprimerFichier("test_output.txt")
            Fio Del puta
        "#;
        assert!(engine.eval(code).is_ok());
        assert_eq!(engine.get_global("existe"), Some(&Value::Bool(true)));
        assert_eq!(engine.get_global("lu"), Some(&Value::String("Contenu de test".to_string())));
    }

    #[test]
    fn test_chunk_serialization_roundtrip() {
        let mut chunk = Chunk::new();
        let c0 = chunk.add_constant(Value::Number(123.45));
        let c1 = chunk.add_constant(Value::String("Salut".to_string()));
        chunk.write_op(OpCode::OpConstant(c0), 1);
        chunk.write_op(OpCode::OpConstant(c1), 1);
        chunk.write_op(OpCode::OpAdd, 2);
        chunk.write_op(OpCode::OpHalt, 3);

        let bytes = chunk.to_bytes();
        let restored = Chunk::from_bytes(&bytes).unwrap();

        assert_eq!(chunk, restored);
    }
}
