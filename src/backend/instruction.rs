use crate::{
    frontend::{Expr, Op, AST},
    stdlib::{add_std, NativeFunction},
};
// use hashbrown::HashMap;
use fxhash::FxHashMap;
use std::fmt;

use super::{memory::addr, stack::StackValue};

#[derive(Clone, PartialEq)]
pub enum Type {
    Number(f64),
    String(String),
    Bool(bool),
    None,

    Null,

    Addr(usize),
    FuncPtr(usize),

    Error(String),
}

impl Type {
    pub fn to_string(&self) -> String {
        match self {
            Type::Number(n) => n.to_string(),
            Type::String(s) => s.clone(),
            Type::Bool(b) => b.to_string(),
            Type::None => "none".to_string(),
            Type::Null => "null".to_string(),
            Type::Addr(a) => format!("<addr={}>", a),
            Type::FuncPtr(f) => format!("<function at={}>", f),
            Type::Error(e) => format!("Error({})", e),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Number(n) => write!(f, "num({})", n),
            Type::String(s) => write!(f, "str({:?})", s),
            Type::Bool(b) => write!(f, "bool({})", b),
            Type::None => write!(f, "none"),
            Type::Null => write!(f, "null"),
            Type::Addr(addr) => write!(f, "#{}", addr),
            Type::FuncPtr(addr) => write!(f, "fn(@{})", addr),
            Type::Error(s) => write!(f, "Error({})", s),
        }
    }
}

#[allow(non_camel_case_types)]
type offset = usize;

#[derive(Clone, PartialEq)]
pub enum Instr {
    Noop,

    Halt,

    // Load(offset),
    // LoadDeref(offset),
    LoadAddr(addr),
    LoadLocal(offset),
    LoadGlobal(offset),

    // Store(offset),
    StoreAddr(addr),
    StoreLocal(offset),
    StoreGlobal(offset),

    // Register(offset, addr),
    Push(StackValue),
    Pop,

    Jump(usize),
    JumpIf(usize),
    JumpIfNot(usize),
    Call,
    NativeCall(NativeFunction),
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

    Index,
    IndexStore,
    Join,
    JoinMany(usize),
}

pub type State = FxHashMap<String, (usize, usize)>;

impl Instr {
    pub fn compile(ast: AST) -> (Vec<Instr>, usize) {
        let mut program = vec![];
        let mut state = FxHashMap::default();
        let mut next = 0;

        // add_std(&mut program, &mut state, 0, &mut next);
        add_std(&mut program, &mut state, 0, &mut next);
        let prog_start = program.len();
        Self::iter_build(&mut program, ast, &mut state, 0, &mut next);

        let mut last = None;
        for (i, op) in program.clone().into_iter().enumerate() {
            if let Some(l) = last.clone() {
                match (l, op.clone()) {
                    (Instr::Push(_), Instr::Pop) => {
                        program[i - 1] = Instr::Noop;
                        program[i] = Instr::Noop;
                    }
                    // (Instr::LoadDeref(_), Instr::Pop) => {
                    //     program[i - 1] = Instr::Noop;
                    //     program[i] = Instr::Noop;
                    // }
                    (Instr::LoadAddr(_), Instr::Pop) => {
                        program[i - 1] = Instr::Noop;
                        program[i] = Instr::Noop;
                    }
                    (Instr::LoadLocal(_), Instr::Pop) => {
                        program[i - 1] = Instr::Noop;
                        program[i] = Instr::Noop;
                    }
                    (Instr::LoadGlobal(_), Instr::Pop) => {
                        program[i - 1] = Instr::Noop;
                        program[i] = Instr::Noop;
                    }
                    _ => {}
                }
            }

            last = Some(op);
        }

        (program, prog_start)
    }

