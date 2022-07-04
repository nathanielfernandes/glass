use crate::frontend::{Expr, AST};
// use hashbrown::HashMap;
use fxhash::FxHashMap;
use std::fmt;

use super::{memory::addr, scope::id, stack::StackValue};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number(f64),
    String(String),
    Bool(bool),
    None,
    Null,

    Addr(usize),
    Ref(usize),

    Error(String),
}

#[derive(Clone, PartialEq)]
pub enum Opcode {
    Noop,

    Halt,

    Load(id),
    Store(id),
    Set(id),

    Register(id, addr),

    Push(StackValue),
    Pop,

    Jump(usize),
    JumpIf(usize),
    JumpIfNot(usize),
    Call(id),
    Return,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Neg,
    Not,
    And,
    Or,
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,

    Print,
}

pub type State = FxHashMap<String, usize>;

impl Opcode {
    pub fn compile(ast: AST) -> Vec<Opcode> {
        let mut program = vec![];
        let mut state = FxHashMap::default();
        let mut next = 0;
        for expr in ast {
            Opcode::build(&mut program, expr.clone(), &mut state, 0, &mut next);

            if let Some(op) = program.last() {
                match expr {
                    Expr::If(_, _, _) => {}
                    _ => {
                        if op.pushes_to_stack() {
                            program.push(Opcode::Pop);
                        }
                    }
                }
            }
        }

        let mut last = None;
        for (i, op) in program.clone().into_iter().enumerate() {
            if let Some(l) = last.clone() {
                match (l, op.clone()) {
                    (Opcode::Push(_), Opcode::Pop) => {
                        program[i - 1] = Opcode::Noop;
                        program[i] = Opcode::Noop;
                    }
                    (Opcode::Load(_), Opcode::Pop) => {
                        program[i - 1] = Opcode::Noop;
                        program[i] = Opcode::Noop;
                    }
                    _ => {}
                }
            }

            last = Some(op);
        }

        program
    }

