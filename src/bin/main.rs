use std::fs::{read_to_string, File};
use std::io::Write;
use std::path;

use glass::backend::instruction::Instr;
use glass::backend::vm::VM;

use glass::frontend::parser;

fn write_program(program: &[Instr], path: &path::Path) {
    let mut file = File::create(path).unwrap();
    write!(file, "ln#\topcode    \toffset/value\n").unwrap();
    write!(file, "-------------------------\n").unwrap();
    for (i, instruction) in program.iter().enumerate() {
        write!(file, "{}:\t{:?}\n", i, instruction).unwrap();
    }
    write!(file, "-------------------------\n").unwrap();
}

fn main() {
    // let s = std::time::Instant::now();

    let code = read_to_string(path::Path::new("src/bin/test.rv")).unwrap();
    let ast = parser::parse_code(&code).unwrap();
    let (program, prog_start) = Instr::compile(ast);

    write_program(
        &program[prog_start..],
        // &program,
        path::Path::new("src/bin/test.rv.out"),
    );

    let mut vm = VM::new();
    vm.program = program;

    // println!("Took {:?}μs", s.elapsed().as_micros());

    vm.run();
    // vm.debug();
    // println!("Took {:?}ms", s.elapsed().as_millis());

    // println!("{:?}", vm.heap.0);
    // println!("{:?}", vm.scopes[0].0);
}
