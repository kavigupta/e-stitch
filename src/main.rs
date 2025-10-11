mod lang;
mod util;

use egg::ENodeOrVar;
use lang::StitchLang;
use egg::Language; // to put traits in view bc its such a pain otherwise


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

#[derive(Debug)]
pub struct Pattern {
    pub pattern: egg::PatternAst<StitchLang>,
    pub holes: Vec<egg::Id>,
    pub max_var: u32, // not same as arity because can expand away a var
}

impl Pattern {
    /// Creates the initial #?0 pattern which is just a single hole
    pub fn single_hole() -> Self {
        Pattern {
            pattern: "#?0".parse().unwrap(),
            holes: vec![0.into()],
            max_var: 0,
        }
    }

    /// Inserts a node at the given Id into the pattern, and increments the ids of the nodes
    /// of all children after the inserted node by 1
    /// Assumes if you're pushing a hole, you'll do that after calling this function not before
    pub fn insert(&mut self, i: egg::Id, node: egg::ENodeOrVar<StitchLang>) {
        let i: usize = i.into();
        self.pattern.nodes.insert(i, node);
        for node in &mut self.pattern.nodes[i+1..] { // upshift ones above i
            node.children_mut().iter_mut().for_each(|child| {
                *child = egg::Id::from(1 + usize::from(*child));
            });
        }
        for hole in &mut self.holes {
            if *hole >= i.into() { // we wanna upshift the ones AT i to for holes
                *hole = egg::Id::from(1 + usize::from(*hole));
            }
        }
    }

    /// Expands the pattern at the given Id with the given node
    pub fn expand(&mut self, i: egg::Id, node: &StitchLang) {
        let mut new_node = node.clone();
        let num_holes = new_node.len();
        for j in 0..num_holes {
            let arg_id: egg::Id = j.into();
            self.max_var += 1;
            let arg_idx = self.max_var;
            let arg_node = ENodeOrVar::Var(egg::Var::from(arg_idx));
            new_node.children[j] = arg_id;
            self.insert(arg_id, arg_node);
            self.holes.push(arg_id);
        }
        self.pattern.nodes[usize::from(i) + num_holes] = ENodeOrVar::ENode(new_node);
    }
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
