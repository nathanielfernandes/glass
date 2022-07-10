# Glass 
A dynamically typed intepreted toy language.

## `🚧 Under Construction 🚧`
```rust
fn fib(n) {
    if (n == 0 || n == 1) {
        return n
    } else {
        return fib(n - 1) + fib(n - 2)
    }
}   

let val = fib(30)
print(val)
```
##### `132μs` to open file, parse code and generate bytecode. `268ms` to execute `fib(30)`
```
832040
```
## Features
Right now, Glass is just a hobby project that I'm using to explore language design and compiler development.

```rust
// recursion
fn fib(n) {
    // branching (aswell as short circuiting)
    if (n == 0 || n == 1) {
        return n
    } else {
        return fib(n - 1) + fib(n - 2)
    }
}   

// functional features
fn run_func(n, func) {
    return func(n)
}
let val = return_func(30, fib)


// format  strings
let name = "nathan"
let age = 40
print(f"my name is {name} and i am {age / 2} years old")


// indexing
let name = "nathan"
print(name[0])
```


## How it works
Glass parses your source code and generates it's own bytecode.

This bytecode is then interpreted by the Glass VM.

## Under the hood

### Backend
#### Instrcution Set
Glass generates high level bytecode which is interpreted by it's stack based VM.

| Instruction | data | description |
|---|---|---|
| Noop |  | No op |
| Halt |  | Stops the program |
|  |  |  |
| LoadAddr | address | pushes a value from the heap onto the stack (does not deref) |
| LoadLocal | offset | pushes a value from the local scope onto the stack |
| LoadGlobal | offset | pushes a value from the global scope onto the stack |
|  |  |  |
| StoreAddr | address | pops a value off the stack and stores it on the heap |
| StoreLocal | offset | pops a value off the stack and stores it on the heap |
| StoreGlobal | offset | pops a value off the stack and stores it on the heap |
|  |  |  |
| Push | type | pushes a value onto the stack |
| Pop |  | pops a value off the stack |
|  |  |  |
| Jump | address | sets the pc to the given address |
| JumpIf | address | pops a value off the stack sets the pc to the given address if the value is true |
| JumpIfNot | address | pops a value off the stack sets the pc to the given address if the value is false |
| Call |  | pops a value off the stack and jumps to the value if it is a function ptr |
| NativeCall | | calls a native rust function |
| Return |  | pops a value off the stack and jumps to the return address |
| | | |
| Join | | pops two values off the stack and joins them |
| JoinMany | amount | pops values off the stack and joins them |
| Index |  | pops two values off the stack and gets the index of the first value by the second value |
| binary_ops... |  | pops two values off the stack and pushes the result |


#### Example Bytecode

```rust
fn fib(n) {
    if (n == 0 || n == 1) {
        return n
    } else {
        return fib(n - 1) + fib(n - 2)
    }
}   

fib(30)
```
**compiles to**
```
ln#	opcode    	offset/value
-------------------------
0:	Jump    	27
1:	StoreLocal	1
2:	LoadLocal	1
3:	Push    	num(0)
4:	Eq              
5:	JumpIf  	10
6:	LoadLocal	1
7:	Push    	num(1)
8:	Eq              
9:	JumpIfNot	13
10:	LoadLocal	1
11:	Return           
12:	Jump    	25
13:	LoadLocal	1
14:	Push    	num(1)
15:	Sub              
16:	LoadGlobal	0
17:	Call              
18:	LoadLocal	1
19:	Push    	num(2)
20:	Sub              
21:	LoadGlobal	0
22:	Call              
23:	Add              
24:	Return           
25:	Push    	none
26:	Return           
27:	Push    	fn(@1)
28:	StoreGlobal	0
29:	Push    	num(30)
30:	LoadGlobal	0
31:	Call              
32:	Pop           
-------------------------
```

### Frontend
Glass uses the [**peg**](https://docs.rs/peg/latest/peg/) crate to do all it's parsing.

**Parser Types**
- Number
- String
- Bool
- None
- Symbol
- Declartion
- Assignment
- Function
- Lambda
- Call
- BinaryOperation
- If
- Return
