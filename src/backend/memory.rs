// use hashbrown::HashMap;
use fxhash::FxHashMap;

use super::instruction::Type;

#[allow(non_camel_case_types)]
pub type addr = usize;

pub struct Memory(FxHashMap<addr, Type>);

impl Memory {
    pub fn new() -> Memory {
        Memory(FxHashMap::default())
    }

    pub fn get<'a>(&'a self, addr: addr) -> &'a Type {
        self.0.get(&addr).expect("Address not found")
        // self.internal.get(&addr).unwrap_or(&Type::Null)
    }

    pub fn set(&mut self, addr: addr, value: Type) {
        self.0.insert(addr, value);
        // if let None = self.internal.insert(addr, value) {
        //     // self.size += 1;
        // }
    }

    pub fn delete(&mut self, addr: addr) {
        // self.size -= 1;
        // self.internal.len();
        self.0.remove(&addr);
    }

    pub fn add(&mut self, value: Type) -> addr {
        // self.size += 1;
        let size = self.0.len();
        self.0.insert(size, value);
        // self.size
        size
    }
}

// pub struct Memory {
//     pub internal: Vec<Type>,
//     pub size: usize,
// }

// impl Memory {
//     pub fn new() -> Memory {
//         Memory {
//             internal: Vec::with_capacity(100_000),
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
//         // if index + 1 >= self.size {
//         //     self.internal.resize(index + 1, Type::None);
//         //     self.size = index + 1
//         // }

//         self.internal[index] = value;
//     }

//     pub fn delete(&mut self, index: usize) {
//         // self.internal.remove(index);
//         // self.internal[index] = Type::None;
//         self.size -= 1;
//     }

//     pub fn end(&self) -> usize {
//         self.size
//     }
// }
