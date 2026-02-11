mod commands;
mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "prop-amm", about = "Prop AMM Challenge CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build program (native for simulation, BPF for submission)
    Build {
        /// Path to the program crate directory
        path: String,
    },
    /// Validate a BPF .so program (convexity, monotonicity, CU)
    Validate {
        /// Path to the compiled BPF .so file
        so_path: String,
    },
    /// Run simulation batch
    Run {
        /// Path to the program crate directory
        path: String,
        /// Number of simulations
        #[arg(long, default_value = "1000")]
        simulations: u32,
        /// Number of steps per simulation
        #[arg(long, default_value = "10000")]
        steps: u32,
        /// Number of parallel workers (0 = auto)
        #[arg(long, default_value = "0")]
        workers: usize,
        /// Use BPF runtime instead of native (slower, for validation)
        #[arg(long)]
        bpf: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { path } => commands::build::run(&path),
        Commands::Validate { so_path } => commands::validate::run(&so_path),
        Commands::Run {
            path,
            simulations,
            steps,
            workers,
            bpf,
        } => commands::run::run(&path, simulations, steps, workers, bpf),
    }
}
