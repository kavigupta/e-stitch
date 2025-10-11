mod lang;
mod util;
mod pattern;

use egg::ENodeOrVar;
use lang::StitchLang;
use egg::Language; // to put traits in view bc its such a pain otherwise
use pattern::Pattern;

fn main() {
    let (egraph, root) = util::load_egraph::<StitchLang>("data/domains/simple-arithmetic/aplusbplusc.json");
    let extractor = egg::Extractor::new(&egraph, egg::AstSize);
    let (_, term) = extractor.find_best(root);
    util::print_programs(&term);

    let mut pattern = Pattern::single_hole();
    println!("{}", pattern.pattern);
    println!("{:?}", pattern.pattern.nodes);
    

    let recexpr: egg::RecExpr<StitchLang> = "(+ 2 3)".parse().unwrap();
    println!("{}", recexpr);
    println!("{:?}", recexpr.nodes);

    pattern.expand(0.into(), &StitchLang{op: "+".into(), children: vec![2.into(), 3.into()]});
    println!("{}", pattern.pattern);
    println!("{:?}", pattern.pattern.nodes);

}



pub struct SharedSearchData {
    pub egraph: egg::EGraph<StitchLang, ()>,
}

#[derive(Debug)]
pub struct MatchAtEClass {
    pub root_eclass: egg::Id,
    // variables[i][j] represents the j'th variable in the i'th way to match the pattern
    pub variables: Vec<egg::Subst>,

}

#[derive(Debug)]
pub struct SearchState {
    pattern: Pattern,
    // each match represents a different eclass at which `pattern` can be rooted
    matches: Vec<MatchAtEClass>,
}

impl SearchState {
    pub fn empty(shared: &SharedSearchData) -> Self {
        Self {
            pattern: Pattern::single_hole(),
            matches: shared.egraph.classes().map(|c| MatchAtEClass {
                root_eclass: c.id,
                variables: vec![egg::Subst::default()],
            }).collect(),
        }
    }
}
