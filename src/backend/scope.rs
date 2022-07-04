// use hashbrown::HashMap;
extern crate fxhash;

use fxhash::FxHashMap;

#[allow(non_camel_case_types)]
pub type id = usize;
#[allow(non_camel_case_types)]
pub type addr = usize;

// pub fn get_id(name: &str) -> id {
//     let mut hasher = std::collections::hash_map::DefaultHasher::new();
//     name.hash(&mut hasher);
//     hasher.finish()
// }

#[derive(Clone, Debug, PartialEq)]
pub struct Scope(pub FxHashMap<id, addr>);

impl Scope {
    pub fn new() -> Scope {
        Scope(FxHashMap::default())
    }
    #[inline]
    pub fn get(&self, id: id) -> Option<addr> {
        self.0.get(&id).cloned()
    }
    #[inline]
    pub fn set(&mut self, id: id, addr: addr) {
        self.0.insert(id, addr);
    }
}
