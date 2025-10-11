use crate::lang::StitchLang;
use egg::{Language, ENodeOrVar, Id};

#[derive(Debug)]
pub struct Pattern {
    pub pattern: egg::PatternAst<StitchLang>,
    pub holes: Vec<Id>,
    pub max_var: u32, // not same as arity because can expand away a var
}

impl Pattern {
    /// Creates the initial #?0 pattern which is just a single hole
    pub fn single_hole() -> Self {
        // annoyingly parsing "#?0" doesn't create a ENodeOrVar::Var it creates an ENodeOrVar::ENode
        let rec_expr: egg::RecExpr<ENodeOrVar<StitchLang>>  = vec![ENodeOrVar::Var(egg::Var::from(0))].into();
        Pattern {
            pattern: rec_expr,
            holes: vec![0.into()],
            max_var: 0,
        }
    }

    /// Inserts a node at the given Id into the pattern, and increments the ids of the nodes
    /// of all children after the inserted node by 1
    pub fn insert(&mut self, i: Id, node: egg::ENodeOrVar<StitchLang>) {
        let i: usize = i.into();
        for node in &mut self.pattern.nodes {
            node.children_mut().iter_mut().for_each(|child| {
                if *child >= i.into() {
                    *child = Id::from(1 + usize::from(*child));
                }
            });
        }
        for hole in &mut self.holes {
            if *hole >= i.into() {
                *hole = Id::from(1 + usize::from(*hole));
            }
        }
        self.pattern.nodes.insert(i, node);
    }

    pub fn new_hole(&mut self, i: Id) {
        self.max_var += 1;
        let arg_node = ENodeOrVar::Var(egg::Var::from(self.max_var));
        self.insert(i, arg_node);
        self.holes.push(i); // must happen after insert, since insert upshifts holes
    }

    /// Expands the pattern at the given Id with the given node
    pub fn expand(&mut self, hole_idx: usize, node: &StitchLang) {
        let hole = self.holes.remove(hole_idx);
        let i = usize::from(hole);
        let mut new_node = node.clone();
        let num_holes = new_node.len();
        for j in 0..num_holes {
            let hole_id = Id::from(j);
            self.new_hole(hole_id);
            new_node.children[j] = hole_id;
        }
        let replace_at = usize::from(i) + num_holes;
        assert!(matches!(self.pattern.nodes[replace_at], ENodeOrVar::Var(_)), "Attempting to expand a non-var");
        self.pattern.nodes[replace_at] = ENodeOrVar::ENode(new_node);
    }
}