    pub fn iter_build(
        ins: &mut Vec<Instr>,
        code: Vec<Expr>,
        state: &mut State,
        depth: usize,
        next: &mut usize,
    ) {
        for expr in code {
            Self::build(ins, expr.clone(), state, depth, next);

            if let Some(op) = ins.last() {
                match expr {
                    Expr::If {
                        condition: _,
                        then: _,
                        otherwise: _,
                    } => {}
                    _ => {
                        if op.pushes_to_stack() {
                            ins.push(Instr::Pop);
                        }
                    }
                }
            }
        }
    }

    pub fn build(
        ins: &mut Vec<Instr>,
        expr: Expr,
        state: &mut State,
        depth: usize,
        next: &mut usize,
    ) {
        macro_rules! ins {
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
                Self::build(ins, $val, state, depth, next)
            };
            ($val:expr, $incr:expr) => {
                Self::build(ins, $val, state, $incr, next)
            };
        }

        macro_rules! load {
            ($id:expr, $d:expr) => {
                if $d == 0 {
                    ins.push(Self::LoadGlobal($id))
                } else if $d == depth {
                    ins.push(Self::LoadLocal($id))
                } else {
                    ins.push(Self::LoadAddr($id + $d))
                }
            };
        }

        macro_rules! declare {
            ($id:expr, $d:expr) => {
                if $d == 0 {
                    ins.push(Self::StoreGlobal($id))
                } else if $d == depth {
                    ins.push(Self::StoreLocal($id))
                } else {
                    ins.push(Self::StoreAddr($id + $d))
                }
            };
        }

        macro_rules! assign {
            ($id:expr, $d:expr) => {
                // if $d < depth {
                //     return;
                // }

                if $d == 0 {
                    ins.push(Self::StoreGlobal($id))
                } else if $d == depth {
                    ins.push(Self::StoreLocal($id))
                } else {
                    ins.push(Self::StoreAddr($id + $d))
                }
            };
        }

