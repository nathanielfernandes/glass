// use hashbrown::HashMap;
use fxhash::FxHashMap;

use super::instruction::Type;

#[allow(non_camel_case_types)]
pub type addr = usize;

pub struct Memory {
    pub internal: FxHashMap<addr, Type>,
    pub size: addr,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            internal: FxHashMap::default(),
            size: 0,
        }
    }

    pub fn get(&self, addr: addr) -> &Type {
        self.internal.get(&addr).expect("Address not found")
        // self.internal.get(&addr).unwrap_or(&Type::Null)
    }

    pub fn set(&mut self, addr: addr, value: Type) {
        if let None = self.internal.insert(addr, value) {
            self.size += 1;
        }
    }

    pub fn delete(&mut self, addr: addr) {
        self.size -= 1;
        self.internal.remove(&addr);
    }

    pub fn add(&mut self, value: Type) -> addr {
        self.size += 1;
        self.internal.insert(self.size, value);
        self.size
    }
}

// pub struct Memory {
//     pub internal: Vec<Type>,
//     pub size: usize,
// }

// impl Memory {
//     pub fn new() -> Memory {
//         Memory {
//             internal: vec![],
//             size: 0,
//         }
//     }

//     pub fn push(&mut self, value: Type) {
//         self.internal.push(value);
//         self.size += 1;
//     }

//     pub fn add(&mut self, value: Type) -> usize {
//         self.push(value);
//         self.size - 1
//     }

//     pub fn get(&self, index: usize) -> &Type {
//         &self.internal[index]
//     }

//     pub fn pop(&mut self, index: usize) -> Type {
//         self.internal.remove(index)
//     }

//     pub fn set(&mut self, index: usize, value: Type) {
//         if index + 1 >= self.size {
//             self.internal.resize(index + 1, Type::Null);
//             self.size = index + 1
//         }

//         self.internal[index] = value;
//     }

//     pub fn delete(&mut self, index: usize) {
//         // self.internal.remove(index);
//         self.internal[index] = Type::Null;
//         self.size -= 1;
//     }

//     pub fn end(&self) -> usize {
//         self.size
//     }
// }