    pub fn build(
        ins: &mut Vec<Opcode>,
        expr: Expr,
        state: &mut State,
        depth: usize,
        next: &mut usize,
    ) {
        macro_rules! op {
            ($op:expr) => {
                ins.push($op)
            };
        }
        macro_rules! push_literal {
            ($val:expr) => {
                ins.push(Self::Push(StackValue::Literal($val)))
            };
        }
        macro_rules! build {
            ($val:expr) => {
                Self::build(ins, $val, state, depth + 1, next)
            };
        }

        match expr {
            Expr::Number(num) => {
                push_literal!(Type::Number(num));
            }
            Expr::String(str) => {
                push_literal!(Type::String(str));
            }
            Expr::Bool(bool) => {
                push_literal!(Type::Bool(bool));
            }
            Expr::None => {
                push_literal!(Type::None);
            }
            Expr::Declaration(name, value) => {
                build!(*value);
                // Self::build(ins, *value, state, depth + 1, next, stack);
                // let id = get_id(&name) + depth;
                if let Some(id) = state.get(&name) {
                    op!(Self::Store(*id));
                } else {
                    let id = depth + *next;

                    *next += 1;
                    state.insert(name, id.clone());
                    op!(Self::Store(id));
                }
            }
            Expr::Assignment(name, value) => {
                let id = state.get(&name).expect("Variable not found").clone();
                // let id = get_id(&name);
                *next += 1;
                build!(*value);
                // Self::build(ins, *value, state, depth + 1, next, stack);
                op!(Self::Store(id));
            }
            Expr::Symbol(name) => {
                let id = state.get(&name).expect("Variable not found").clone();
                // let id = get_id(&name);
                op!(Self::Load(id));
            }

            Expr::Function(name, args, code) => {
                let top = ins.len();
                ins.push(Opcode::Noop); // placeholder for return address

                let depth = depth + 1;

                let id = depth - 1 + *next;
                // let id = get_id(&name) + depth - 1;

                *next += 1;
                state.insert(name, id.clone());

                let mut fn_state = state.clone();

                let mut arg_ids = vec![];
                for arg in args {
                    let id = depth + *next;
                    // let id = get_id(&arg) + depth;
                    *next += 1;
                    fn_state.insert(arg, id.clone());
                    arg_ids.push(id);
                }

                for arg in arg_ids {
                    op!(Self::Store(arg));
                }

                for expr in code {
                    Self::build(ins, expr, &mut fn_state, depth, next);
                }

                push_literal!(Type::None);
                op!(Self::Return);

                ins[top] = Self::Jump(ins.len());

                // push_literal!(Type::Ref(top + 1));

                op!(Self::Register(id, top + 1));
            }
            Expr::If(condition, then, otherwise) => {
                match *condition {
                    Expr::Bool(true) => {
                        for expr in then {
                            Self::build(ins, expr, state, depth, next);
                        }
                    }
                    Expr::Bool(false) => {
                        for expr in otherwise {
                            Self::build(ins, expr, state, depth, next);
                        }
                    }
                    Expr::Or(lhs, rhs) => {
                        Self::build(ins, *lhs, state, depth, next);
                        let jump_if_idx = ins.len();
                        ins.push(Self::Noop);

                        Self::build(ins, *rhs, state, depth, next);
                        let jump_if_not_idx = ins.len();
                        ins.push(Self::Noop);

                        let then_jump_to = ins.len();
                        for expr in then {
                            Self::build(ins, expr, state, depth, next);
                        }
                        let jump_idx = ins.len();
                        ins.push(Self::Noop); // placeholder for Jump
                        let jump_to = ins.len();

                        for expr in otherwise {
                            Self::build(ins, expr, state, depth, next);
                        }

                        ins[jump_if_idx] = Self::JumpIf(then_jump_to);
                        ins[jump_if_not_idx] = Self::JumpIfNot(jump_to);
                        ins[jump_idx] = Self::Jump(ins.len());
                    }
                    Expr::And(lhs, rhs) => {
                        Self::build(ins, *lhs, state, depth, next);
                        let jump_if_idx = ins.len();
                        ins.push(Self::Noop);

                        Self::build(ins, *rhs, state, depth, next);
                        let jump_if_not_idx = ins.len();
                        ins.push(Self::Noop);

                        let then_jump_to = ins.len();
                        for expr in then {
                            Self::build(ins, expr, state, depth, next);
                        }
                        let jump_idx = ins.len();
                        ins.push(Self::Noop); // placeholder for Jump
                        let jump_to = ins.len();

                        for expr in otherwise {
                            Self::build(ins, expr, state, depth, next);
                        }

                        ins[jump_if_idx] = Self::JumpIfNot(then_jump_to);
                        ins[jump_if_not_idx] = Self::JumpIfNot(jump_to);
                        ins[jump_idx] = Self::Jump(ins.len());
                    }
                    _ => {
                        Self::build(ins, *condition, state, depth, next);

                        let jump_if_not_idx = ins.len();
                        ins.push(Self::Noop); // placeholder for JumpIfNot

                        for expr in then {
                            Self::build(ins, expr, state, depth, next);
                        }

                        let jump_idx = ins.len();
                        ins.push(Self::Noop); // placeholder for Jump
                        let jump_to = ins.len();

                        for expr in otherwise {
                            Self::build(ins, expr, state, depth, next);
                        }

                        ins[jump_if_not_idx] = Self::JumpIfNot(jump_to);
                        ins[jump_idx] = Self::Jump(ins.len());
                    }
                }
            }
            Expr::Call(name, args) => {
                let id = state.get(&name).expect("Function not found").clone();

                for arg in args {
                    build!(arg);
                }
                op!(Self::Call(id));
            }
            Expr::Return(expr) => {
                // build!(*expr);

                Self::build(ins, *expr, state, depth - 1, next);
                op!(Self::Return);
            }
            Expr::Add(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Add);
            }
            Expr::Sub(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Sub);
            }
            Expr::Mul(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Mul);
            }
            Expr::Div(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Div);
            }
            Expr::Mod(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Mod);
            }
            Expr::Pow(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Pow);
            }
            Expr::Eq(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Eq);
            }
            Expr::Neq(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Neq);
            }
            Expr::Lt(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Lt);
            }
            Expr::Gt(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Gt);
            }
            Expr::Lte(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Lte);
            }
            Expr::Gte(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Gte);
            }
            Expr::And(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::And);
            }
            Expr::Or(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                op!(Opcode::Or);
            }
            Expr::Not(expr) => {
                build!(*expr);
                op!(Opcode::Not);
            }
            Expr::Neg(expr) => {
                build!(*expr);
                op!(Opcode::Neg);
            }
            _ => {
                panic!("Not implemented");
            }
        }
    }

    pub fn pushes_to_stack(&self) -> bool {
        match self {
            Self::Halt => false,
            Self::Jump(_) => false,
            Self::Pop => false,
            Self::Print => false,
            Self::Store(_) => false,
            Self::Register(_, _) => false,
            Self::Call(_) => false,
            Self::Return => false,

            _ => true,
        }
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noop => write!(f, "Noop"),
            Self::Halt => write!(f, "Halt"),
            Self::Load(id) => write!(f, "Load    \t{}", id),
            Self::Store(id) => write!(f, "Store    \t{}", id),
            Self::Register(id, addr) => write!(f, "Register\t{} {addr}", id),
            Self::Set(id) => write!(f, "Set\t{}", id),

            Self::Push(arg) => write!(f, "Push    \t{:?}", arg),
            Self::Pop => write!(f, "Pop"),
            Self::Jump(id) => write!(f, "Jump    \t{}", id),
            Self::JumpIf(id) => write!(f, "JumpIf  \t{}", id),
            Self::JumpIfNot(id) => write!(f, "JumpIfNot\t{}", id),
            Self::Call(id) => write!(f, "Call    \t{}", id),
            Self::Return => write!(f, "Return"),

            Self::Print => write!(f, "Print"),

            Self::Add => write!(f, "Add"),
            Self::Sub => write!(f, "Sub"),
            Self::Mul => write!(f, "Mul"),
            Self::Div => write!(f, "Div"),
            Self::Mod => write!(f, "Mod"),
            Self::Pow => write!(f, "Pow"),
            Self::Eq => write!(f, "Eq"),
            Self::Neq => write!(f, "Neq"),
            Self::Lt => write!(f, "Lt"),
            Self::Gt => write!(f, "Gt"),
            Self::Lte => write!(f, "Lte"),
            Self::Gte => write!(f, "Gte"),
            Self::And => write!(f, "And"),
            Self::Or => write!(f, "Or"),
            Self::Not => write!(f, "Not"),
            Self::Neg => write!(f, "Neg"),
        }
    }
}
