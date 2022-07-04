use glass::backend::instruction::Opcode;
use glass::backend::vm::VM;

use glass::frontend::parser;

// fn factorial(n) {
//     if (n == 0 || n == 1) {
//         return 1
//     } else {
//         return n * factorial(n - 1)
//     }
// }

fn main() {
    let code = "
        let x = 2
        
    ";

    let ast = parser::parse_code(code).unwrap();
    println!("{:?}", ast);

    let program = Opcode::compile(ast);
    // program.push(Opcode::Print);

    println!("ln#\topcode    \tid/value");

    for (i, instruction) in program.iter().enumerate() {
        if instruction != &Opcode::Noop {
            println!("{}:\t{:?}", i, instruction);
        }
    }
    println!();

    let mut vm = VM::new();
    vm.program = program;

    let s = std::time::Instant::now();
    vm.run();
    println!("Took {:?}ms", s.elapsed().as_millis());

    println!("{:?}", vm.heap.internal);
}
