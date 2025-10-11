use crate::lang::StitchLang;
use egg::{Language, ENodeOrVar};

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