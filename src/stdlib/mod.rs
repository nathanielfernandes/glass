use std::fs::read_to_string;
use std::io::Write;
use std::path;
use std::time::{SystemTime, UNIX_EPOCH};

use fxhash::FxHashMap;

use crate::backend::instruction::Instr;
use crate::backend::stack::StackValue;
use crate::backend::{instruction::Type, vm::VM};
use crate::frontend::parser;

macro_rules! native {
    ($(fn $name:ident ( $($args:ident)*  ) $func:block)*) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, PartialEq)]
        pub enum NativeFunction {
            $($name,)*
        }

        impl NativeFunction {
            pub fn name(&self) -> &'static str {
                match self {
                    $(NativeFunction::$name => stringify!($name),)*
                }
            }

            pub fn from(name: &str) -> Option<NativeFunction> {
                match name {
                    $(stringify!($name) => Some(NativeFunction::$name),)*
                    _ => None,
                }
            }

            pub fn call(&self, vm: &mut VM) {
                let result = match self {
                    $(NativeFunction::$name => {
                        $(let $args = vm.pop_stack();
                        let $args = $args.as_ref();)*

                        $func
                    },)*
                };
                vm.stack.push(StackValue::Literal(result));
            }
        }
    };
}

native!(
    fn stdout(_str) {
        print!("{}", _str.to_string());
        Type::None
    }

    fn stdin() {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok().expect("Failed to read line");
        Type::String(input)
    }

    fn flushout() {
        std::io::stdout().flush().ok().expect("Failed to flush stdout");
        Type::None
    }

    fn time() {
        Type::Number(SystemTime::now().duration_since(UNIX_EPOCH).expect("Failed to get time").as_millis() as f64)
    }
);

pub fn add_std(
    program: &mut Vec<Instr>,
    state: &mut FxHashMap<String, (usize, usize)>,
    depth: usize,
    next: &mut usize,
) {
    let code = read_to_string(path::Path::new("src/stdlib/std.rv")).unwrap();
    let ast = parser::parse_code(&code).unwrap();
    Instr::iter_build(program, ast, state, depth, next);
}
