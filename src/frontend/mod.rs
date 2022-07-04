#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Comment(String),

    Number(f64),
    String(String),
    Bool(bool),
    None,

    Symbol(String),

    Declaration(String, Box<Expr>),
    Assignment(String, Box<Expr>),

    Function(String, Vec<String>, Vec<Expr>),
    Lambda(Vec<String>, Vec<Expr>),
    Call(String, Vec<Expr>),

    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),

    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    // Error(String),
    If(Box<Expr>, Vec<Expr>, Vec<Expr>),
    // While(Box<Expr>, Box<Expr>),
    Return(Box<Expr>),
    // Break,
    // Continue,
}

pub type AST = Vec<Node>;
pub type Node = Expr;

peg::parser!(
    pub grammar parser() for str {
        rule whitespace()
        = [' '| '\t' | '\n' | '\r' |'\u{A}']
        rule _
        = whitespace()*
        rule __
        = whitespace()+

        rule string() -> String
        = "\"" n:$([^ '"']*) "\"" {n.to_owned()}

        rule symbol() -> String
        = quiet!{ _ n:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) _ { n.to_owned() } }
        / expected!("identifier")

        rule number() -> f64
        = n:$("-"?['0'..='9']+ ("."['0'..='9'])?) { n.parse::<f64>().unwrap_or_else(|_|panic!("value: {} is not a valid number!", n)) }

        rule bool() -> bool
        = "true" { true } / "false" { false }

        rule none() -> Expr
        = "none" { Expr::None }

        rule function() -> Expr
        = _ "fn" __ name:symbol() _
        "(" params:(symbol() ** ",") ")" _
        code:block() _
        { Expr::Function(name, params, code)}

        rule lambda() -> Expr
        = _ "(" params:(symbol() ** ",") ")" _ "=>" _
        code:block() _
        { Expr::Lambda(params, code)}

        rule call() -> Expr
        = _ name:symbol() _ "(" args:((_ e:expr() _ {e})  ** ",") ")" _
        { Expr::Call(name, args) }

        rule _return() -> Expr
        = _ "return" e:(__ e:expr() _ { e } / _ { Expr::None }) {Expr::Return(Box::new(e))}

        rule declaration() -> Expr
        = _ "let" __ name:symbol() _ value:(("=" _ value:expr() _ { value }) / { Expr::None }) { Expr::Declaration(name, Box::new(value)) }

        rule assignment() -> Expr
        = _ name:symbol() _ "=" _ value:expr() _ { Expr::Assignment(name, Box::new(value)) };

        rule block() -> Vec<Expr>
        = _ "{" _ e:(parse()*) _ "}" _ { e }

        rule _else() -> Vec<Expr>
        = code:block() {code}
        rule _elif() -> Vec<Expr>
        = code: if_condition() {vec![code]}
        rule else_elif() -> Vec<Expr>
        = "else" _ res:(_else() / _elif()) {res}
        rule if_condition() -> Expr
        = _ "if" _ "(" _ condition:operation() _ ")"  _ then:block() _ otherwise:(else_elif())? _ {
            Expr::If(Box::new(condition), then, otherwise.unwrap_or(vec![]))
        }

        #[cache_left_rec]
        rule arithmetic() -> Expr
        = precedence! {
            _ "(" _ x:arithmetic() _ ")" _ { x }
            --
            x:symbol() _ "++" _ { Expr::Assignment(x.clone(), Box::new(Expr::Add(Box::new(Expr::Symbol(x)), Box::new(Expr::Number(1.0))))) }
            x:symbol() _ "--" _ { Expr::Assignment(x.clone(), Box::new(Expr::Sub(Box::new(Expr::Symbol(x)), Box::new(Expr::Number(1.0))))) }
            x:symbol() _ "+=" _ y:@ { Expr::Assignment(x.clone(), Box::new(Expr::Add(Box::new(Expr::Symbol(x)), Box::new(y)))) }
            x:symbol() _ "-=" _ y:@ { Expr::Assignment(x.clone(), Box::new(Expr::Sub(Box::new(Expr::Symbol(x)), Box::new(y)))) }
            x:symbol() _ "*=" _ y:@ { Expr::Assignment(x.clone(), Box::new(Expr::Mul(Box::new(Expr::Symbol(x)), Box::new(y)))) }
            x:symbol() _ "/=" _ y:@ { Expr::Assignment(x.clone(), Box::new(Expr::Div(Box::new(Expr::Symbol(x)), Box::new(y)))) }
            x:symbol() _ "%=" _ y:@ { Expr::Assignment(x.clone(), Box::new(Expr::Mod(Box::new(Expr::Symbol(x)), Box::new(y)))) }
            x:symbol() _ "**=" _ y:@ { Expr::Assignment(x.clone(), Box::new(Expr::Pow(Box::new(Expr::Symbol(x)), Box::new(y)))) }
            --
            x:(@) _ "+" _  y:@ { Expr::Add(Box::new(x), Box::new(y)) }
            x:(@) _ "-" _  y:@ { Expr::Sub(Box::new(x), Box::new(y)) }
            --
            x:(@) _ "*" _  y:@ { Expr::Mul(Box::new(x), Box::new(y)) }
            x:(@) _ "/" _  y:@ { Expr::Div(Box::new(x), Box::new(y)) }
            x:(@) _ "%" _  y:@ { Expr::Mod(Box::new(x), Box::new(y)) }
            --
            x:(@) _ "**" _  y:@ { Expr::Pow(Box::new(x), Box::new(y)) }
            --
            x:value() { x }
            "-" _ e:arithmetic() { Expr::Neg(Box::new(e)) }
        }

        #[cache_left_rec]
        rule operation() -> Expr
        = precedence! {
            "(" _ x:operation() _ ")" _ { x }
            --
            x:(@) _ "&&" _  y:@ { Expr::And(Box::new(x), Box::new(y)) }
            x:(@) _ "||" _  y:@ { Expr::Or(Box::new(x), Box::new(y)) }
            --
            x:(@) _ "==" _  y:@ { Expr::Eq(Box::new(x), Box::new(y)) }
            x:(@) _ "!=" _  y:@ { Expr::Neq(Box::new(x), Box::new(y)) }
            --
            x:(@) _ ">=" _  y:@ { Expr::Gte(Box::new(x), Box::new(y)) }
            x:(@) _ "<=" _  y:@ { Expr::Lte(Box::new(x), Box::new(y)) }
            --
            x:(@) _ "<" _  y:@ { Expr::Lt(Box::new(x), Box::new(y)) }
            x:(@) _ ">" _  y:@ { Expr::Gt(Box::new(x), Box::new(y)) }
            --
            x:value() { x }
            "!" _  x:operation() { Expr::Not(Box::new(x)) }
        }

        #[cache_left_rec]
        rule value() -> Expr
        = precedence!{
            n:arithmetic() { n }
            n:operation() { n }
            n:lambda() { n }
            c:call() { c }
            --
            n:if_condition() { n }
            --
            n:bool() { Expr::Bool(n) }
            n:none() { Expr::None }
            n:number() { Expr::Number(n) }
            s:string() { Expr::String(s) }
            n:symbol() { Expr::Symbol(n) }
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
            n:lambda() { n }
            n:function() { n }
            --
            n:if_condition() { n }
            --
            n:arithmetic() { n }
            n:operation() { n }
            --
            n:call() { n }
            --
            n:bool() { Expr::Bool(n) }
            n:none() { Expr::None }
            n:number() { Expr::Number(n) }
            s:string() { Expr::String(s) }
            n:symbol() { Expr::Symbol(n) }

        }

        rule parse() -> Expr =
        _  n:expr() &_  {n}

        pub rule parse_code() -> AST
        = _ code:((x:parse() (";"/"\n"/";"/_) {x})*) _ {code}

    }
);
