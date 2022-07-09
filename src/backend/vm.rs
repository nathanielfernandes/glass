use std::{borrow::Cow, thread, time, usize};

use super::{
    instruction::{Instr, Type},
    memory::Memory,
    stack::{Stack, StackValue},
};

const FALSE: Type = Type::Bool(false);
const TRUE: Type = Type::Bool(true);

// native functions

pub struct VM {
    pub program: Vec<Instr>,
    pub pc: usize,

    pub stack: Stack,
    pub call_stack: Vec<(usize, usize)>,

    pub fp: usize,

    pub local_addrs: Vec<usize>,
    pub heap: Memory,
}

impl VM {
    pub fn new() -> VM {
        VM {
            program: vec![],
            pc: 0,

            stack: Stack::new(),

            local_addrs: vec![],
            call_stack: {
                let mut cs = Vec::with_capacity(1000);
                cs.push((0, 0));
                cs
            },

            fp: 0,
            heap: Memory::new(),
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

    pub fn peek_stack(&mut self) -> Option<&Type> {
        if let Some(value) = self.stack.peek() {
            return Some(match value {
                StackValue::Literal(value) => value,
                StackValue::Addr(addr) => self.heap.get(*addr),
            });
        }

        None
    }

    #[inline]
    pub fn pop_stack<'a>(&'a mut self) -> Cow<'a, Type> {
        match self.stack.pop() {
            StackValue::Literal(value) => Cow::Owned(value),
            StackValue::Addr(addr) => {
                let val = self.heap.get(addr);
                Cow::Borrowed(&*val)
            }
        }
    }

    #[inline]
    pub fn double_pop_stack<'a>(&'a mut self) -> (Cow<'a, Type>, Cow<'a, Type>) {
        (
            match self.stack.pop() {
                StackValue::Literal(value) => Cow::Owned(value),
                StackValue::Addr(addr) => Cow::Borrowed(self.heap.get(addr)),
            },
            match self.stack.pop() {
                StackValue::Literal(value) => Cow::Owned(value),
                StackValue::Addr(addr) => Cow::Borrowed(self.heap.get(addr)),
            },
        )
    }

    pub fn free_locals(&mut self, amnt: usize) {
        for _ in 0..amnt {
            self.heap.free(self.local_addrs.pop().unwrap());
        }
        self.heap.cleanup()
    }

    #[inline]
    pub fn enter_scope(&mut self, return_to: usize) {
        // self.scopes.push(Scope::new(return_to));
        self.call_stack.push((return_to, 0));
        self.fp += 1;

        // if self.fp > 1000 {
        //     panic!("Stack overflow");
        // }
    }

    #[inline]
    pub fn exit_scope(&mut self) -> usize {
        // let scope = self.scopes.pop().expect("Exited from empty scope");
        let (return_to, amnt) = self.call_stack.pop().expect("Exited from empty scope");
        // self.heap.free(amnt);

        self.free_locals(amnt);
        self.fp -= 1;
        return_to
    }

    #[inline]
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

            Instr::StoreAddr(addr) => {
                let addr = *addr;
                let value = self.pop_stack().into_owned();
                self.heap.set(addr, value);
            }
            Instr::StoreLocal(offset) => {
                let addr = *offset + self.fp;
                let value = self.pop_stack().into_owned();
                self.heap.set(addr, value);

                self.call_stack[self.fp].1 += 1;
                self.local_addrs.push(addr);
            }
            Instr::StoreGlobal(offset) => {
                let addr = *offset;
                let value = self.pop_stack().into_owned();
                self.heap.set(addr, value);
            }

            // Instr::LoadDeref(offset) => {
            //     let addr = *offset + self.fp;
            //     let value = self.heap.get(addr);
            //     self.stack.push(StackValue::Literal(value.clone()));
            // }
            Instr::LoadAddr(addr) => {
                self.stack.push(StackValue::Addr(*addr));
            }
            Instr::LoadLocal(offset) => {
                self.stack.push(StackValue::Addr(*offset + self.fp));
            }
            Instr::LoadGlobal(offset) => {
                self.stack.push(StackValue::Addr(*offset));
            }

            Instr::Jump(to) => {
                self.pc = *to;
            }
            Instr::JumpIf(to) => {
                let to = *to;
                let c_val = self.pop_stack();
                let value = c_val.as_ref();

                if value == &TRUE {
                    self.pc = to;
                }
            }
            Instr::JumpIfNot(to) => {
                let to = *to;
                let c_val = self.pop_stack();
                let value = c_val.as_ref();

                if value == &FALSE {
                    self.pc = to;
                }
            }

            Instr::Call => {
                let c_val = self.pop_stack();
                let top = c_val.as_ref();

                if let Type::FuncPtr(jump) = top {
                    let jump = *jump;
                    self.enter_scope(self.pc);
                    self.pc = jump;
                } else {
                    panic!("Call to non-function {:?}", top);
                }
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
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::Number(lhs), Type::Number(rhs)) => Type::Number(lhs + rhs),
                    (Type::String(lhs), rhs) => Type::String(lhs.to_owned() + &rhs.to_string()),
                    (lhs, Type::String(rhs)) => Type::String(lhs.to_string() + rhs),
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

                // println!("Eq {:?} {:?}", lhs, rhs);

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
            Instr::Index => {
                let (c1, c2) = self.double_pop_stack();
                let item = c1.as_ref();
                let index = c2.as_ref();

                let result = match (item, index) {
                    // (Type::List(list), Type::Number(index)) => {
                    //     let index = index as usize;
                    //     if index >= list.len() {
                    //         panic!("Index out of bounds");
                    //     }
                    //     list[index].clone()
                    // }
                    (Type::String(string), Type::Number(index)) => {
                        let index = *index as usize;
                        if index >= string.len() {
                            panic!("Index out of bounds");
                        }
                        Type::String(string.chars().nth(index).unwrap().to_string())
                    }
                    _ => panic!("Index not supported"),
                };
                self.stack.push(StackValue::Literal(result));
            }
            Instr::Join => {
                let (c1, c2) = self.double_pop_stack();
                let rhs = c1.as_ref();
                let lhs = c2.as_ref();

                let result = match (lhs, rhs) {
                    (Type::String(lhs), rhs) => Type::String(lhs.to_owned() + &rhs.to_string()),
                    (lhs, Type::String(rhs)) => Type::String(lhs.to_string() + rhs),
                    _ => panic!("Power not supported"),
                };

                self.stack.push(StackValue::Literal(result));
            }
            Instr::JoinMany(amnt) => {
                let mut res = None;

                for _ in 0..*amnt {
                    let c_val = self.pop_stack();
                    let value = c_val.as_ref();

                    if let Some(result) = res {
                        res = Some(match (result, value) {
                            (lhs, Type::String(value)) => Type::String(lhs.to_string() + value),
                            (Type::String(value), rhs) => Type::String(value + &rhs.to_string()),
                            _ => panic!("Join not supported"),
                        });
                    } else {
                        res = Some(value.to_owned());
                    }
                }
                self.stack
                    .push(StackValue::Literal(res.unwrap_or(Type::None)));
            }

            Instr::Print => {
                let c = self.pop_stack();
                let value = c.as_ref();

                match value {
                    Type::String(value) => println!("{}", value),
                    Type::Number(value) => println!("{}", value),
                    Type::Bool(value) => println!("{}", value),
                    _ => println!("{}", &value.to_string()),
                    // _ => panic!("Print not supported"),
                }
            }
            Instr::Noop => {}
            _ => {
                panic!("NOT HANDLED: {:?}", instruction);
            }
        }
    }
}
