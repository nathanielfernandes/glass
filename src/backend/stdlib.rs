use super::{
    instruction::{Opcode, State, Type},
    stack::StackValue,
};

fn get_id(name: &str, state: &State, depth: usize, next: usize) -> usize {
    if let Some((id, _)) = state.get(name) {
        *id
    } else {
        depth + next
    }
}

pub fn add_std(ins: &mut Vec<Opcode>, state: &mut State, depth: usize, next: &mut usize) {
    macro_rules! op {
        ($op:expr) => {
            ins.push($op)
        };
    }
    macro_rules! push_literal {
        ($val:expr) => {
            ins.push(Opcode::Push(StackValue::Literal($val)))
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
                ins.push(Opcode::LoadGlobal($id))
            } else if $d == depth {
                ins.push(Opcode::LoadLocal($id))
            } else {
                ins.push(Opcode::Load($id))
            }
        };
    }

    macro_rules! store {
        ($id:expr, $d:expr) => {
            // println!("{:?} {:?}", $d, depth);
            if $d == 0 {
                ins.push(Opcode::StoreGlobal($id))
            } else if $d == depth {
                ins.push(Opcode::StoreLocal($id))
            } else {
                ins.push(Opcode::Store($id))
            }
        };
    }

    // print
    let top = ins.len();

    let id = get_id("print", state, depth, *next);
    *next += 1;
    state.insert("print".to_string(), (id, depth));

    op!(Opcode::Jump(top + 4));
    op!(Opcode::Print);
    push_literal!(Type::None);
    op!(Opcode::Return);
    push_literal!(Type::FuncPtr(top + 1));
    store!(id, depth);
}