        match expr {
            Expr::Number(num) => {
                push_literal!(Type::Number(num));
            }
            Expr::String(str) => {
                push_literal!(Type::String(str));
            }
            Expr::FormatString(string) => {
                let n = string.len();

                if n == 0 {
                    push_literal!(Type::String("".to_string()));
                } else if n == 1 {
                    build!(string[0].clone());
                // } else if n == 2 {
                // build!(string[0].clone());
                // build!(string[1].clone());
                // ins!(Self::Add);
                } else {
                    for expr in string.iter().rev() {
                        build!(expr.clone());
                    }
                    ins.push(Self::JoinMany(n));
                }
            }
            Expr::Join(lhs, rhs) => {
                build!(*lhs);
                build!(*rhs);
                ins!(Self::Join);
            }
            Expr::Bool(bool) => {
                push_literal!(Type::Bool(bool));
            }
            Expr::None => {
                push_literal!(Type::None);
            }
            Expr::Index { item, index } => {
                // match item

                // if let Expr::String(s) = *item {
                //     if let Expr::Number(n) = *index {
                //         build!(Expr::String(s.chars().nth(n as usize).unwrap().to_string()))
                //     }
                // } else {

                // }

                build!(*index);
                build!(*item);
                ins!(Self::Index);
            }
            Expr::Declaration(name, value) => {
                build!(*value);
                // Self::build(ins, *value, state, depth + 1, next, stack);
                // let id = get_id(&name) + depth;

                // if let Some((id, dep)) = state.get(&name) {
                //     if *dep == depth {
                //         declare!(*id, *dep);
                //     } else {
                //         let id = depth + *next;

                //         *next += 1;
                //         state.insert(name, (id.clone(), depth));
                //         // op!(Self::Store(id));
                //         declare!(id, depth);
                //     }
                // } else {
                let id = depth + *next;

                *next += 1;
                state.insert(name, (id.clone(), depth));
                // op!(Self::Store(id));
                declare!(id, depth);
                // }
            }
            Expr::Assignment(assignee, value) => {
                match *assignee {
                    Expr::Identifier(name) => {
                        let (id, dep) = state
                            .get(&name)
                            .expect(&format!("Variable not found {}", name))
                            .clone();
                        // let id = get_id(&name);
                        *next += 1;
                        build!(*value);
                        // Self::build(ins, *value, state, depth + 1, next, stack);
                        // op!(Self::Store(id));
                        assign!(id, dep);
                    }
                    // Expr::Index { item, index } => {
                    //     build!(*index);
                    //     build!(*value);
                    //     build!(*item);
                    //     ins!(Self::IndexStore);
                    //     // match *item => {
                    //     //     Expr::Identifier(name) => {

                    //     //     }
                    //     // }
                    // }
                    // Expr::Call(_, _) => {}
                    _ => panic!("cannot assign"),
                }
            }
            // Expr::Walrus(name, value) => {
            //     build!(*value);
            //     // Self::build(ins, *value, state, depth + 1, next, stack);
            //     // let id = get_id(&name) + depth;
            //     if let Some((id, dep)) = state.get(&name) {
            //         // op!(Self::Store(*id));
            //         store!(*id, *dep);
            //     } else {
            //         let id = depth + *next;

            //         *next += 1;
            //         state.insert(name, (id.clone(), depth));
            //         // op!(Self::Store(id));
            //         store!(id, depth);
            //     }
            // }
            Expr::Identifier(name) => {
                let (id, dep) = state
                    .get(&name)
                    .expect(&format!("Variable not found {}", name))
                    .clone();
                // let id = get_id(&name);
                // op!(Self::Load(id));
                // load!(id, d);

                // if dep != 0 && dep < depth {
                //     ins!(Self::LoadAddr(id + dep))
                // } else {
                // load!(id, dep);
                // if dep != 0 && dep < depth {
                //     ins!(Self::LoadAddr(id + dep))
                // } else {
                load!(id, dep);
                // }

                // }
            }
            Expr::Lambda(args, code) => {
                let top = ins.len();
                ins.push(Instr::Noop); // placeholder for return address

                // let depth = depth + 1;
                let mut new_next = *next;
                // let id: usize = depth + *next;
                *next += 1;
                // state.insert(name, (id.clone(), depth));

                let mut fn_state = state.clone();

                for arg in args {
                    let id = depth + 1 + new_next;
                    new_next += 1;
                    fn_state.insert(arg, (id.clone(), depth + 1));

                    ins!(Self::StoreLocal(id));
                }

                Self::iter_build(ins, code, &mut fn_state, depth + 1, &mut new_next);

                push_literal!(Type::None);
                ins!(Self::Return);

                ins[top] = Self::Jump(ins.len());

                push_literal!(Type::FuncPtr(top + 1));
                // declare!(id, depth);
            }
            Expr::Function {
                name,
                args,
                body: code,
            } => {
                let top = ins.len();
                ins.push(Instr::Noop); // placeholder for return address

                // let depth = depth + 1;
                let mut new_next = *next;
                let id: usize = if let Some((id, _dep)) = state.get(&name) {
                    // if depth == *dep {
                    *id
                    // } else {
                    //     depth + *next
                    // }
                } else {
                    depth + *next
                };

                // let id =
                // let id = get_id(&name) + depth - 1;
                *next += 1;
                state.insert(name, (id.clone(), depth));

                let mut fn_state = state.clone();

                // println!("{} {:?}", depth, fn_state);

                // let mut arg_ids = vec![];
                for arg in args {
                    let id = depth + 1 + new_next;
                    // let id = get_id(&arg) + depth;
                    new_next += 1;
                    fn_state.insert(arg, (id.clone(), depth + 1));

                    // arg_ids.push(id);

                    ins!(Self::StoreLocal(id));
                }

                // println!("{} {:?}", depth + 1, fn_state);

                // for arg in arg_ids {
                //     // op!(Self::Store(arg));
                //     // store!(arg, depth + 1);
                //     ins!(Self::StoreLocal(arg));
                // }

                // for expr in code {
                //     Self::build(ins, expr, &mut fn_state, depth, next);
                // }
                Self::iter_build(ins, code, &mut fn_state, depth + 1, &mut new_next);

                push_literal!(Type::None);
                ins!(Self::Return);

                ins[top] = Self::Jump(ins.len());

                push_literal!(Type::FuncPtr(top + 1));
                // op!(Self::Store(id));
                declare!(id, depth);

                // op!(Self::Register(id, top + 1));
            }
            Expr::If {
                condition,
                then,
                otherwise,
            } => {
                match *condition {
                    Expr::Bool(true) => {
                        // for expr in then {
                        //     Self::build(ins, expr, state, depth, next);
                        // }
                        Self::iter_build(ins, then, state, depth, next);
                    }
                    Expr::Bool(false) => {
                        // for expr in otherwise {
                        //     Self::build(ins, expr, state, depth, next);
                        // }
                        Self::iter_build(ins, otherwise, state, depth, next);
                    }
                    Expr::Op(Op::Or, lhs, rhs) => {
                        Self::build(ins, *lhs, state, depth, next);
                        let jump_if_idx = ins.len();
                        ins.push(Self::Noop);

                        Self::build(ins, *rhs, state, depth, next);
                        let jump_if_not_idx = ins.len();
                        ins.push(Self::Noop);

                        let then_jump_to = ins.len();
                        // for expr in then {
                        //     Self::build(ins, expr, state, depth, next);
                        // }
                        Self::iter_build(ins, then, state, depth, next);

                        let jump_idx = ins.len();
                        ins.push(Self::Noop); // placeholder for Jump
                        let jump_to = ins.len();

                        // for expr in otherwise {
                        //     Self::build(ins, expr, state, depth, next);
                        // }

                        Self::iter_build(ins, otherwise, state, depth, next);

                        ins[jump_if_idx] = Self::JumpIf(then_jump_to);
                        ins[jump_if_not_idx] = Self::JumpIfNot(jump_to);
                        ins[jump_idx] = Self::Jump(ins.len());
                    }
                    Expr::Op(Op::And, lhs, rhs) => {
                        Self::build(ins, *lhs, state, depth, next);
                        let jump_if_idx = ins.len();
                        ins.push(Self::Noop);

                        Self::build(ins, *rhs, state, depth, next);
                        let jump_if_not_idx = ins.len();
                        ins.push(Self::Noop);

                        let then_jump_to = ins.len();
                        // for expr in then {
                        //     Self::build(ins, expr, state, depth, next);
                        // }
                        Self::iter_build(ins, then, state, depth, next);

                        let jump_idx = ins.len();
                        ins.push(Self::Noop); // placeholder for Jump
                        let jump_to = ins.len();

                        // for expr in otherwise {
                        //     Self::build(ins, expr, state, depth, next);
                        // }
                        Self::iter_build(ins, otherwise, state, depth, next);

                        ins[jump_if_idx] = Self::JumpIfNot(then_jump_to);
                        ins[jump_if_not_idx] = Self::JumpIfNot(jump_to);
                        ins[jump_idx] = Self::Jump(ins.len());
                    }
                    _ => {
                        Self::build(ins, *condition, state, depth, next);

                        let jump_if_not_idx = ins.len();
                        ins.push(Self::Noop); // placeholder for JumpIfNot

                        // for expr in then {
                        //     Self::build(ins, expr, state, depth, next);
                        // }
                        Self::iter_build(ins, then, state, depth, next);

                        let jump_idx = ins.len();
                        ins.push(Self::Noop); // placeholder for Jump
                        let jump_to = ins.len();

                        // for expr in otherwise {
                        //     Self::build(ins, expr, state, depth, next);
                        // }
                        Self::iter_build(ins, otherwise, state, depth, next);

                        ins[jump_if_not_idx] = Self::JumpIfNot(jump_to);
                        ins[jump_idx] = Self::Jump(ins.len());
                    }
                }
            }
            Expr::Call(name, args) => {
                // if let Expr::Identifier(name) = *name {
                //     let (id, dep) = state.get(&name).expect("Function not found").clone();
                //     // let return_addr = ins.len();
                //     // ins!(Self::Noop);
                //     for arg in args.into_iter().rev() {
                //         build!(arg);
                //     }
                //     // println!("{} {:?}", depth, state);
                //     // if dep != 0 && dep < depth {
                //     //     ins!(Self::LoadAddr(id + dep))
                //     // } else {
                //     load!(id, dep);
                //     // }
                //     // load!(id, dep);
                //     ins!(Self::Call);
                // } else {
                for arg in args.into_iter().rev() {
                    build!(arg);
                }

                build!(*name);
                ins!(Self::Call);
                // }
            }
            Expr::NativeCall(name, args) => {
                for arg in args.into_iter().rev() {
                    build!(arg);
                }

                if let Some(nf) = NativeFunction::from(&name) {
                    ins!(Self::NativeCall(nf));
                } else {
                    panic!("Native function not found: {}", name);
                }
            }
            Expr::Return(expr) => {
                // build!(*expr);

                Self::build(ins, *expr, state, depth, next);
                ins!(Self::Return);
            }
            Expr::Op(op, lhs, rhs) => {
                Self::build(ins, *lhs, state, depth, next);
                Self::build(ins, *rhs, state, depth, next);
                match op {
                    Op::Add => ins!(Self::Add),
                    Op::Sub => ins!(Self::Sub),
                    Op::Mul => ins!(Self::Mul),
                    Op::Div => ins!(Self::Div),
                    Op::Mod => ins!(Self::Mod),
                    Op::Eq => ins!(Self::Eq),
                    Op::Neq => ins!(Self::Neq),
                    Op::Lt => ins!(Self::Lt),
                    Op::Gt => ins!(Self::Gt),
                    Op::Lte => ins!(Self::Lte),
                    Op::Gte => ins!(Self::Gte),
                    Op::Or => ins!(Self::Or),
                    Op::And => ins!(Self::And),
                    Op::Not => ins!(Self::Not),
                    Op::Neg => ins!(Self::Neg),
                    Op::Pow => ins!(Self::Pow),
                }
            } // match (*lhs, *rhs) {
            //     (Expr::Number(lhs), Expr::Number(rhs)) => match op {
            //         Op::Add => build!(Expr::Number(lhs + rhs)),
            //         Op::Sub => build!(Expr::Number(lhs - rhs)),
            //         Op::Mul => build!(Expr::Number(lhs * rhs)),
            //         Op::Div => build!(Expr::Number(lhs / rhs)),
            //         Op::Mod => build!(Expr::Number(lhs % rhs)),
            //         Op::Pow => build!(Expr::Number(lhs.powf(rhs))),
            //         Op::Eq => build!(Expr::Bool(lhs == rhs)),
            //         Op::Neq => build!(Expr::Bool(lhs != rhs)),
            //         Op::Lt => build!(Expr::Bool(lhs < rhs)),
            //         Op::Gt => build!(Expr::Bool(lhs > rhs)),
            //         Op::Lte => build!(Expr::Bool(lhs <= rhs)),
            //         Op::Gte => build!(Expr::Bool(lhs >= rhs)),
            //         Op::Neg => build!(Expr::Number(-lhs)),
            //         _ => panic!("cannot perform op"),
            //     },
            //     (Expr::Bool(lhs), Expr::Bool(rhs)) => match op {
            //         Op::Eq => build!(Expr::Bool(lhs == rhs)),
            //         Op::Neq => build!(Expr::Bool(lhs != rhs)),
            //         Op::And => build!(Expr::Bool(lhs && rhs)),
            //         Op::Or => build!(Expr::Bool(lhs || rhs)),
            //         _ => panic!("cannot perform op"),
            //     },
            //     (lhs, rhs) => {
            //         Self::build(ins, lhs, state, depth, next);
            //         Self::build(ins, rhs, state, depth, next);
            //         match op {
            //             Op::Add => ins!(Self::Add),
            //             Op::Sub => ins!(Self::Sub),
            //             Op::Mul => ins!(Self::Mul),
            //             Op::Div => ins!(Self::Div),
            //             Op::Mod => ins!(Self::Mod),
            //             Op::Eq => ins!(Self::Eq),
            //             Op::Neq => ins!(Self::Neq),
            //             Op::Lt => ins!(Self::Lt),
            //             Op::Gt => ins!(Self::Gt),
            //             Op::Lte => ins!(Self::Lte),
            //             Op::Gte => ins!(Self::Gte),
            //             Op::Or => ins!(Self::Or),
            //             Op::And => ins!(Self::And),
            //             Op::Not => ins!(Self::Not),
            //             Op::Neg => ins!(Self::Neg),
            //             Op::Pow => ins!(Self::Pow),
            //         }
            //     }
            // },
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
            // Self::Store(_) => false,
            Self::StoreAddr(_) => false,
            Self::StoreGlobal(_) => false,
            Self::StoreLocal(_) => false,
            // Self::Register(_, _) => false,
            // Self::Call => false,
            Self::IndexStore => false,
            Self::Return => false,
            Self::JumpIfNot(_) => false,
            Self::JumpIf(_) => false,
            Self::Noop => false,

            _ => true,
        }
    }
}

