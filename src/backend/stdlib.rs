use super::{
    instruction::{Instr, State, Type},
    stack::StackValue,
};

fn get_id(name: &str, state: &State, depth: usize, next: usize) -> usize {
    if let Some((id, _)) = state.get(name) {
        *id
    } else {
        depth + next
    }
}

pub fn add_std(ins: &mut Vec<Instr>, state: &mut State, depth: usize, next: &mut usize) {
    macro_rules! ins {
        ($op:expr) => {
            ins.push($op)
        };
    }
    macro_rules! push_literal {
        ($val:expr) => {
            ins.push(Instr::Push(StackValue::Literal($val)))
        };
    }
    macro_rules! build {
        ($val:expr) => {
            Self::build(ins, $val, state, depth, next)
        };
        ($val:expr, $incr:expr) => {
            Self::build(ins, $val, state, depth + $incr, next)
        };
    }

    macro_rules! load {
        ($id:expr, $d:expr) => {
            if $d == 0 {
                ins.push(Instr::LoadGlobal($id))
            } else if $d == depth {
                ins.push(Instr::LoadLocal($id))
            } else {
                ins.push(Instr::LoadAddr($id + $d))
            }
        };
    }

    macro_rules! store {
        ($id:expr, $d:expr) => {
            // println!("{:?} {:?}", $d, depth);
            if $d == 0 {
                ins.push(Instr::StoreGlobal($id))
            } else if $d == depth {
                ins.push(Instr::StoreLocal($id))
            } else {
                ins.push(Instr::StoreAddr($id + $d))
            }
        };
    }

    // print
    let top = ins.len();

    let id = get_id("print", state, depth, *next);
    *next += 1;
    state.insert("print".to_string(), (id, depth));

    ins!(Instr::Jump(top + 4));
    ins!(Instr::Print);
    push_literal!(Type::None);
    ins!(Instr::Return);
    push_literal!(Type::FuncPtr(top + 1));
    store!(id, depth);
}
