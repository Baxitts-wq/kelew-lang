use crate::ast::{BinaryOp, Expr, Program, Stmt, Value};
use crate::opcode::{Chunk, OpCode};

pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
        }
    }

    pub fn compile(mut self, program: &Program) -> Result<Chunk, String> {
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }

        self.chunk.write_op(OpCode::OpHalt, 0);
        Ok(self.chunk)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::VarDecl { name, initializer, span } => {
                self.compile_expr(initializer)?;
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write_op(OpCode::OpDefineGlobal(name_idx), span.line);
            }
            Stmt::Assign { name, value, span } => {
                self.compile_expr(value)?;
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write_op(OpCode::OpSetGlobal(name_idx), span.line);
                self.chunk.write_op(OpCode::OpPop, span.line);
            }
            Stmt::Print { expr, span } => {
                self.compile_expr(expr)?;
                self.chunk.write_op(OpCode::OpPrint, span.line);
            }
            Stmt::Input { name, span } => {
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write_op(OpCode::OpInput(name_idx), span.line);
            }
            Stmt::If { condition, then_branch, else_branch, span } => {
                self.compile_expr(condition)?;

                // 1. Émettre le saut conditionnel (placeholder)
                let jump_to_else = self.chunk.write_op(OpCode::OpJumpIfFalse(0), span.line);

                // 2. Compiler la branche alors
                for s in then_branch {
                    self.compile_stmt(s)?;
                }

                if let Some(else_stmts) = else_branch {
                    // Saut inconditionnel à la fin du alors
                    let jump_to_end = self.chunk.write_op(OpCode::OpJump(0), span.line);

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
                let exit_jump = self.chunk.write_op(OpCode::OpJumpIfFalse(0), span.line);

                // Corps de la boucle
                for s in body {
                    self.compile_stmt(s)?;
                }

                // Saut arrière vers le début de la boucle
                self.chunk.write_op(OpCode::OpLoop(loop_start), span.line);

                // Patcher le saut de sortie
                let loop_end = self.chunk.code.len();
                self.patch_jump(exit_jump, loop_end);
            }
            Stmt::Function { name, params, body, span } => {
                let mut fn_compiler = Compiler::new();
                for s in body {
                    fn_compiler.compile_stmt(s)?;
                }
                fn_compiler.chunk.write_op(OpCode::OpNil, span.line);
                fn_compiler.chunk.write_op(OpCode::OpReturn, span.line);

                let user_func = crate::ast::UserFunction {
                    name: name.clone(),
                    params: params.clone(),
                    chunk: fn_compiler.chunk,
                };
                let fn_val = Value::Function(Box::new(user_func));
                let const_idx = self.chunk.add_constant(fn_val);
                self.chunk.write_op(OpCode::OpConstant(const_idx), span.line);
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write_op(OpCode::OpDefineGlobal(name_idx), span.line);
            }
            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                } else {
                    self.chunk.write_op(OpCode::OpNil, span.line);
                }
                self.chunk.write_op(OpCode::OpReturn, span.line);
            }
            Stmt::ExprStmt { expr, span } => {
                self.compile_expr(expr)?;
                self.chunk.write_op(OpCode::OpPop, span.line);
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(val, span) => match val {
                Value::Number(_) | Value::String(_) => {
                    let idx = self.chunk.add_constant(val.clone());
                    self.chunk.write_op(OpCode::OpConstant(idx), span.line);
                }
                Value::Bool(true) => {
                    self.chunk.write_op(OpCode::OpTrue, span.line);
                }
                Value::Bool(false) => {
                    self.chunk.write_op(OpCode::OpFalse, span.line);
                }
                Value::Nil => {
                    self.chunk.write_op(OpCode::OpNil, span.line);
                }
                Value::Function(_) => {
                    let idx = self.chunk.add_constant(val.clone());
                    self.chunk.write_op(OpCode::OpConstant(idx), span.line);
                }
            },
            Expr::Variable(name, span) => {
                let idx = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write_op(OpCode::OpGetGlobal(idx), span.line);
            }
            Expr::Input(span) => {
                let idx = self.chunk.add_constant(Value::String("__tmp_input__".to_string()));
                self.chunk.write_op(OpCode::OpInput(idx), span.line);
                self.chunk.write_op(OpCode::OpGetGlobal(idx), span.line);
            }
            Expr::Grouping { expr, .. } => {
                self.compile_expr(expr)?;
            }
            Expr::Call { callee, args, span } => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let name_idx = self.chunk.add_constant(Value::String(callee.clone()));
                self.chunk.write_op(OpCode::OpCall(name_idx, args.len()), span.line);
            }
            Expr::Binary { left, op, right, span } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                match op {
                    BinaryOp::Add => self.chunk.write_op(OpCode::OpAdd, span.line),
                    BinaryOp::Subtract => self.chunk.write_op(OpCode::OpSubtract, span.line),
                    BinaryOp::Multiply => self.chunk.write_op(OpCode::OpMultiply, span.line),
                    BinaryOp::Divide => self.chunk.write_op(OpCode::OpDivide, span.line),
                    BinaryOp::Modulo => self.chunk.write_op(OpCode::OpModulo, span.line),
                    BinaryOp::Equal => self.chunk.write_op(OpCode::OpEqual, span.line),
                    BinaryOp::NotEqual => self.chunk.write_op(OpCode::OpNotEqual, span.line),
                    BinaryOp::Less => self.chunk.write_op(OpCode::OpLess, span.line),
                    BinaryOp::LessEqual => self.chunk.write_op(OpCode::OpLessEqual, span.line),
                    BinaryOp::Greater => self.chunk.write_op(OpCode::OpGreater, span.line),
                    BinaryOp::GreaterEqual => self.chunk.write_op(OpCode::OpGreaterEqual, span.line),
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
