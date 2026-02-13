use std::sync::atomic::{AtomicPtr, Ordering};

use prop_amm_executor::{AfterSwapFn, BpfProgram};
use prop_amm_shared::normalizer::{
    after_swap as normalizer_after_swap_fn, compute_swap as normalizer_swap,
};
use prop_amm_sim::runner;

use super::compile;
use crate::output;

type FfiSwapFn = unsafe extern "C" fn(*const u8, usize) -> u64;
type FfiAfterSwapFn = unsafe extern "C" fn(*const u8, usize, *mut u8, usize);

static LOADED_SWAP: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());
static LOADED_AFTER_SWAP: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

fn dynamic_swap(data: &[u8]) -> u64 {
    let ptr = LOADED_SWAP.load(Ordering::Relaxed);
    let f: FfiSwapFn = unsafe { std::mem::transmute(ptr) };
    unsafe { f(data.as_ptr(), data.len()) }
}

fn dynamic_after_swap(data: &[u8], storage: &mut [u8]) {
    let ptr = LOADED_AFTER_SWAP.load(Ordering::Relaxed);
    let f: FfiAfterSwapFn = unsafe { std::mem::transmute(ptr) };
    unsafe {
        f(
            data.as_ptr(),
            data.len(),
            storage.as_mut_ptr(),
            storage.len(),
        )
    }
}

pub fn run(
    file: &str,
    simulations: u32,
    steps: u32,
    workers: usize,
    seed_start: u64,
    seed_stride: u64,
    bpf: bool,
    bpf_so: Option<&str>,
) -> anyhow::Result<()> {
    if seed_stride == 0 {
        anyhow::bail!("--seed-stride must be >= 1");
    }
    let n_workers = if workers == 0 { None } else { Some(workers) };

    if bpf {
        run_bpf(
            file,
            simulations,
            steps,
            n_workers,
            bpf_so,
            seed_start,
            seed_stride,
        )
    } else {
        run_native(file, simulations, steps, n_workers, seed_start, seed_stride)
    }
}

fn run_native(
    file: &str,
    simulations: u32,
    steps: u32,
    n_workers: Option<usize>,
    seed_start: u64,
    seed_stride: u64,
) -> anyhow::Result<()> {
    let total_start = std::time::Instant::now();
    println!("Compiling {} (native)...", file);
    let build_start = std::time::Instant::now();
    let native_path = compile::compile_native(file)?;
    let build_elapsed = build_start.elapsed();

    // Load the native library — leak it so symbols remain valid for the process lifetime.
    let load_start = std::time::Instant::now();
    let lib = Box::new(
        unsafe { libloading::Library::new(&native_path) }
            .map_err(|e| anyhow::anyhow!("Failed to load {}: {}", native_path.display(), e))?,
    );
    let lib = Box::leak(lib);

    let swap_fn: libloading::Symbol<FfiSwapFn> = unsafe {
        lib.get(compile::NATIVE_SWAP_SYMBOL)
            .or_else(|_| lib.get(b"compute_swap_ffi"))
    }
    .map_err(|e| anyhow::anyhow!("Missing native swap symbol: {}", e))?;
    LOADED_SWAP.store(*swap_fn as *mut (), Ordering::Relaxed);

    let has_after_swap = if let Ok(after_fn) = unsafe {
        lib.get::<FfiAfterSwapFn>(compile::NATIVE_AFTER_SWAP_SYMBOL)
            .or_else(|_| lib.get::<FfiAfterSwapFn>(b"after_swap_ffi"))
    } {
        LOADED_AFTER_SWAP.store(*after_fn as *mut (), Ordering::Relaxed);
        true
    } else {
        false
    };

    let submission_after_swap: Option<AfterSwapFn> = if has_after_swap {
        Some(dynamic_after_swap)
    } else {
        None
    };
    let compile_or_load_elapsed = build_elapsed + load_start.elapsed();

    println!(
        "Running {} simulations ({} steps each) natively with seeds {} + i*{}...",
        simulations, steps, seed_start, seed_stride,
    );

    let sim_start = std::time::Instant::now();
    let result = runner::run_default_batch_native_seeded(
        dynamic_swap,
        submission_after_swap,
        normalizer_swap,
        Some(normalizer_after_swap_fn),
        simulations,
        steps,
        n_workers,
        seed_start,
        seed_stride,
    )?;
    let sim_elapsed = sim_start.elapsed();

    output::print_results(
        &result,
        output::RunTimings {
            compile_or_load: compile_or_load_elapsed,
            simulation: sim_elapsed,
            total: total_start.elapsed(),
        },
    );
    Ok(())
}

fn run_bpf(
    file: &str,
    simulations: u32,
    steps: u32,
    n_workers: Option<usize>,
    bpf_so: Option<&str>,
    seed_start: u64,
    seed_stride: u64,
) -> anyhow::Result<()> {
    let total_start = std::time::Instant::now();
    let build_or_load_start = std::time::Instant::now();
    let bpf_path = if let Some(path) = bpf_so {
        println!("Using prebuilt BPF .so: {}", path);
        std::path::PathBuf::from(path)
    } else {
        println!("Compiling {} (BPF)...", file);
        compile::compile_bpf(file)?
    };

    let bytes = std::fs::read(&bpf_path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", bpf_path.display(), e))?;
    let submission_program = BpfProgram::load(&bytes)
        .map_err(|e| anyhow::anyhow!("Failed to load BPF program: {}", e))?;
    let compile_or_load_elapsed = build_or_load_start.elapsed();

    let meter_disabled = std::env::var_os("PROP_AMM_BPF_DISABLE_METER").is_some();

    println!(
        "Running {} simulations ({} steps each) via BPF{}{} with seeds {} + i*{}...",
        simulations,
        steps,
        if submission_program.jit_available() {
            " (JIT)"
        } else {
            " (interpreter)"
        },
        if meter_disabled { " (no meter)" } else { "" },
        seed_start,
        seed_stride,
    );

    let sim_start = std::time::Instant::now();
    let result = runner::run_default_batch_mixed_seeded(
        submission_program,
        normalizer_swap,
        Some(normalizer_after_swap_fn),
        simulations,
        steps,
        n_workers,
        seed_start,
        seed_stride,
    )?;
    let sim_elapsed = sim_start.elapsed();

    output::print_results(
        &result,
        output::RunTimings {
            compile_or_load: compile_or_load_elapsed,
            simulation: sim_elapsed,
            total: total_start.elapsed(),
        },
    );
    Ok(())
}
