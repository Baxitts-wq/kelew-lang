pub mod ast;
pub mod bundler;
pub mod compiler;
pub mod disassembler;
pub mod lexer;
pub mod opcode;
pub mod parser;
pub mod stdlib;
pub mod token;
pub mod vm;

use std::env;
use std::fs;
use std::io::{self, Write};

use compiler::Compiler;
use disassembler::Disassembler;
use lexer::Lexer;
use parser::Parser;
use vm::{InterpretResult, VM};

fn print_banner() {
    println!(r#"
  ██████╗  ██╗  ███████╗ ██╗      ███████╗ ██╗    ██╗
  ██╔══██╗ ██║  ██╔════╝ ██║      ██╔════╝ ██║    ██║
  ██████╔╝ ██║  █████╗   ██║      █████╗   ██║ █╗ ██║
  ██╔═══╝  ██║  ██╔══╝   ██║      ██╔══╝   ██║███╗██║
  ██║      ██║  ███████╗ ███████╗ ███████╗ ╚███╔███╔╝
  ╚═╝      ╚═╝  ╚══════╝ ╚══════╝ ╚══════╝  ╚══╝╚══╝ 
    Kelew Bytecode Compiler & Virtual Machine v1.0.0
"#);
}

fn print_help() {
    print_banner();
    println!("USAGE:");
    println!("  kelew run <fichier.klw> [--trace]     Execute bytecode on stack VM");
    println!("  kelew build <fichier.klw> [-o <out>]  Build standalone native executable binary (.exe)");
    println!("  kelew disasm <fichier.klw>            Disassemble bytecode chunk");
    println!("  kelew check <fichier.klw>             Lexical and syntactic verification");
    println!("  kelew repl                            Interactive REPL");
    println!("  kelew demo                            Run demo script");
    println!();
}

fn execute_source(source: &str, trace: bool) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let compiler = Compiler::new();
    let chunk = compiler.compile(&program)?;

    let mut vm = VM::new();
    vm.set_trace(trace);
    match vm.interpret(chunk) {
        InterpretResult::Ok => Ok(()),
        InterpretResult::CompileError => Err("Bytecode compilation error.".to_string()),
        InterpretResult::RuntimeError => Err("Virtual machine runtime error.".to_string()),
    }
}

fn disassemble_source(source: &str, filename: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let compiler = Compiler::new();
    let chunk = compiler.compile(&program)?;

    let disasm = Disassembler::new(&chunk, filename);
    disasm.disassemble();

    Ok(())
}

fn check_source(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let token_count = tokens.len();

    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    let compiler = Compiler::new();
    let chunk = compiler.compile(&program)?;

    println!("[ok] Lexer: {} tokens", token_count);
    println!("[ok] Parser: {} AST statements", program.statements.len());
    println!("[ok] Compiler: {} opcodes, {} constants", chunk.code.len(), chunk.constants.len());
    println!("[ok] Syntax and semantic checks passed.");

    Ok(())
}

fn run_repl() {
    print_banner();
    println!("Shell interactif Kelew VM. Tapez 'exit' pour quitter.\n");

    let mut vm = VM::new();

    loop {
        print!("kelew> ");
        io::stdout().flush().unwrap_or(());

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }

        let trimmed = line.trim();
        if trimmed == "exit" || trimmed == "quit" {
            println!("Baraka ! Sortie du REPL.");
            break;
        }

        if trimmed.is_empty() {
            continue;
        }

        let mut lexer = Lexer::new(trimmed);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse() {
                    Ok(program) => {
                        let comp = Compiler::new();
                        match comp.compile(&program) {
                            Ok(chunk) => {
                                vm.interpret(chunk);
                            }
                            Err(e) => eprintln!("Erreur comp: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Erreur parse: {}", e),
                }
            }
            Err(e) => eprintln!("Erreur lexer: {}", e),
        }
    }
}

fn run_demo() {
    let demo_code = r#"
        T3boun dymek

        # Calcul mathématique avec précédence des opérateurs
        Kelew note1 = 15
        Kelew note2 = 19
        Kelew coefficient = 2
        Kelew moyenne = (note1 + note2) / coefficient

        Ejacule "==============================="
        Ejacule " RAPPORT ACADEMIQUE KELEW"
        Ejacule "==============================="
        Ejacule "Total calcule :"
        Ejacule note1 + note2
        Ejacule "Moyenne ponderee :"
        Ejacule moyenne

        # Branchement conditionnel
        Zaama moyenne >= 10
            Ejacule "Statut : Admis avec mention (Sah) !"
        Walou
            Ejacule "Statut : Recale (K7hba) !"
        Khalas

        # Décompte avec boucle bytecode haute performance
        Ejacule "Decompte Bytecode VM :"
        Kelew i = 3
        Dima i > 0
            Ejacule i
            Kelew i = i - 1
        Baraka

        Ejacule "Execution achevee avec succes."
        Fio Del puta
    "#;

    println!("=== EXECUTION DU SCRIPT DEMO SUR LA VM ===");
    if let Err(e) = execute_source(demo_code, false) {
        eprintln!("Erreur: {}", e);
    }
}

