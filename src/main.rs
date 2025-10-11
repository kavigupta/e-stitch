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
    println!("before expansion: {:?}", pattern.pattern.nodes);
    println!("before expansion: {}", pattern.pattern);

    pattern.expand(0, &StitchLang{op: "+".into(), children: vec![2.into(), 3.into()]});
    println!("after expansion: {:?}", pattern.pattern.nodes);
    println!("after expansion: {}", pattern.pattern);

}