impl fmt::Debug for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noop => write!(f, "Noop"),
            Self::Halt => write!(f, "Halt"),

            // Self::Load(id) => write!(f, "Load       \t{}", id),
            // Self::LoadDeref(id) => write!(f, "LoadDeref\t{}", id),
            Self::LoadAddr(id) => write!(f, "LoadAddr   \t{}", id),
            Self::LoadLocal(id) => write!(f, "LoadLocal\t{}", id),
            Self::LoadGlobal(id) => write!(f, "LoadGlobal\t{}", id),

            // Self::Store(id) => write!(f, "Store    \t{}", id),
            Self::StoreAddr(id) => write!(f, "StoreAddr   \t{}", id),
            Self::StoreLocal(id) => write!(f, "StoreLocal\t{}", id),
            Self::StoreGlobal(id) => write!(f, "StoreGlobal\t{}", id),

            // Self::Register(id, addr) => write!(f, "Register\t{} {addr}", id),
            Self::Push(arg) => write!(f, "Push    \t{:?}", arg),
            Self::Pop => write!(f, "Pop           "),
            Self::Jump(id) => write!(f, "Jump    \t{}", id),
            Self::JumpIf(id) => write!(f, "JumpIf  \t{}", id),
            Self::JumpIfNot(id) => write!(f, "JumpIfNot\t{}", id),
            Self::Call => write!(f, "Call              "),
            Self::NativeCall(id) => write!(f, "NativeCall\t{:?}", id),
            Self::Return => write!(f, "Return           "),

            Self::JoinMany(amnt) => write!(f, "JoinMany\t{}", amnt),
            Self::Join => write!(f, "Join           "),
            Self::Index => write!(f, "Index          "),
            Self::IndexStore => write!(f, "IndexStore          "),

            Self::Add => write!(f, "Add              "),
            Self::Sub => write!(f, "Sub              "),
            Self::Mul => write!(f, "Mul              "),
            Self::Div => write!(f, "Div              "),
            Self::Mod => write!(f, "Mod              "),
            Self::Pow => write!(f, "Pow              "),
            Self::Eq => write!(f, "Eq              "),
            Self::Neq => write!(f, "Neq              "),
            Self::Lt => write!(f, "Lt              "),
            Self::Gt => write!(f, "Gt              "),
            Self::Lte => write!(f, "Lte              "),
            Self::Gte => write!(f, "Gte              "),
            Self::And => write!(f, "And              "),
            Self::Or => write!(f, "Or              "),
            Self::Not => write!(f, "Not              "),
            Self::Neg => write!(f, "Neg              "),
        }
    }
}
