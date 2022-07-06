use std::{borrow::Cow, thread, time, usize};

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

    pub fn debug(&mut self) {
        let delay = time::Duration::from_millis(20);

        while self.pc < self.program.len() {
            let ins = self.program[self.pc].clone();

            if ins != Instr::Noop {
                let i = self.pc;
                let top = self.peek_stack();
                if let Some(top) = top {
                    println!("{}:\t{:?}\t <- {:?}", i, ins, top);
                } else {
                    println!("{}:\t{:?}", i, ins);
                }
                thread::sleep(delay);
            }

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

    pub fn peek_stack(&mut self) -> Option<&Type> {
        if let Some(value) = self.stack.peek() {
            return Some(match value {
                StackValue::Literal(value) => value,
                StackValue::Addr(addr) => self.heap.get(*addr),
            });
        }

        None
    }

    pub fn pop_stack<'a>(&'a mut self) -> Cow<'a, Type> {
        let value = self.stack.pop();

        match value {
            StackValue::Literal(value) => Cow::Owned(value),
            StackValue::Addr(addr) => {
                let val = self.heap.get(addr);
                Cow::Borrowed(&*val)
            }
        }
    }

    pub fn double_pop_stack<'a>(&'a mut self) -> (Cow<'a, Type>, Cow<'a, Type>) {
        let value1 = self.stack.pop();
        let value2 = self.stack.pop();

        (
            match value1 {
                StackValue::Literal(value) => Cow::Owned(value),
                StackValue::Addr(addr) => Cow::Borrowed(self.heap.get(addr)),
            },
            match value2 {
                StackValue::Literal(value) => Cow::Owned(value),
                StackValue::Addr(addr) => Cow::Borrowed(self.heap.get(addr)),
            },
        )
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

        match instruction {
            Instr::Halt => {
                println!("Halt");
                return;
            }
            Instr::Push(value) => {
                self.stack.push(value.clone());
            }
            Instr::Pop => {
                self.stack.pop();
            }
            Instr::Store(id) => {
                let id = *id + self.fp;
                let value = self.pop_stack().into_owned();

                if let Some(addr) = self.all_scopes_get(id) {
                    self.heap.set(addr, value);
                } else {
                    let addr = self.heap.add(value);
                    self.scopes[self.fp].set(id, addr);
                }
            }
            Instr::StoreLocal(id) => {
                let id = *id + self.fp;
                let value = self.pop_stack().into_owned();

                if let Some(addr) = self.scopes[self.fp].get(id) {
                    self.heap.set(addr, value);
                } else {
                    let addr = self.heap.add(value);
                    self.scopes[self.fp].set(id, addr);
                }
            }
            Instr::StoreGlobal(id) => {
                let id = *id;
                let value = self.pop_stack().into_owned();

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
                self.stack.push(StackValue::Addr(addr));
            }
            Instr::LoadGlobal(id) => {
                let addr = self.scopes[0].get(*id).expect("Undefined variable");
                self.stack.push(StackValue::Addr(addr));
            }
            Instr::LoadLocal(id) => {
                let id = *id + self.fp;
                let addr = self.scopes[self.fp].get(id).expect("Undefined variable");
                self.stack.push(StackValue::Addr(addr));
            }
            Instr::LoadName(id) => {
                let id = *id + self.fp;
                let addr = self.all_scopes_get_desc(id).expect("Undefined variable");

                // let value = self.heap.get(addr);
                // self.stack.push(StackValue::Literal(value.clone()));
                self.stack.push(StackValue::Addr(addr));
            }
            Instr::Jump(to) => {
                self.pc = *to;
            }
            Instr::JumpIf(to) => {
                const TRUE: Type = Type::Bool(true);

                let to = *to;
                let c_val = self.pop_stack();
                let value = c_val.as_ref();

                if value == &TRUE
                // && value != &Type::Number(0.0)
                // && value != &Type::String("".to_owned())
                {
                    self.pc = to;
                }
            }
            Instr::JumpIfNot(to) => {
                const FALSE: Type = Type::Bool(false);

                let to = *to;
                let c_val = self.pop_stack();
                let value = c_val.as_ref();

                if value == &FALSE
                // || value == &Type::Number(0.0)
                // || value == &Type::String("".to_owned())
                {
                    self.pc = to;
                }
            }

            Instr::Call => {
                // let addr = self.all_scopes_get(id).expect("Call to undefined function");
                let c_val = self.pop_stack();
                let top = c_val.as_ref();

                if let Type::FuncPtr(jump) = top {
                    let jump = *jump;
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
                // let (c1, c2) = self.double_pop_stack();
                // let value = c1.to_owned();
                // let addr = c2.as_ref();

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
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs + rhs),
                    // (Type::String(lhs), Type::String(rhs)) => Type::String(lhs + rhs),
                    _ => panic!("Addition not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Sub => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs - rhs),
                    _ => panic!("Subtraction not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Mul => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs * rhs),
                    _ => panic!("Multiplication not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Div => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs / rhs),
                    _ => panic!("Division not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Mod => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs % rhs),
                    _ => panic!("Modulo not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Eq => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (&lhs, &rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs == rhs),
                    (Type::String(lhs), Type::String(rhs)) => Type::Bool(lhs == rhs),
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs == rhs),
                    _ => Type::Bool(lhs == rhs),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Neq => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs != rhs),
                    (Type::String(lhs), Type::String(rhs)) => Type::Bool(lhs != rhs),
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(lhs != rhs),
                    _ => panic!("Inequality not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Lt => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs < rhs),
                    _ => panic!("Less than not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Gt => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs > rhs),
                    _ => panic!("Greater than not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Lte => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs <= rhs),
                    _ => panic!("Less than or equal not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Gte => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Bool(lhs >= rhs),
                    _ => panic!("Greater than or equal not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::And => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(*lhs && *rhs),
                    _ => panic!("And not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Or => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Bool(lhs), Type::Bool(rhs)) => Type::Bool(*lhs || *rhs),
                    _ => panic!("Or not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Not => {
                let c_val = self.pop_stack();
                let value = c_val.as_ref();

                let result = match value {
                    Type::Bool(value) => Type::Bool(!value),
                    _ => panic!("Not not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Neg => {
                let c_val = self.pop_stack();
                let value = c_val.as_ref();

                let result = match value {
                    Type::Number(value) => Type::Number(-value),
                    _ => panic!("Negation not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Pow => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs.powf(*rhs)),
                    _ => panic!("Power not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::Print => {
                let c = self.pop_stack();
                let value = c.as_ref();

                match value {
                    Type::String(value) => println!("{}", value),
                    Type::Number(value) => println!("{}", value),
                    Type::Bool(value) => println!("{}", value),
                    _ => println!("{:?}", value),
                    // _ => panic!("Print not supported"),
                }
            }
            Instr::Noop => {}
            _ => {
                println!("NOT HANDLED: {:?}", instruction);
            }
        }
    }
}
