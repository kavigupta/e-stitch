/// Like an egg::RecExpr but with the nodes in reverse order
/// and publicly accessible
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RevExpr<L: egg::Language> {
    pub nodes: Vec<L>,
}

impl<L: egg::Language> RevExpr<L> {
    pub fn new(nodes: Vec<L>) -> Self {
        Self { nodes }
    }
}

impl<L: egg::FromOp> std::str::FromStr for RevExpr<L> {
    type Err = egg::RecExprParseError<L::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let recexpr = s.parse::<egg::RecExpr<L>>()?;
        Ok(recexpr.into())
    }
}

impl<L: egg::Language + std::fmt::Display> std::fmt::Display for RevExpr<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // somewhat silly clone now but it's okay – display isn't performance critical and isn't a huge clone
        let recexpr: egg::RecExpr<L> = self.clone().into();
        std::fmt::Display::fmt(&recexpr, f)
    }
}


impl<L: egg::Language> From<RevExpr<L>> for egg::RecExpr<L> {
    fn from(rev_expr: RevExpr<L>) -> Self {
        let mut nodes: Vec<L> = rev_expr.nodes;
        nodes.reverse();
        egg::RecExpr::from(nodes)
    }
}

impl<L: egg::Language> From<egg::RecExpr<L>> for RevExpr<L> {
    fn from(recexpr: egg::RecExpr<L>) -> Self {
        let mut nodes: Vec<L> = recexpr.into();
        nodes.reverse();
        RevExpr::new(nodes)
    }
}
