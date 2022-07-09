use super::{Expr, Node, AST};

fn optimize_ast(ast: AST, initial: bool) -> AST {
    ast.into_iter().filter_map(|n| pass(n, initial)).collect()
}

fn pass(node: Node, initial: bool) -> Option<Node> {
    match node {
        Expr::Declaration(_, _) => todo!(),
        Expr::Assignment(_, _) => todo!(),
        Expr::Index { item, index } => todo!(),
        // Expr::Slice { item, start, end } => todo!(),
        Expr::Function { name, args, body } => todo!(),
        Expr::Lambda(_, _) => todo!(),
        Expr::Call(_, _) => todo!(),
        Expr::NativeCall(_, _) => todo!(),
        Expr::Join(_, _) => todo!(),
        Expr::Op(_, _, _) => todo!(),
        Expr::If {
            condition,
            then,
            otherwise,
        } => todo!(),
        Expr::Return(r) => Some(Expr::Return(Box::new(pass(*r, false).unwrap()))),
        Expr::FormatString(nodes) => Some(Expr::FormatString(optimize_ast(nodes, false))),

        Expr::Identifier(_) => if_initial(node, initial),
        Expr::Number(_) => if_initial(node, initial),
        Expr::String(_) => if_initial(node, initial),
        Expr::Bool(_) => if_initial(node, initial),
        Expr::None => if_initial(node, initial),

        _ => Some(node),
    }
}

fn if_initial(node: Node, initial: bool) -> Option<Node> {
    if initial {
        None
    } else {
        Some(node)
    }
}
