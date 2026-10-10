use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::ast::{BinaryOp, Expr, Program, Stmt, Value};
use crate::lexer::Lexer;
use crate::opcode::{Chunk, OpCode};
use crate::parser::Parser;

pub struct Compiler {
    chunk: Chunk,
    base_dir: PathBuf,
    current_file_id: usize,
    loaded_modules: HashSet<PathBuf>,
    importing_stack: Vec<PathBuf>,
}

impl Compiler {
    pub fn new() -> Self {
        let mut chunk = Chunk::new();
        let current_file_id = chunk.add_file("<source>");
        Compiler {
            chunk,
            base_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            current_file_id,
            loaded_modules: HashSet::new(),
            importing_stack: Vec::new(),
        }
    }

    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        let mut chunk = Chunk::new();
        let current_file_id = chunk.add_file("<source>");
        Compiler {
            chunk,
            base_dir,
            current_file_id,
            loaded_modules: HashSet::new(),
            importing_stack: Vec::new(),
        }
    }

    pub fn with_source_file(base_dir: PathBuf, filename: &str) -> Self {
        let mut chunk = Chunk::new();
        let current_file_id = chunk.add_file(filename);
        Compiler {
            chunk,
            base_dir,
            current_file_id,
            loaded_modules: HashSet::new(),
            importing_stack: Vec::new(),
        }
    }

    fn emit(&mut self, op: OpCode, line: usize) -> usize {
        self.chunk.write_op_loc(op, self.current_file_id, line)
    }

    pub fn compile(mut self, program: &Program) -> Result<Chunk, String> {
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }

        self.emit(OpCode::OpHalt, 0);
        Ok(self.chunk)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::VarDecl { name, initializer, span } => {
                self.compile_expr(initializer)?;
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.emit(OpCode::OpDefineGlobal(name_idx), span.line);
            }
            Stmt::Assign { name, value, span } => {
                self.compile_expr(value)?;
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.emit(OpCode::OpSetGlobal(name_idx), span.line);
                self.emit(OpCode::OpPop, span.line);
            }
            Stmt::IndexAssign { target, index, value, span } => {
                let target_idx = self.chunk.add_constant(Value::String(target.clone()));
                self.emit(OpCode::OpGetGlobal(target_idx), span.line);
                self.compile_expr(index)?;
                self.compile_expr(value)?;
                self.emit(OpCode::OpSetIndex, span.line);
            }
            Stmt::Print { expr, span } => {
                self.compile_expr(expr)?;
                self.emit(OpCode::OpPrint, span.line);
            }
            Stmt::Input { name, span } => {
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.emit(OpCode::OpInput(name_idx), span.line);
            }
            Stmt::If { condition, then_branch, else_branch, span } => {
                self.compile_expr(condition)?;

                // 1. Émettre le saut conditionnel (placeholder)
                let jump_to_else = self.emit(OpCode::OpJumpIfFalse(0), span.line);

                // 2. Compiler la branche alors
                for s in then_branch {
                    self.compile_stmt(s)?;
                }

                if let Some(else_stmts) = else_branch {
                    // Saut inconditionnel à la fin du alors
                    let jump_to_end = self.emit(OpCode::OpJump(0), span.line);

                    // Patcher le saut vers le sinon
                    let else_target = self.chunk.code.len();
                    self.patch_jump(jump_to_else, else_target);

                    for s in else_stmts {
                        self.compile_stmt(s)?;
                    }

                    // Patcher le saut de fin
                    let end_target = self.chunk.code.len();
                    self.patch_jump(jump_to_end, end_target);
                } else {
                    // Patcher directement à la fin du alors
                    let end_target = self.chunk.code.len();
                    self.patch_jump(jump_to_else, end_target);
                }
            }
            Stmt::While { condition, body, span } => {
                let loop_start = self.chunk.code.len();

                // Compiler la condition
                self.compile_expr(condition)?;

                // Saut si faux vers la sortie
                let exit_jump = self.emit(OpCode::OpJumpIfFalse(0), span.line);

                // Corps de la boucle
                for s in body {
                    self.compile_stmt(s)?;
                }

                // Saut arrière vers le début de la boucle
                self.emit(OpCode::OpLoop(loop_start), span.line);

                // Patcher le saut de sortie
                let loop_end = self.chunk.code.len();
                self.patch_jump(exit_jump, loop_end);
            }
            Stmt::Function { name, params, body, span } => {
                let current_fn_file = self.chunk.files.get(self.current_file_id).cloned().unwrap_or_else(|| "<source>".to_string());
                let mut fn_compiler = Compiler::with_source_file(self.base_dir.clone(), &current_fn_file);
                for s in body {
                    fn_compiler.compile_stmt(s)?;
                }
                fn_compiler.emit(OpCode::OpNil, span.line);
                fn_compiler.emit(OpCode::OpReturn, span.line);

                let user_func = crate::ast::UserFunction {
                    name: name.clone(),
                    params: params.clone(),
                    chunk: fn_compiler.chunk,
                };
                let fn_val = Value::Function(Box::new(user_func));
                let const_idx = self.chunk.add_constant(fn_val);
                self.emit(OpCode::OpConstant(const_idx), span.line);
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.emit(OpCode::OpDefineGlobal(name_idx), span.line);
            }
            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                } else {
                    self.emit(OpCode::OpNil, span.line);
                }
                self.emit(OpCode::OpReturn, span.line);
            }
            Stmt::Import { path, span } => {
                let target_path = self.base_dir.join(path);
                let canonical = match target_path.canonicalize() {
                    Ok(c) => c,
                    Err(_) => {
                        return Err(format!(
                            "Module introuvable a la ligne {}: '{}' (chemin teste: '{}')",
                            span.line,
                            path,
                            target_path.display()
                        ));
                    }
                };

                if self.importing_stack.contains(&canonical) {
                    return Err(format!(
                        "Dependance circulaire detectee a la ligne {}: le module '{}' est deja en cours de chargement",
                        span.line, path
                    ));
                }

                if self.loaded_modules.contains(&canonical) {
                    return Ok(());
                }

                self.loaded_modules.insert(canonical.clone());
                self.importing_stack.push(canonical.clone());

                let source = fs::read_to_string(&canonical).map_err(|e| {
                    format!("Erreur de lecture du module '{}': {}", path, e)
                })?;

                let mut lexer = Lexer::new(&source);
                let tokens = lexer.tokenize()?;
                let mut parser = Parser::new(tokens);
                let mod_program = parser.parse()?;

                let prev_base_dir = self.base_dir.clone();
                if let Some(parent) = canonical.parent() {
                    self.base_dir = parent.to_path_buf();
                }

                let mod_file_id = self.chunk.add_file(path);
                let prev_file_id = self.current_file_id;
                self.current_file_id = mod_file_id;

                for mod_stmt in &mod_program.statements {
                    self.compile_stmt(mod_stmt)?;
                }

                self.current_file_id = prev_file_id;
                self.base_dir = prev_base_dir;
                self.importing_stack.pop();
            }
            Stmt::ExprStmt { expr, span } => {
                self.compile_expr(expr)?;
                self.emit(OpCode::OpPop, span.line);
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(val, span) => match val {
                Value::Number(_) | Value::String(_) => {
                    let idx = self.chunk.add_constant(val.clone());
                    self.emit(OpCode::OpConstant(idx), span.line);
                }
                Value::Bool(true) => {
                    self.emit(OpCode::OpTrue, span.line);
                }
                Value::Bool(false) => {
                    self.emit(OpCode::OpFalse, span.line);
                }
                Value::Nil => {
                    self.emit(OpCode::OpNil, span.line);
                }
                Value::Function(_) | Value::List(_) | Value::Map(_) => {
                    let idx = self.chunk.add_constant(val.clone());
                    self.emit(OpCode::OpConstant(idx), span.line);
                }
            },
            Expr::ListLiteral { elements, span } => {
                for elem in elements {
                    self.compile_expr(elem)?;
                }
                self.emit(OpCode::OpBuildList(elements.len()), span.line);
            }
            Expr::MapLiteral { entries, span } => {
                for (k, v) in entries {
                    self.compile_expr(k)?;
                    self.compile_expr(v)?;
                }
                self.emit(OpCode::OpBuildMap(entries.len()), span.line);
            }
            Expr::Index { target, index, span } => {
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                self.emit(OpCode::OpGetIndex, span.line);
            }
            Expr::Variable(name, span) => {
                let idx = self.chunk.add_constant(Value::String(name.clone()));
                self.emit(OpCode::OpGetGlobal(idx), span.line);
            }
            Expr::Input(span) => {
                let idx = self.chunk.add_constant(Value::String("__tmp_input__".to_string()));
                self.emit(OpCode::OpInput(idx), span.line);
                self.emit(OpCode::OpGetGlobal(idx), span.line);
            }
            Expr::Grouping { expr, .. } => {
                self.compile_expr(expr)?;
            }
            Expr::Call { callee, args, span } => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let name_idx = self.chunk.add_constant(Value::String(callee.clone()));
                self.emit(OpCode::OpCall(name_idx, args.len()), span.line);
            }
            Expr::Binary { left, op, right, span } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                match op {
                    BinaryOp::Add => self.emit(OpCode::OpAdd, span.line),
                    BinaryOp::Subtract => self.emit(OpCode::OpSubtract, span.line),
                    BinaryOp::Multiply => self.emit(OpCode::OpMultiply, span.line),
                    BinaryOp::Divide => self.emit(OpCode::OpDivide, span.line),
                    BinaryOp::Modulo => self.emit(OpCode::OpModulo, span.line),
                    BinaryOp::Equal => self.emit(OpCode::OpEqual, span.line),
                    BinaryOp::NotEqual => self.emit(OpCode::OpNotEqual, span.line),
                    BinaryOp::Less => self.emit(OpCode::OpLess, span.line),
                    BinaryOp::LessEqual => self.emit(OpCode::OpLessEqual, span.line),
                    BinaryOp::Greater => self.emit(OpCode::OpGreater, span.line),
                    BinaryOp::GreaterEqual => self.emit(OpCode::OpGreaterEqual, span.line),
                };
            }
        }
        Ok(())
    }

    fn patch_jump(&mut self, jump_idx: usize, target: usize) {
        match self.chunk.code[jump_idx] {
            OpCode::OpJumpIfFalse(_) => {
                self.chunk.code[jump_idx] = OpCode::OpJumpIfFalse(target);
            }
            OpCode::OpJump(_) => {
                self.chunk.code[jump_idx] = OpCode::OpJump(target);
            }
            _ => panic!("Tentative de patch d'une instruction non-jump"),
        }
    }
}
