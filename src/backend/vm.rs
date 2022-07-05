use std::{thread, time, usize};

use super::{
    instruction::{Instr, Type},
    memory::Memory,
    scope::{id, Scope},
    stack::{Stack, StackValue},
};

pub struct VM {
    pub program: Vec<Instr>,
    pub pc: usize,

    pub stack: Stack,
    pub scopes: Vec<Scope>,
    pub fp: usize,

    pub heap: Memory,
}

impl VM {
    pub fn new() -> VM {
        VM {
            program: vec![],
            pc: 0,

            stack: Stack::new(),
            heap: Memory::new(),

            scopes: vec![Scope::new(0)],
            fp: 0,
        }
    }

    pub fn run(&mut self) {
        while self.pc < self.program.len() {
            self.step();
        }
    }

    pub fn all_scopes_get(&self, id: id) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(addr) = scope.get(id) {
                return Some(addr);
            }
        }

        None
    }

    pub fn all_scopes_get_desc(&self, id: id) -> Option<usize> {
        let mut n = 0;
        for scope in self.scopes.iter().rev() {
            if let Some(addr) = scope.get(id - n) {
                return Some(addr);
            }
            n += 1;
        }

        None
    }

    pub fn pop_stack(&mut self) -> Type {
        let value = self.stack.pop();

        match value {
            StackValue::Literal(value) => value,
            StackValue::Addr(addr) => self.heap.get(addr).to_owned(),
            // StackValue::Ref(addr) => ,
            // StackValue::Offset(offset) => self.get_mem_offset(offset),
        }
    }

    pub fn delete_locals(&mut self, scope: &Scope) {
        for addr in scope.0.values() {
            // println!("DELETING LOCAL {}", addr);
            self.heap.delete(*addr);
        }
    }

    pub fn enter_scope(&mut self, return_to: usize) {
        self.scopes.push(Scope::new(return_to));
        self.fp += 1;
    }

    pub fn exit_scope(&mut self) -> usize {
        let scope = self.scopes.pop().expect("Exited from empty scope");
        self.delete_locals(&scope);
        self.fp -= 1;
        scope.1
    }

    pub fn step(&mut self) {
        let instruction = &self.program[self.pc];
        self.pc += 1;

        // let ten_millis = time::Duration::from_millis(200);
        // thread::sleep(ten_millis);

        // if instruction != &Opcode::Noop {
        //     // println!("{:?}", self.stack.internal);
        //     println!("{:?}", instruction);
        // }

        match instruction {
            Instr::Halt => {
                println!("Halt");
                return;
            }
            Instr::Push(value) => {
                self.stack.push(value.to_owned());
            }
            Instr::Pop => {
                self.stack.pop();
            }
            Instr::Store(id) => {
                let id = *id + self.fp;
                let value = self.pop_stack();

                if let Some(addr) = self.all_scopes_get(id) {
                    self.heap.set(addr, value);
                } else {
                    let addr = self.heap.add(value);
                    self.scopes[self.fp].set(id, addr);
                }
            }
            Instr::StoreLocal(id) => {
                let id = *id + self.fp;
                let value = self.pop_stack();

                if let Some(addr) = self.scopes[self.fp].get(id) {
                    self.heap.set(addr, value);
                } else {
                    let addr = self.heap.add(value);
                    self.scopes[self.fp].set(id, addr);
                }
            }
            Instr::StoreGlobal(id) => {
                let id = *id;
                let value = self.pop_stack();

                if let Some(addr) = self.scopes[0].get(id) {
                    self.heap.set(addr, value);
                } else {
                    let addr = self.heap.add(value);
                    self.scopes[self.fp].set(id, addr);
                }
            }

            Instr::Register(id, addr) => {
                self.scopes[self.fp].set(*id, *addr);
            }
            Instr::Load(id) => {
                let id = *id + self.fp;
                let addr = self.all_scopes_get(id).expect("Undefined variable ");
                self.stack.push(StackValue::Addr(addr.clone()));
            }
            Instr::LoadGlobal(id) => {
                let addr = self.scopes[0].get(*id).expect("Undefined variable");
                self.stack.push(StackValue::Addr(addr.clone()));
            }
            Instr::LoadLocal(id) => {
                let id = *id + self.fp;
                let addr = self.scopes[self.fp].get(id).expect("Undefined variable");
                self.stack.push(StackValue::Addr(addr.clone()));
            }
            Instr::LoadName(id) => {
                let id = *id + self.fp;
                let addr = self.all_scopes_get_desc(id).expect("Undefined variable");

                // let value = self.heap.get(addr);
                // self.stack.push(StackValue::Literal(value.clone()));
                self.stack.push(StackValue::Addr(addr.clone()));
            }
            Instr::Jump(to) => {
                self.pc = *to;
            }
            Instr::JumpIf(to) => {
                let to = *to;
                let value = self.pop_stack();

                if value == Type::Bool(true)
                    && value != Type::Number(0.0)
                    && value != Type::String("".to_owned())
                {
                    self.pc = to;
                }
            }
            Instr::JumpIfNot(to) => {
                let to = *to;
                let value = self.pop_stack();

                if value == Type::Bool(false)
                    || value == Type::Number(0.0)
                    || value == Type::String("".to_owned())
                {
                    self.pc = to;
                }
            }

            Instr::Call => {
                // let addr = self.all_scopes_get(id).expect("Call to undefined function");
                let top = self.pop_stack();
                if let Type::FuncPtr(jump) = top {
                    // let jump = *jump;
                    self.enter_scope(self.pc);
                    self.pc = jump;
                } else {
                    panic!("Call to non-function {:?}", top);
                }

                // let addr = self.scopes[self.fp]
                //     .get(id)
                //     .expect("Call to undefined function");
            }
            Instr::Return => {
                let value = &self.stack.pop();
                match value {
                    StackValue::Literal(_) => {
                        self.stack.push(value.to_owned());
                    }
                    StackValue::Addr(addr) => self
                        .stack
                        .push(StackValue::Literal(self.heap.get(*addr).to_owned())),
                }

                self.pc = self.exit_scope();
            }

            Instr::Add => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs + rhs),
                    (Type::String(lhs), Type::String(rhs)) => Type::String(lhs + &rhs),
                    _ => panic!("Addition not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Sub => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs - rhs),
                    _ => panic!("Subtraction not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Mul => {
                let lhs = self.pop_stack();
                let rhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs * rhs),
                    _ => panic!("Multiplication not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Div => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs / rhs),
                    _ => panic!("Division not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Mod => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs % rhs),
                    _ => panic!("Modulo not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Eq => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (&lhs, &rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs == rhs),
                    (Type::String(lhs), Type::String(rhs)) => Type::Bool(lhs == rhs),
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs == rhs),
                    _ => Type::Bool(lhs == rhs),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Neq => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs != rhs),
                    (Type::String(lhs), Type::String(rhs)) => Type::Bool(lhs != rhs),
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs != rhs),
                    _ => panic!("Inequality not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Lt => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs < rhs),
                    _ => panic!("Less than not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Gt => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs > rhs),
                    _ => panic!("Greater than not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Lte => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs <= rhs),
                    _ => panic!("Less than or equal not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Gte => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs >= rhs),
                    _ => panic!("Greater than or equal not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::And => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs && rhs),
                    _ => panic!("And not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Or => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs || rhs),
                    _ => panic!("Or not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Not => {
                let value = self.pop_stack();

                let result = match value {
                    Type::Bool(value) => Type::Bool(!value),
                    _ => panic!("Not not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Neg => {
                let value = self.pop_stack();

                let result = match value {
                    Type::Number(value) => Type::Number(-value),
                    _ => panic!("Negation not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Pow => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs.powf(rhs)),
                    _ => panic!("Power not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Print => match self.pop_stack() {
                Type::String(value) => println!("{}", value),
                Type::Number(value) => println!("{}", value),
                Type::Bool(value) => println!("{}", value),
                _ => panic!("Print not supported"),
            },
            Instr::Noop => {}
            _ => {
                println!("NOT HANDLED: {:?}", instruction);
            }
        }
    }
}
