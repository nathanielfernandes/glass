use std::fmt;

use super::{instruction::Type, memory::addr};

#[derive(Clone, PartialEq)]
pub enum StackValue {
    Literal(Type),
    Addr(addr),
}

impl fmt::Debug for StackValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackValue::Literal(ty) => write!(f, "{:?}", ty),
            StackValue::Addr(addr) => write!(f, "addr({})", addr),
        }
    }
}

pub struct Stack {
    internal: Vec<StackValue>,
    // pub sp: usize,
    // pub fp: usize,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            internal: Vec::new(),
            // sp: 0,
            // fp: 0,
        }
    }

    pub fn new_with(first: StackValue) -> Stack {
        Stack {
            internal: vec![first],
            // sp: 0,
            // fp: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, value: StackValue) {
        self.internal.push(value);
        // self.sp += 1;
    }
    #[inline]
    pub fn pop(&mut self) -> StackValue {
        let val = self.internal.pop().expect("Popped from empty stack");
        // self.sp -= 1;
        val
    }

    // #[inline]
    // pub fn try_pop(&mut self) {
    //     if self.sp > 1 {
    //         println!("POPPED {:?}", self.pop());
    //     }
    // }

    #[inline]
    pub fn peek(&self) -> Option<&StackValue> {
        self.internal.last()
    }

    #[inline]
    pub fn peek_mut(&mut self) -> &mut StackValue {
        self.internal.last_mut().expect("Peeked from empty stack")
    }
}
