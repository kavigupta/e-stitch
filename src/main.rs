mod lang;
mod util;
mod pattern;
mod search;
mod revexpr;

use lang::StitchLang;
use pattern::Pattern;
use search::{SharedSearchData, SearchState};

fn main() {
    let (egraph, root) = util::load_egraph("data/domains/simple-arithmetic/aplusbplusc.json");
    let extractor = egg::Extractor::new(&egraph, egg::AstSize);
    let (_, term) = extractor.find_best(root);
    util::print_programs(&term);

    let mut pattern = Pattern::single_var();
    println!("before expansion: {:?}", pattern.pattern.nodes);
    println!("before expansion: {}", pattern.pattern);

    pattern.expand(0, &StitchLang{op: "+".into(), children: vec![2.into(), 3.into()]});
    println!("after expansion: {:?}", pattern.pattern.nodes);
    println!("after expansion: {}", pattern.pattern);

    let shared = SharedSearchData { egraph };
    let mut search_state = SearchState::new(&shared);
    println!("search state: {}", search_state);

    println!("****************************");

    while search_state.pattern.vars.len() > 0 {
        search_state.expand_random(&shared);
        println!("cost: {}", compute_cost(shared.egraph.clone(), root, &search_state));
        
    }


}


fn compute_cost(egraph: egg::EGraph<StitchLang, ()>, root: egg::Id, search_state: &SearchState) -> usize {
    let mut egraph = egraph;
    println!("search state: {}", search_state);
    for m in &search_state.matches {
        // println!("match at eclass {}: {:?}", m.root_eclass, m.substs);
        for subst in &m.substs {
            let node: StitchLang = StitchLang { op: "inv_0".into(), children: subst.vars.clone() };
            let x = egraph.add(node);
            egraph.union(x, m.root_eclass);
        }
    }
    egraph.rebuild();
    let extractor = egg::Extractor::new(&egraph, egg::AstSize);
    let (cost, term) = extractor.find_best(root);
    util::print_programs(&term);
    cost
}