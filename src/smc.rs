use crate::lang::StitchLang;
use crate::pattern::Pattern;
use crate::search::{SearchState, SharedSearchData};
use rand::Rng;

pub fn smc(egraph: egg::EGraph<StitchLang, ()>, root: egg::Id) -> Option<(usize, SearchState)> {
    let shared = SharedSearchData { egraph };

    let num_particles = 10;
    let num_steps = 10;
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

        let mut weights: Vec<f64> = costs.iter().map(|cost| (-(*cost as f64)/temperature).exp()).collect();

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

    let (cost, term) = rewrite(&shared.egraph, root, &best_so_far.as_ref().unwrap().1);
    println!("best: {}", cost);
    crate::util::print_programs(&term);

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
    egraph: &egg::EGraph<StitchLang, ()>,
    root: egg::Id,
    search_state: &SearchState,
) -> usize {
    let (cost, _) = rewrite(egraph, root, search_state);
    return cost;
}

pub fn rewrite(
    egraph: &egg::EGraph<StitchLang, ()>,
    root: egg::Id,
    search_state: &SearchState,
) -> (usize, egg::RecExpr<StitchLang>) {
    let mut egraph = egraph.clone(); // todo be smarter

    // println!("search state: {}", search_state);
    for m in &search_state.matches {
        // println!("match at eclass {}: {:?}", m.root_eclass, m.substs);
        for subst in &m.substs {
            let node: StitchLang = StitchLang {
                op: "inv_0".into(),
                children: subst.vars.clone(),
            };
            let x = egraph.add(node);
            egraph.union(x, m.root_eclass);
        }
    }
    egraph.rebuild();
    let extractor = egg::Extractor::new(&egraph, egg::AstSize);
    let (cost, term) = extractor.find_best(root);
    return (cost, term);
}
