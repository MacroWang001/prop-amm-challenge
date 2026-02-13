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
        /// Path to the .rs source file
        file: String,
    },
    /// Validate a program (convexity, monotonicity, CU)
    Validate {
        /// Path to the .rs source file
        file: String,
    },
    /// Run simulation batch
    Run {
        /// Path to the .rs source file
        file: String,
        /// Number of simulations
        #[arg(long, default_value = "1000")]
        simulations: u32,
        /// Number of steps per simulation
        #[arg(long, default_value = "10000")]
        steps: u32,
        /// Number of parallel workers (0 = auto)
        #[arg(long, default_value = "0")]
        workers: usize,
        /// Starting seed for simulation config generation
        #[arg(long, default_value = "0")]
        seed_start: u64,
        /// Seed step between simulations
        #[arg(long, default_value = "1")]
        seed_stride: u64,
        /// Use BPF runtime instead of native (slower, for validation)
        #[arg(long)]
        bpf: bool,
        /// Path to a prebuilt BPF .so to use when running with --bpf (skips compilation).
        /// Useful on machines without the Solana SBF toolchain installed.
        #[arg(long)]
        bpf_so: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file } => commands::build::run(&file),
        Commands::Validate { file } => commands::validate::run(&file),
        Commands::Run {
            file,
            simulations,
            steps,
            workers,
            seed_start,
            seed_stride,
            bpf,
            bpf_so,
        } => commands::run::run(
            &file,
            simulations,
            steps,
            workers,
            seed_start,
            seed_stride,
            bpf,
            bpf_so.as_deref(),
        ),
    }
}
