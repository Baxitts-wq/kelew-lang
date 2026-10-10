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

    pub fn get_globals(&self) -> &HashMap<String, Value> {
        &self.globals
    }

    pub fn get_globals_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.globals
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
                            return self.runtime_error(&format!("Types incompatibles pour l'addition: '{}' et '{}'", a, b));
                        }
                    }
                }
                OpCode::OpSubtract => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Number(x - y)),
                        (a, b) => {
                            return self.runtime_error(&format!("Soustraction requiert deux nombres: '{}' et '{}'", a, b));
                        }
                    }
                }
                OpCode::OpMultiply => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Number(x * y)),
                        (a, b) => {
                            return self.runtime_error(&format!("Multiplication requiert deux nombres: '{}' et '{}'", a, b));
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
                            return self.runtime_error(&format!("Division requiert deux nombres: '{}' et '{}'", a, b));
                        }
                    }
                }
                OpCode::OpModulo => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.push(Value::Number(x % y)),
                        (a, b) => {
                            return self.runtime_error(&format!("Modulo requiert deux nombres: '{}' et '{}'", a, b));
                        }
                    }
                }
                OpCode::OpNegate => {
                    let val = self.pop();
                    match val {
                        Value::Number(n) => self.push(Value::Number(-n)),
                        other => {
                            return self.runtime_error(&format!("Négation numérique impossible sur '{}'", other));
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
                        return self.runtime_error(&format!("Variable globale non définie: '{}'", name));
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
                OpCode::OpCall(name_idx, argc) => {
                    let name = match &self.chunk.constants[name_idx] {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(self.pop());
                    }
                    args.reverse();

                    if let Some(Value::Function(user_fn)) = self.globals.get(&name).cloned() {
                        if args.len() != user_fn.params.len() {
                            return self.runtime_error(&format!(
                                "Fonction '{}' attend {} arguments, reçu: {}",
                                name,
                                user_fn.params.len(),
                                args.len()
                            ));
                        }

                        let mut sub_globals = self.globals.clone();
                        for (param, arg) in user_fn.params.iter().zip(args.into_iter()) {
                            sub_globals.insert(param.clone(), arg);
                        }

                        let mut sub_vm = VM {
                            chunk: user_fn.chunk,
                            ip: 0,
                            stack: Vec::with_capacity(64),
                            globals: sub_globals,
                            trace_execution: self.trace_execution,
                        };

                        let status = sub_vm.run();
                        let ret_val = sub_vm.stack.pop().unwrap_or(Value::Nil);

                        for (k, v) in sub_vm.globals {
                            if !user_fn.params.contains(&k) {
                                self.globals.insert(k, v);
                            }
                        }

                        if status == InterpretResult::RuntimeError {
                            return InterpretResult::RuntimeError;
                        }

                        self.push(ret_val);
                    } else {
                        match crate::stdlib::StdLib::call(&name, &args) {
                            Ok(result) => self.push(result),
                            Err(e) => {
                                return self.runtime_error(&format!("Erreur Native Kelew: {}", e));
                            }
                        }
                    }
                }
                OpCode::OpBuildList(count) => {
                    let mut items = Vec::with_capacity(count);
                    for _ in 0..count {
                        items.push(self.pop());
                    }
                    items.reverse();
                    self.push(Value::List(std::rc::Rc::new(std::cell::RefCell::new(items))));
                }
                OpCode::OpBuildMap(count) => {
                    let mut map = HashMap::with_capacity(count);
                    for _ in 0..count {
                        let val = self.pop();
                        let key = match self.pop() {
                            Value::String(s) => s,
                            other => other.to_string(),
                        };
                        map.insert(key, val);
                    }
                    self.push(Value::Map(std::rc::Rc::new(std::cell::RefCell::new(map))));
                }
                OpCode::OpGetIndex => {
                    let index = self.pop();
                    let target = self.pop();
                    match target {
                        Value::List(l) => {
                            let borrowed = l.borrow();
                            match index {
                                Value::Number(n) => {
                                    let idx = n as usize;
                                    if n < 0.0 || idx >= borrowed.len() {
                                        return self.runtime_error(&format!("Index de liste hors limites: {} (taille: {})", n, borrowed.len()));
                                    }
                                    self.push(borrowed[idx].clone());
                                }
                                other => {
                                    return self.runtime_error(&format!("L'index d'une liste doit etre un nombre, recu: '{}'", other));
                                }
                            }
                        }
                        Value::Map(m) => {
                            let key = match index {
                                Value::String(s) => s,
                                other => other.to_string(),
                            };
                            let borrowed = m.borrow();
                            let val = borrowed.get(&key).cloned().unwrap_or(Value::Nil);
                            self.push(val);
                        }
                        Value::String(s) => {
                            match index {
                                Value::Number(n) => {
                                    let idx = n as usize;
                                    if n >= 0.0 && idx < s.len() {
                                        let ch = s.chars().nth(idx).unwrap_or('\0').to_string();
                                        self.push(Value::String(ch));
                                    } else {
                                        self.push(Value::Nil);
                                    }
                                }
                                _ => self.push(Value::Nil),
                            }
                        }
                        other => {
                            return self.runtime_error(&format!("Indexation impossible sur '{}'", other));
                        }
                    }
                }
                OpCode::OpSetIndex => {
                    let val = self.pop();
                    let index = self.pop();
                    let target = self.pop();
                    match target {
                        Value::List(l) => {
                            match index {
                                Value::Number(n) => {
                                    let idx = n as usize;
                                    let mut borrowed = l.borrow_mut();
                                    if n >= 0.0 && idx < borrowed.len() {
                                        borrowed[idx] = val;
                                    } else if n >= 0.0 && idx == borrowed.len() {
                                        borrowed.push(val);
                                    } else {
                                        return self.runtime_error(&format!("Index hors limites pour assignation: {}", n));
                                    }
                                }
                                other => {
                                    return self.runtime_error(&format!("Index invalide pour assignation de liste: '{}'", other));
                                }
                            }
                        }
                        Value::Map(m) => {
                            let key = match index {
                                Value::String(s) => s,
                                other => other.to_string(),
                            };
                            m.borrow_mut().insert(key, val);
                        }
                        other => {
                            return self.runtime_error(&format!("Assignation par index impossible sur '{}'", other));
                        }
                    }
                }
                OpCode::OpReturn => {
                    break;
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
            Value::Function(_) => true,
            Value::List(l) => !l.borrow().is_empty(),
            Value::Map(m) => !m.borrow().is_empty(),
        }
    }

    fn runtime_error(&self, message: &str) -> InterpretResult {
        let line = if self.ip > 0 && self.ip - 1 < self.chunk.lines.len() {
            self.chunk.lines[self.ip - 1]
        } else {
            0
        };
        if line > 0 {
            eprintln!("[Erreur Runtime VM à la ligne {}] {}", line, message);
        } else {
            eprintln!("[Erreur Runtime VM] {}", message);
        }
        InterpretResult::RuntimeError
    }
}
