use crate::{lang::StitchLang, smc::{StitchAnalysis, StitchEgraph}};
use egg::FromOp;

/// Loads a JSON file containing s-expressions and builds an egraph from them.
/// All programs are combined into a single term (programs A B C ...).
/// Returns the egraph and the root e-class Id of the programs node.
pub fn load_egraph(filename: &str) -> (StitchEgraph, egg::Id) {
    let contents = std::fs::read_to_string(filename).expect("Failed to read file");
    let exprs: Vec<String> = serde_json::from_str(&contents).expect("Failed to parse JSON");

    let mut egraph: StitchEgraph = egg::EGraph::default();

    let mut expr_ids = Vec::new();

    for expr_str in &exprs {
        let expr: egg::RecExpr<StitchLang> = expr_str.parse().expect("Failed to parse expression");
        expr_ids.push(egraph.add_expr(&expr));
    }

    let programs_node = StitchLang::from_op("programs", expr_ids).expect("Failed to create programs node");
    let root = egraph.add(programs_node);
    let rules: Vec<egg::Rewrite<StitchLang, StitchAnalysis>>    = vec![
        egg::rewrite!("commute_add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        egg::rewrite!("assoc_add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        egg::rewrite!("identity_add"; "(+ ?a 0)" => "?a"),
    ];
    egraph.rebuild(); // might be unnecessary
    let mut runner: egg::Runner<StitchLang, StitchAnalysis> = egg::Runner::default();
    runner = runner.with_egraph(egraph)
        .with_iter_limit(10)
        .run(&rules);

    runner.egraph.rebuild(); // might be unnecessary
    (runner.egraph, root)
}

/// Prints a programs term with each child on a new line.
/// If the term is not a programs node, prints it normally.
pub fn print_programs(term: &egg::RecExpr<StitchLang>) {
    let root_node = &term.as_ref()[term.as_ref().len() - 1];
    if root_node.op.as_str() == "programs" {
        println!("(programs");
        for &child_id in &root_node.children {
            print!("  ");
            print_expr(term, child_id.into());
            println!();
        }
        println!(")");
    } else {
        println!("{}", term);
    }
}

/// Recursively prints an s-expression starting from the given node id.
fn print_expr(term: &egg::RecExpr<StitchLang>, id: usize) {
    let node = &term.as_ref()[id];
    if node.children.is_empty() {
        print!("{}", node.op);
    } else {
        print!("({}", node.op);
        for &child_id in &node.children {
            print!(" ");
            print_expr(term, child_id.into());
        }
        print!(")");
    }
}
