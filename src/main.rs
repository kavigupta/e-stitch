fn main() {
    let shared = SharedSearchData {
        egraph: egg::EGraph::<egg::SymbolLang, ()>::default(),
    };
    let state = SearchState::empty(&shared);
    println!("{:?}", state);
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
