use prop_amm_shared::result::BatchResult;
use std::time::Duration;

pub struct RunTimings {
    pub compile_or_load: Duration,
    pub simulation: Duration,
    pub total: Duration,
}

pub fn print_results(result: &BatchResult, timings: RunTimings) {
    let seed_range = result
        .results
        .iter()
        .map(|r| r.seed)
        .fold(None::<(u64, u64)>, |acc, seed| match acc {
            Some((lo, hi)) => Some((lo.min(seed), hi.max(seed))),
            None => Some((seed, seed)),
        });

    println!("\n========================================");
    println!("  Simulations: {}", result.n_sims());
    if let Some((seed_start, seed_end)) = seed_range {
        println!("  Seed range:  {}..={}", seed_start, seed_end);
    }
    println!(
        "  Compile/load:{:>8.2}s",
        timings.compile_or_load.as_secs_f64()
    );
    println!("  Simulation:  {:>8.2}s", timings.simulation.as_secs_f64());
    println!("  Total:       {:>8.2}s", timings.total.as_secs_f64());
    println!("  Avg edge:    {:.2}", result.avg_edge());
    println!("  Total edge:  {:.2}", result.total_edge);
    println!("========================================");

    if let Some(stats) = prop_amm_sim::search_stats::snapshot_if_enabled() {
        let arb_calls = stats.arb_golden_calls.max(1);
        let router_calls = stats.router_calls.max(1);
        println!("\nSearch stats (PROP_AMM_SEARCH_STATS=1):");
        println!(
            "  Arb golden:  calls={} iters={} (avg {:.2}/call) evals={} (avg {:.2}/call) early_stop_amount_tol={}",
            stats.arb_golden_calls,
            stats.arb_golden_iters,
            stats.arb_golden_iters as f64 / arb_calls as f64,
            stats.arb_golden_evals,
            stats.arb_golden_evals as f64 / arb_calls as f64,
            stats.arb_early_stop_amount_tol,
        );
        println!(
            "  Arb bracket: calls={} evals={} (avg {:.2}/call)",
            stats.arb_bracket_calls,
            stats.arb_bracket_evals,
            stats.arb_bracket_evals as f64 / stats.arb_bracket_calls.max(1) as f64,
        );
        println!(
            "  Router:     calls={} iters={} (avg {:.2}/call) evals={} (avg {:.2}/call) early_stop_rel_gap={}",
            stats.router_calls,
            stats.router_golden_iters,
            stats.router_golden_iters as f64 / router_calls as f64,
            stats.router_evals,
            stats.router_evals as f64 / router_calls as f64,
            stats.router_early_stop_rel_gap,
        );
    }
}
