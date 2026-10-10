use std::collections::HashMap;
use std::io::{self, Write};

use crate::ast::Value;
use crate::opcode::{Chunk, OpCode};

#[derive(Debug, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    trace_execution: bool,
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            trace_execution: false,
        }
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace_execution = trace;
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        self.stack.clear();

        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        let mut cycle_count: usize = 0;
        const MAX_CYCLES: usize = 10_000_000;

        while self.ip < self.chunk.code.len() {
            cycle_count += 1;
            if cycle_count > MAX_CYCLES {
                eprintln!("\n[Erreur Critique VM] Boucle infinie détectée (> 10M cycles). Arrêt d'urgence.");
                return InterpretResult::RuntimeError;
            }

            if self.trace_execution {
                print!("          [Pile: ");
                for slot in &self.stack {
                    print!("[ {} ]", slot);
                }
                println!("]");
            }

            let op = self.chunk.code[self.ip];
            self.ip += 1;

            match op {
                OpCode::OpConstant(idx) => {
                    let constant = self.chunk.constants[idx].clone();
                    self.push(constant);
                }
                OpCode::OpNil => self.push(Value::Nil),
                OpCode::OpTrue => self.push(Value::Bool(true)),
                OpCode::OpFalse => self.push(Value::Bool(false)),
                OpCode::OpPop => {
                    self.pop();
                }
                OpCode::OpAdd => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Number(x + y)),
                        (Value::String(x), Value::String(y)) => self.push(Value::String(format!("{}{}", x, y))),
                        (Value::String(x), Value::Number(y)) => self.push(Value::String(format!("{}{}", x, y))),
                        (Value::Number(x), Value::String(y)) => self.push(Value::String(format!("{}{}", x, y))),
                        (a, b) => {
                            eprintln!("[Erreur Runtime VM] Types incompatibles pour l'addition: '{}' et '{}'", a, b);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpSubtract => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Number(x - y)),
                        (a, b) => {
                            eprintln!("[Erreur Runtime VM] Soustraction requiert deux nombres: '{}' et '{}'", a, b);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpMultiply => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Number(x * y)),
                        (a, b) => {
                            eprintln!("[Erreur Runtime VM] Multiplication requiert deux nombres: '{}' et '{}'", a, b);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpDivide => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            if y == 0.0 {
                                eprintln!("[Alerte VM] Division par zéro.");
                                self.push(Value::Number(0.0));
                            } else {
                                self.push(Value::Number(x / y));
                            }
                        }
                        (a, b) => {
                            eprintln!("[Erreur Runtime VM] Division requiert deux nombres: '{}' et '{}'", a, b);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpModulo => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Number(x % y)),
                        (a, b) => {
                            eprintln!("[Erreur Runtime VM] Modulo requiert deux nombres: '{}' et '{}'", a, b);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpNegate => {
                    let val = self.pop();
                    match val {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        other => {
                            eprintln!("[Erreur Runtime VM] Négation numérique impossible sur '{}'", other);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::OpEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                OpCode::OpNotEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a != b));
                }
                OpCode::OpGreater => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Bool(x > y)),
                        _ => self.push(Value::Bool(false)),
                    }
                }
                OpCode::OpGreaterEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Bool(x >= y)),
                        _ => self.push(Value::Bool(false)),
                    }
                }
                OpCode::OpLess => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Bool(x < y)),
                        _ => self.push(Value::Bool(false)),
                    }
                }
                OpCode::OpLessEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Bool(x <= y)),
                        _ => self.push(Value::Bool(false)),
                    }
                }
                OpCode::OpNot => {
                    let val = self.pop();
                    let b = self.is_truthy(&val);
                    self.push(Value::Bool(!b));
                }
                OpCode::OpDefineGlobal(idx) => {
                    let name = match &self.chunk.constants[idx] {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    let val = self.pop();
                    self.globals.insert(name, val);
                }
                OpCode::OpGetGlobal(idx) => {
                    let name = match &self.chunk.constants[idx] {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    if let Some(val) = self.globals.get(&name) {
                        self.push(val.clone());
                    } else {
                        eprintln!("[Erreur Runtime VM] Variable globale non définie: '{}'", name);
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpSetGlobal(idx) => {
                    let name = match &self.chunk.constants[idx] {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    let val = self.peek(0).clone();
                    if self.globals.contains_key(&name) {
                        self.globals.insert(name, val);
                    } else {
                        self.globals.insert(name, val);
                    }
                }
                OpCode::OpJumpIfFalse(target) => {
                    let cond = self.pop();
                    if !self.is_truthy(&cond) {
                        self.ip = target;
                    }
                }
                OpCode::OpJump(target) => {
                    self.ip = target;
                }
                OpCode::OpLoop(target) => {
                    self.ip = target;
                }
                OpCode::OpPrint => {
                    let val = self.pop();
                    println!("{}", val);
                }
                OpCode::OpInput(idx) => {
                    let name = match &self.chunk.constants[idx] {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    print!("Kelew [Saisie '{}']> ", name);
                    io::stdout().flush().unwrap_or(());
                    let mut input = String::new();
                    if io::stdin().read_line(&mut input).is_ok() {
                        let trim = input.trim();
                        let val = if let Ok(n) = trim.parse::<f64>() {
                            Value::Number(n)
                        } else if trim.eq_ignore_ascii_case("sah") {
                            Value::Bool(true)
                        } else if trim.eq_ignore_ascii_case("k7hba") {
                            Value::Bool(false)
                        } else {
                            Value::String(trim.to_string())
                        };
                        self.globals.insert(name, val);
                    }
                }
                OpCode::OpHalt => {
                    break;
                }
            }
        }

        InterpretResult::Ok
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Nil)
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Bool(b) => *b,
            Value::Nil => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
        }
    }
}
