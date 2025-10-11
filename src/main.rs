mod lang;

use lang::StitchLang;

fn main() {
    let (egraph, root) = load_egraph::<StitchLang>("data/domains/simple-arithmetic/aplusbplusc.json");
    let extractor = egg::Extractor::new(&egraph, egg::AstSize);
    let (_, term) = extractor.find_best(root);
    println!("{}", term);
}

/// Loads a JSON file containing s-expressions and builds an egraph from them.
/// All programs are combined into a single term (programs A B C ...).
/// Returns the egraph and the root e-class Id of the programs node.
fn load_egraph<L: egg::Language + egg::FromOp>(filename: &str) -> (egg::EGraph<L, ()>, egg::Id) {
    let contents = std::fs::read_to_string(filename).expect("Failed to read file");
    let exprs: Vec<String> = serde_json::from_str(&contents).expect("Failed to parse JSON");

    let mut egraph = egg::EGraph::default();
    let mut expr_ids = Vec::new();

    for expr_str in &exprs {
        let expr: egg::RecExpr<L> = expr_str.parse().expect("Failed to parse expression");
        expr_ids.push(egraph.add_expr(&expr));
    }

    let programs_node = L::from_op("programs", expr_ids).expect("Failed to create programs node");
    let root = egraph.add(programs_node);
    egraph.rebuild();
    (egraph, root)
}

pub struct SharedSearchData<L: egg::Language> {
    pub egraph: egg::EGraph<L, ()>,
}

#[derive(Debug)]
pub struct MatchAtEClass {
    pub root_eclass: egg::Id,
    // variables[i][j] represents the j'th variable in the i'th way to match the pattern
    pub variables: Vec<egg::Subst>,

}

#[derive(Debug)]
pub struct SearchState<L: egg::Language> {
    pattern: egg::PatternAst<L>,
    // each match represents a different eclass at which `pattern` can be rooted
    matches: Vec<MatchAtEClass>,
}

impl <L: egg::Language + egg::FromOp> SearchState<L> {
    pub fn empty(shared: &SharedSearchData<L>) -> Self {
        let recexpr: egg::PatternAst<L> = "?#0".parse().unwrap();
        Self {
            pattern: recexpr,
            matches: shared.egraph.classes().map(|c| MatchAtEClass {
                root_eclass: c.id,
                variables: vec![egg::Subst::default()],
            }).collect(),
        }
    }
}
