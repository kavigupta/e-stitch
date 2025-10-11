mod lang;
mod util;
mod pattern;
mod search;

use lang::StitchLang;
use pattern::Pattern;
use search::{SharedSearchData, SearchState};

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

