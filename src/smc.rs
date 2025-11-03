use std::cmp::min;

use crate::lang::StitchLang;
use crate::pattern::Pattern;
use crate::search::{SearchState, SharedSearchData};
use egg::{Analysis, Id};
use rand::Rng;

#[derive(Clone, Debug, Default)]
pub struct StitchAnalysis;

impl Analysis<StitchLang> for StitchAnalysis {
    type Data = u32;

    fn make(egraph: &mut egg::EGraph<StitchLang, Self>, enode: &StitchLang, _id: egg::Id) -> Self::Data {
        1 + enode.children.iter().map(|&child_id| egraph[child_id].data).sum::<u32>()
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> egg::DidMerge {
        if from < *to {
            *to = from;
            egg::DidMerge(true, false)
        } else if from == *to {
            egg::DidMerge(false, false)
        } else {
            // from = *to; but we don't do this because types; idk it seems like they don't want us to
            egg::DidMerge(false, true)
        }
    }
}

pub type StitchEgraph = egg::EGraph<StitchLang, StitchAnalysis>;

pub fn smc(egraph: StitchEgraph, root: egg::Id) -> Option<(usize, SearchState)> {
    let shared = SharedSearchData { egraph };

    let num_particles = 100;
    let num_steps = 1000;
    let temperature = 1.0;

    let mut best_so_far: Option<(usize, SearchState)> = None;

    // make a bunch of search states
    let mut search_states: Vec<SearchState> = (0..num_particles)
        .map(|i| SearchState::new(&shared))
        .collect();

    for i in 0..num_steps {
        for search_state in &mut search_states {
            search_state.expand_random(&shared);
        }

        let costs: Vec<usize> = search_states
            .iter()
            .map(|search_state| compute_cost(&shared.egraph, root, search_state))
            .collect();

        for (i, cost) in costs.iter().enumerate() {
            if best_so_far.as_ref().is_none_or(|best| *cost < best.0) {
                println!("new best: {} {}", cost, search_states[i].pattern);
                best_so_far = Some((*cost, search_states[i].clone()));
            }
        }


        let mut weights: Vec<f64> = costs.iter().map(|cost| (-(*cost as f64)/temperature)).collect();
        let max_weight = weights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        for w in &mut weights {
            *w = (*w - max_weight).exp();
        }

        // force no resampling of completed patterns
        for (i, state) in search_states.iter().enumerate() {
            if state.pattern.vars.is_empty() {
                weights[i] = 0.0;
            }
        }

        if weights.iter().sum::<f64>() == 0.0 {
            println!("all particles died, stopping");
            break;
        }

        // resample
        normalize_and_accumulate(&mut weights);
        search_states = (0..num_particles).map(|i| {
            let idx = weighted_choice(&weights);
            search_states[idx].clone()
        }).collect();
    }

    let (cost) = rewrite(&shared.egraph, root, &best_so_far.as_ref().unwrap().1);
    println!("best: {}", cost);
    // crate::util::print_programs(&term);

    return best_so_far;
}

pub fn weighted_choice(acc_weights: &[f64]) -> usize {
    // println!("Choosing from weights: {:?}", cum_weights);
    let r: f64 = rand::rng().random_range(0.0..1.0);
    // println!("r: {:?}", r);
    match acc_weights.binary_search_by(|&w| w.partial_cmp(&r).unwrap()) {
        Ok(idx) => idx,
        Err(idx) => idx, // it could be inserted at idx, which means it's <= cum_weights[idx]
    }
}

pub fn normalize_and_accumulate(weights: &mut Vec<f64>) {
    let weight_sum = weights.iter().sum::<f64>();
    if weight_sum == 0.0 {
        let len = weights.len();
        weights.fill(1.0 / len as f64);
    } else {
        weights.iter_mut().for_each(|w| *w /= weight_sum);
    }
    let mut accum = 0.0;
    for w in weights {
        accum += *w;
        *w = accum;
    }
}

pub fn compute_cost(
    egraph: &StitchEgraph,
    root: egg::Id,
    search_state: &SearchState,
) -> usize {
    let cost = rewrite(egraph, root, search_state);
    return cost;
}

pub fn rewrite(
    egraph: &StitchEgraph,
    root: egg::Id,
    search_state: &SearchState,
) -> usize {
    let mut egraph = egraph.clone(); // todo be smarter

    // println!("search state: {}", search_state);
    // let mut nodes = vec![];
    for m in &search_state.matches {
        // println!("match at eclass {}: {:?}", m.root_eclass, m.substs);
        for subst in &m.substs {
            let node: StitchLang = StitchLang {
                op: "inv_0".into(),
                children: subst.vars.clone(),
            };
            let x = egraph.add(node);
            egraph.union(x, m.root_eclass);
            // nodes.push(x);
        }
    }
    egraph.rebuild();
    // let extractor = egg::Extractor::new(&egraph, egg::AstSize);
    // let (cost, term) = extractor.find_best(root);
    // for n in nodes {
    //     egraph
    // }
    // assert_eq!(egraph[root].data as usize, cost);
    // println!("cost from extractor: {}", cost);
    // println!("cost from egraph: {}", egraph[root].data);
    let cost = egraph[root].data as usize;
    cost
    // return (cost, term);
}
