pub mod second;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Comment(String),
    Number(f64),
    String(String),
    FormatString(Vec<Expr>),
    Bool(bool),
    None,

    Identifier(String),

    Declaration(String, Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),

    Index {
        item: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        item: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },

    // Walrus(String, Box<Expr>),
    Function {
        name: String,
        args: Vec<String>,
        body: Vec<Expr>,
    },
    Lambda(Vec<String>, Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    NativeCall(String, Vec<Expr>),

    Join(Box<Expr>, Box<Expr>),

    Op(Op, Box<Expr>, Box<Expr>),
    // Error(String),
    If {
        condition: Box<Expr>,
        then: Vec<Expr>,
        otherwise: Vec<Expr>,
    },
    // While(Box<Expr>, Box<Expr>),
    Return(Box<Expr>),
    // Break,
    // Continue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Not,
    Neg,
}

pub type AST = Vec<Node>;
pub type Node = Expr;

fn fix_str(s: Vec<char>) -> String {
    let mut f = Vec::new();
    let mut rep = false;
    for c in s {
        if c == '\\' {
            rep = true;
        } else {
            if rep {
                f.push(match c {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    _ => c,
                });
                rep = false;
            } else {
                f.push(c);
            }
        }
    }

    f.into_iter().collect()
}

peg::parser!(
    pub grammar parser() for str {

        rule whitespace()
        = [' '| '\t' | '\n' | '\r' |'\u{A}']
        rule _
        = whitespace()*
        rule __
        = whitespace()+

        rule string() -> String
        = quiet!{ "\"" s:(doubleQuotedCharacter()*) "\"" { fix_str(s) }}
        / expected!("string")

        rule doubleQuotedCharacter() -> char
          = !("\"") c:([_]) { c }
          / "\\u{" value:$(['0'..='9' | 'a'..='f' | 'A'..='F']+) "}" { char::from_u32(u32::from_str_radix(value, 16).unwrap()).unwrap() }
          / expected!("valid escape sequence")

        rule doubleQuotedCharacterNoBrac() -> char
        = !(&"{" / &"}" / "\"") c:([_]) { c }
        / "\\u{" value:$(['0'..='9' | 'a'..='f' | 'A'..='F']+) "}" { char::from_u32(u32::from_str_radix(value, 16).unwrap()).unwrap() }
        / expected!("valid escape sequence")

        rule f() -> Expr
        = "{" _ e:expr() _ "}" {e} /
        e:(doubleQuotedCharacterNoBrac()+) {Expr::String(fix_str(e))}

        rule format_string() -> Expr
        = "f\"" s:(f() *) "\"" {
            Expr::FormatString(s)
        }

        rule symbol() -> String
        = quiet!{ _ n:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) _ { n.to_owned() } }
        / expected!("identifier")

        rule integer() -> f64
        = quiet!{ _ i:$("-"?['0'..='9']+) _ { i.parse().unwrap() } }
        / expected!("integer")

        rule float() -> f64
        = quiet!{ _ i:$("-"?['0'..='9']+ "." !"." ['0'..='9']*) _ { i.parse().unwrap() } }
        / expected!("float")

        rule number() -> f64
        = float() / integer()


        // rule index() -> Expr
        // = _ n:value() "[" _ s:(e:value() **<1,2> ":") _ "]" _  {
        //     if s.len() == 2 {
        //         Expr::Slice{item:Box::new(n), start:Box::new(s[0].clone()), end:Box::new(s[1].clone())}
        //     } else {
        //         Expr::Index{item:Box::new(n), index:Box::new(s[0].clone())}
        //     }
        // }


        #[cache_left_rec]
        rule index() -> Expr
        = n:(i:index(){i} / s:call(){s} / s:string(){Expr::String(s)} / s:format_string(){s} / s:symbol(){Expr::Identifier(s)})
        "[" _ i:value() _ "]" { Expr::Index{item: Box::new(n.clone()), index: Box::new(i.clone())} }


        rule bool() -> bool
        = "true" { true } / "false" { false }
        / expected!("bool")

        rule none() -> Expr
        = "none" { Expr::None }
        / expected!("none")

        rule function() -> Expr
        = _ "fn" __ name:symbol() _
        "(" args:(symbol() ** ",") ")" _
        body:block() _
        { Expr::Function {name, args, body} }

        rule lambda() -> Expr
        = _ "(" params:(symbol() ** ",") ")" _ "=>" _
        code:block() _
        { Expr::Lambda(params, code)}

        #[cache_left_rec]
        rule call() -> Expr
        = _ name:( c:call() / i:index() / s:symbol(){Expr::Identifier(s)} ) _ "(" args:((_ e:value() _ {e})  ** ",") ")" _
        { Expr::Call(Box::new(name), args) }

        rule native_call() -> Expr
        = _ "#" name:symbol() _ "(" args:((_ e:value() _ {e})  ** ",") ")" _
        { Expr::NativeCall(name, args) }

        rule _return() -> Expr
        = _ "return" e:(__ e:value() _ { e } / _ { Expr::None }) {Expr::Return(Box::new(e))}

        rule declaration() -> Expr
        = _ "let" __ name:symbol() _ value:(("=" _ value:value() _ { value }) / { Expr::None }) { Expr::Declaration(name, Box::new(value)) }

        rule assignment() -> Expr
        = _ name:(i:index(){i} / s:call(){s} / s:symbol(){Expr::Identifier(s)}) _ "=" _ value:value() _ { Expr::Assignment(Box::new(name), Box::new(value)) };

        // #[cache_left_rec]
        // rule join() -> Expr
        // = _ e:value() _ ".." _ e2:value() _{ Expr::Join(Box::new(e), Box::new(e2)) }

        // rule assignment_expression() -> Expr
        // = _ name:symbol() _ ":=" _ value:value() _ { Expr::Walrus(name, Box::new(value)) };

        rule block() -> Vec<Expr>
        = _ "{" _ e:(parse()*) _ "}" _ { e }

        rule _else() -> Vec<Expr>
        = code:block() {code}
        rule _elif() -> Vec<Expr>
        = code: if_condition() {vec![code]}
        rule else_elif() -> Vec<Expr>
        = "else" _ res:(_else() / _elif()) {res}
        rule if_condition() -> Expr
        = _ "if" _ condition:operation() _ then:block() _ otherwise:(else_elif())? _ {
            Expr::If{ condition: Box::new(condition), then, otherwise: otherwise.unwrap_or(vec![])}
        }



        rule item() -> Expr
        = i:index(){i} / s:call(){s} / s:symbol(){Expr::Identifier(s)}

        #[cache_left_rec]
        rule arithmetic() -> Expr
        = precedence! {
            _ "(" _ x:arithmetic() _ ")" _ { x }
            --
            x:item() _ "++" _ {Expr::Assignment(Box::new(x.clone()), Box::new(Expr::Op(Op::Add, Box::new(x), Box::new(Expr::Number(1.0)))))}
            x:item() _ "--" _ {Expr::Assignment(Box::new(x.clone()), Box::new(Expr::Op(Op::Sub, Box::new(x), Box::new(Expr::Number(1.0)))))}
            x:item() _ "+=" _ y:@ {Expr::Assignment(Box::new(x.clone()), Box::new(Expr::Op(Op::Add, Box::new(x), Box::new(y))))}
            x:item() _ "-=" _ y:@ {Expr::Assignment(Box::new(x.clone()), Box::new(Expr::Op(Op::Sub, Box::new(x), Box::new(y))))}
            x:item() _ "*=" _ y:@ {Expr::Assignment(Box::new(x.clone()), Box::new(Expr::Op(Op::Mul, Box::new(x), Box::new(y))))}
            x:item() _ "/=" _ y:@ {Expr::Assignment(Box::new(x.clone()), Box::new(Expr::Op(Op::Div, Box::new(x), Box::new(y))))}
            x:item() _ "%=" _ y:@ {Expr::Assignment(Box::new(x.clone()), Box::new(Expr::Op(Op::Mod, Box::new(x), Box::new(y))))}
            x:item() _ "**=" _ y:@ {Expr::Assignment(Box::new(x.clone()), Box::new(Expr::Op(Op::Pow, Box::new(x), Box::new(y))))}
            --
            x:(@) _ ".." _  y:@ {Expr::Join(Box::new(x), Box::new(y))}
            x:(@) _ "+" _  y:@ {Expr::Op(Op::Add, Box::new(x), Box::new(y))}
            x:(@) _ "-" _  y:@ {Expr::Op(Op::Sub, Box::new(x), Box::new(y))}
            --
            x:(@) _ "*" _  y:@ {Expr::Op(Op::Mul, Box::new(x), Box::new(y))}
            x:(@) _ "/" _  y:@ {Expr::Op(Op::Div, Box::new(x), Box::new(y))}
            x:(@) _ "%" _  y:@ {Expr::Op(Op::Mod, Box::new(x), Box::new(y))}
            --
            x:(@) _ "**" _  y:@ {Expr::Op(Op::Pow, Box::new(x), Box::new(y))}
            --
            x:value_end() { x }
            "-" _ e:arithmetic() { Expr::Op(Op::Neg, Box::new(e), Box::new(Expr::None)) }
        }

        #[cache_left_rec]
        rule operation() -> Expr
        = precedence! {
            "(" _ x:operation() _ ")" _ { x }
            --
            x:(@) _ "&&" _  y:@ { Expr::Op(Op::And, Box::new(x), Box::new(y)) }
            x:(@) _ "||" _  y:@ { Expr::Op(Op::Or, Box::new(x), Box::new(y)) }
            --
            x:(@) _ "==" _  y:@ { Expr::Op(Op::Eq, Box::new(x), Box::new(y)) }
            x:(@) _ "!=" _  y:@ { Expr::Op(Op::Neq, Box::new(x), Box::new(y)) }
            --
            x:(@) _ ">=" _  y:@ { Expr::Op(Op::Gte, Box::new(x), Box::new(y)) }
            x:(@) _ "<=" _  y:@ { Expr::Op(Op::Lte, Box::new(x), Box::new(y)) }
            --
            x:(@) _ "<" _  y:@ { Expr::Op(Op::Lt, Box::new(x), Box::new(y)) }
            x:(@) _ ">" _  y:@ { Expr::Op(Op::Gt, Box::new(x), Box::new(y)) }
            --
            x:value_end() { x }
            "!" _  x:operation() {
                if let Expr::Bool(b) = x {
                    return Expr::Bool(!b)
                }
                Expr::Op(Op::Not, Box::new(x), Box::new(Expr::None))
            }
        }

        #[cache_left_rec]
        rule value_end() -> Expr
        = precedence!{
            n:index() { n }
            n:native_call() { n }
            c:call() { c }
            --
            n:bool() { Expr::Bool(n) }
            n:none() { Expr::None }
            n:number() { Expr::Number(n) }
            s:format_string() { s }
            s:string() { Expr::String(s) }
            n:symbol() { Expr::Identifier(n) }
        }

        #[cache_left_rec]
        rule value() -> Expr
        = precedence!{

            n:arithmetic() { n }
            n:operation() { n }
            // n:lambda() { n }
            --
            n:index() { n }
            n:native_call() { n }
            c:call() { c }
            --
            n:bool() { Expr::Bool(n) }
            n:none() { Expr::None }
            n:number() { Expr::Number(n) }
            s:format_string() { s }
            s:string() { Expr::String(s) }
            n:symbol() { Expr::Identifier(n) }
        }

        rule expr() -> Expr
        = precedence!{
            // _ "(" _ x:expr() _ ")" _ { x }
            // --
            n:_return() { n }
            --
            n:declaration() { n }
            --
            n:assignment() { n }
            --
            // n:lambda() { n }
            n:function() { n }
            --
            n:if_condition() { n }
            --
            n:arithmetic() { n }
            n:operation() { n }
            --
            n:native_call() { n }
            n:call() { n }
            n:index() { n }
            --
            n:bool() { Expr::Bool(n) }
            n:none() { Expr::None }
            n:number() { Expr::Number(n) }
            s:format_string() { s }
            s:string() { Expr::String(s) }
            n:symbol() { Expr::Identifier(n) }
        }

        rule parse() -> Expr =
        _  n:expr() &_  {n}

        pub rule parse_code() -> AST
        = _ code:((x:parse() (";"/"\n"/_) {x})*) _ {code}

    }
);
