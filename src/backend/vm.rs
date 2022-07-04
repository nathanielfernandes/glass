use std::{thread, time};

use super::{
    instruction::{Opcode, Type},
    memory::Memory,
    scope::{id, Scope},
    stack::{Stack, StackValue},
};

pub struct VM {
    pub program: Vec<Opcode>,
    pub pc: usize,

    pub stack: Stack,
    pub call_stack: Vec<usize>,

    pub heap: Memory,
    pub scopes: Vec<Scope>,
    pub fp: usize,
}

impl VM {
    pub fn new() -> VM {
        VM {
            program: vec![],
            pc: 0,

            stack: Stack::new(),
            call_stack: Vec::new(),
            heap: Memory::new(),

            scopes: vec![Scope::new()],
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
            self.heap.delete(*addr);
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
        self.fp += 1;
    }

    pub fn exit_scope(&mut self) {
        let scope = self.scopes.pop().expect("Exited from empty scope");
        self.delete_locals(&scope);
        self.fp -= 1;
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
            Opcode::Halt => {
                println!("Halt");
                return;
            }
            Opcode::Push(value) => {
                self.stack.push(value.to_owned());
            }
            Opcode::Pop => {
                self.stack.pop();
            }
            Opcode::Store(id) => {
                let id = *id + self.fp;
                let value = self.pop_stack();

                if let Some(addr) = self.all_scopes_get(id) {
                    self.heap.set(addr, value);
                } else {
                    let addr = self.heap.add(value);
                    self.scopes[self.fp].set(id, addr);
                }
            }
            Opcode::Register(id, addr) => {
                self.scopes[self.fp].set(*id + self.fp, *addr);
            }
            // Opcode::Register(id) => {
            //     let id = *id + self.fp;
            //     let value = self.pop_stack();

            //     if let Type::Ref(r) = value {
            //         self.scopes[self.fp].set(id, r);
            //     } else {
            //         panic!("Register expects a reference");
            //     }
            // }
            Opcode::Load(id) => {
                let id = *id + self.fp;
                let addr = self.all_scopes_get(id).expect("Undefined variable");
                self.stack.push(StackValue::Addr(addr.clone()));
            }
            Opcode::Jump(to) => {
                self.pc = *to;
            }
            Opcode::JumpIf(to) => {
                let to = *to;
                let value = self.pop_stack();

                if value == Type::Bool(true)
                    && value != Type::Number(0.0)
                    && value != Type::String("".to_owned())
                {
                    self.pc = to;
                }
            }
            Opcode::JumpIfNot(to) => {
                let to = *to;
                let value = self.pop_stack();

                if value == Type::Bool(false)
                    || value == Type::Number(0.0)
                    || value == Type::String("".to_owned())
                {
                    self.pc = to;
                }
            }

            Opcode::Call(id) => {
                let id = *id;
                let addr = self.all_scopes_get(id).expect("Call to undefined function");

                self.enter_scope();
                self.call_stack.push(self.pc);
                self.pc = addr;
            }
            Opcode::Return => {
                let value = &self.stack.pop();
                match value {
                    StackValue::Literal(_) => {
                        self.stack.push(value.to_owned());
                    }
                    StackValue::Addr(addr) => self
                        .stack
                        .push(StackValue::Literal(self.heap.get(*addr).to_owned())),
                }

                let addr = self.call_stack.pop().unwrap();
                self.exit_scope();

                self.pc = addr;
            }

            Opcode::Add => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs + rhs),
                    (Type::String(lhs), Type::String(rhs)) => Type::String(lhs + &rhs),
                    _ => panic!("Addition not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Sub => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs - rhs),
                    _ => panic!("Subtraction not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Mul => {
                let lhs = self.pop_stack();
                let rhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs * rhs),
                    _ => panic!("Multiplication not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Div => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs / rhs),
                    _ => panic!("Division not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Mod => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs % rhs),
                    _ => panic!("Modulo not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Eq => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs == rhs),
                    (Type::String(lhs), Type::String(rhs)) => Type::Bool(lhs == rhs),
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs == rhs),
                    _ => panic!("Equality not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Neq => {
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
            Opcode::Lt => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs < rhs),
                    _ => panic!("Less than not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Gt => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs > rhs),
                    _ => panic!("Greater than not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Lte => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs <= rhs),
                    _ => panic!("Less than or equal not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Gte => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs >= rhs),
                    _ => panic!("Greater than or equal not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::And => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs && rhs),
                    _ => panic!("And not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Or => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs || rhs),
                    _ => panic!("Or not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Not => {
                let value = self.pop_stack();

                let result = match value {
                    Type::Bool(value) => Type::Bool(!value),
                    _ => panic!("Not not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Neg => {
                let value = self.pop_stack();

                let result = match value {
                    Type::Number(value) => Type::Number(-value),
                    _ => panic!("Negation not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Pow => {
                let rhs = self.pop_stack();
                let lhs = self.pop_stack();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs.powf(rhs)),
                    _ => panic!("Power not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Opcode::Print => {
                let value = self.pop_stack();
                println!("PRINT: {:?}", value);
            }
            Opcode::Noop => {}
            _ => {
                println!("NOT HANDLED: {:?}", instruction);
            }
        }
    }
}
