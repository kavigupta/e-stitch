use crate::lang::StitchLang;
use crate::pattern::Pattern;


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
