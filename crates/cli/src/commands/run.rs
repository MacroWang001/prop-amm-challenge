use prop_amm_executor::{AfterSwapFn, SwapFn};
use prop_amm_shared::normalizer::{after_swap as normalizer_after_swap_fn, compute_swap as normalizer_swap};
use prop_amm_sim::runner;

use crate::output;

pub fn run(
    lib_path: &str,
    simulations: u32,
    steps: u32,
    workers: usize,
) -> anyhow::Result<()> {
    let swap_fn = load_native_swap(lib_path)?;
    let after_swap_fn = load_native_after_swap(lib_path);
    let n_workers = if workers == 0 { None } else { Some(workers) };

    println!(
        "Running {} simulations ({} steps each)...",
        simulations, steps,
    );

    let start = std::time::Instant::now();
    let result = runner::run_default_batch_native(
        swap_fn,
        after_swap_fn,
        normalizer_swap,
        Some(normalizer_after_swap_fn),
        simulations,
        steps,
        n_workers,
    )?;
    let elapsed = start.elapsed();

    output::print_results(&result, elapsed);
    Ok(())
}

fn load_native_swap(path: &str) -> anyhow::Result<SwapFn> {
    unsafe {
        let lib = libloading::Library::new(path)
            .map_err(|e| anyhow::anyhow!("Failed to load native library {}: {}", path, e))?;
        let func: libloading::Symbol<unsafe extern "C" fn(*const u8, usize) -> u64> =
            lib.get(b"compute_swap_ffi")
                .map_err(|_| anyhow::anyhow!(
                    "Symbol 'compute_swap_ffi' not found in {}. \
                     Make sure your program exports it — see README.",
                    path
                ))?;
        let raw_ptr = *func as usize;
        std::mem::forget(lib);
        SWAP_FN_PTR.store(raw_ptr, std::sync::atomic::Ordering::SeqCst);
        Ok(native_swap_wrapper)
    }
}

fn load_native_after_swap(path: &str) -> Option<AfterSwapFn> {
    unsafe {
        let lib = libloading::Library::new(path).ok()?;
        let func: libloading::Symbol<unsafe extern "C" fn(*const u8, usize, *mut u8, usize)> =
            lib.get(b"after_swap_ffi").ok()?;
        let raw_ptr = *func as usize;
        std::mem::forget(lib);
        AFTER_SWAP_FN_PTR.store(raw_ptr, std::sync::atomic::Ordering::SeqCst);
        Some(native_after_swap_wrapper)
    }
}

static SWAP_FN_PTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static AFTER_SWAP_FN_PTR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn native_swap_wrapper(data: &[u8]) -> u64 {
    let ptr = SWAP_FN_PTR.load(std::sync::atomic::Ordering::SeqCst);
    unsafe {
        let func: unsafe extern "C" fn(*const u8, usize) -> u64 = std::mem::transmute(ptr);
        func(data.as_ptr(), data.len())
    }
}

fn native_after_swap_wrapper(data: &[u8], storage: &mut [u8]) {
    let ptr = AFTER_SWAP_FN_PTR.load(std::sync::atomic::Ordering::SeqCst);
    unsafe {
        let func: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) = std::mem::transmute(ptr);
        func(data.as_ptr(), data.len(), storage.as_mut_ptr(), storage.len())
    }
}