fn main() {
    // 1. Détection de binaire autonome : si le .exe contient du bytecode embarqué, on l'exécute directement
    if let Some(embedded_chunk) = bundler::Bundler::extract_embedded_bytecode() {
        let mut vm = VM::new();
        match vm.interpret(embedded_chunk) {
            InterpretResult::Ok => std::process::exit(0),
            _ => std::process::exit(1),
        }
    }

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "build" => {
            if args.len() < 3 {
                eprintln!("Usage: kelew build <fichier.klw> [-o <sortie.exe>]");
                return;
            }
            let source = &args[2];
            let output = if args.len() >= 5 && args[3] == "-o" {
                args[4].clone()
            } else {
                let p = std::path::Path::new(source);
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("app");
                format!("{}.exe", stem)
            };

            match bundler::Bundler::build_executable(source, &output) {
                Ok(_) => println!("[ok] Executable natif autonome genere : {}", output),
                Err(e) => eprintln!("Erreur de generation : {}", e),
            }
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: kelew run <fichier.klw> [--trace]");
                return;
            }
            let filename = &args[2];
            let trace = args.get(3).map(|s| s == "--trace").unwrap_or(false);

            match fs::read_to_string(filename) {
                Ok(source) => {
                    if let Err(e) = execute_source(&source, trace) {
                        eprintln!("Erreur d'exécution: {}", e);
                    }
                }
                Err(e) => eprintln!("Impossible de lire le fichier '{}': {}", filename, e),
            }
        }
        "disasm" => {
            if args.len() < 3 {
                eprintln!("Usage: kelew disasm <fichier.klw>");
                return;
            }
            let filename = &args[2];
            match fs::read_to_string(filename) {
                Ok(source) => {
                    if let Err(e) = disassemble_source(&source, filename) {
                        eprintln!("Erreur de désassemblage: {}", e);
                    }
                }
                Err(e) => eprintln!("Impossible de lire le fichier '{}': {}", filename, e),
            }
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("Usage: kelew check <fichier.klw>");
                return;
            }
            let filename = &args[2];
            match fs::read_to_string(filename) {
                Ok(source) => {
                    if let Err(e) = check_source(&source) {
                        eprintln!("Erreur de vérification: {}", e);
                    }
                }
                Err(e) => eprintln!("Impossible de lire le fichier '{}': {}", filename, e),
            }
        }
        "repl" => run_repl(),
        "demo" => run_demo(),
        "--help" | "-h" => print_help(),
        other => {
            // Si c'est directement un fichier: kelew script.klw
            if other.ends_with(".klw") || other.ends_with(".kelew") {
                match fs::read_to_string(other) {
                    Ok(source) => {
                        if let Err(e) = execute_source(&source, false) {
                            eprintln!("Erreur: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Erreur lecture: {}", e),
                }
            } else {
                eprintln!("Commande inconnue '{}'. Tapez 'kelew --help' pour la liste des commandes.", other);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_complet_calcul() {
        let code = r#"
            T3boun dymek
            Kelew x = 10 * 2 + 5
            Zaama x == 25
                Kelew res = "VALIDE"
            Walou
                Kelew res = "INVALIDE"
            Khalas
            Ejacule res
            Fio Del puta
        "#;

        let res = execute_source(code, false);
        assert!(res.is_ok());
    }

    #[test]
    fn test_boucle_vm() {
        let code = r#"
            T3boun dymek
            Kelew n = 10
            Dima n > 0
                Kelew n = n - 1
            Baraka
            Fio Del puta
        "#;

        let res = execute_source(code, false);
        assert!(res.is_ok());
    }

    #[test]
    fn test_disassembler_generation() {
        let code = r#"
            T3boun dymek
            Kelew total = 100
            Ejacule total
            Fio Del puta
        "#;

        let res = disassemble_source(code, "test_disasm");
        assert!(res.is_ok());
    }
}
