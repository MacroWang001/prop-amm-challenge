use prop_amm_executor::{BpfExecutor, BpfProgram};
use prop_amm_shared::instruction::STORAGE_SIZE;
use prop_amm_shared::nano::{f64_to_nano, nano_to_f64};

pub fn run(so_path: &str) -> anyhow::Result<()> {
    println!("Validating program: {}", so_path);

    // Load ELF
    let elf_bytes = std::fs::read(so_path)?;
    let program = BpfProgram::load(&elf_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to load ELF: {}", e))?;
    println!("  [PASS] ELF loaded and verified");

    let mut executor = BpfExecutor::new(program);
    let storage = [0u8; STORAGE_SIZE];

    // Basic execution test
    let rx = f64_to_nano(100.0);
    let ry = f64_to_nano(10000.0);

    let buy_output = executor.execute(0, f64_to_nano(10.0), rx, ry, &storage)
        .map_err(|e| anyhow::anyhow!("Buy execution failed: {}", e))?;
    if buy_output == 0 {
        anyhow::bail!("FAIL: Buy X returned zero output");
    }
    println!("  [PASS] Buy X: input_y=10.0 -> output_x={:.6}", nano_to_f64(buy_output));

    let sell_output = executor.execute(1, f64_to_nano(1.0), rx, ry, &storage)
        .map_err(|e| anyhow::anyhow!("Sell execution failed: {}", e))?;
    if sell_output == 0 {
        anyhow::bail!("FAIL: Sell X returned zero output");
    }
    println!("  [PASS] Sell X: input_x=1.0 -> output_y={:.6}", nano_to_f64(sell_output));

    // Monotonicity check: larger input -> larger output
    println!("  Checking monotonicity...");
    let trade_sizes = [0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0];

    // Buy side monotonicity
    let mut prev_output = 0u64;
    for &size in &trade_sizes {
        let output = executor.execute(0, f64_to_nano(size), rx, ry, &storage)
            .map_err(|e| anyhow::anyhow!("Execution failed at size {}: {}", size, e))?;
        if output <= prev_output && prev_output > 0 {
            anyhow::bail!(
                "FAIL: Monotonicity violation (buy side). size={} output={} <= prev_output={}",
                size, output, prev_output
            );
        }
        prev_output = output;
    }
    println!("  [PASS] Buy side monotonicity");

    // Sell side monotonicity
    prev_output = 0;
    for &size in &trade_sizes {
        let output = executor.execute(1, f64_to_nano(size), rx, ry, &storage)
            .map_err(|e| anyhow::anyhow!("Execution failed at size {}: {}", size, e))?;
        if output <= prev_output && prev_output > 0 {
            anyhow::bail!(
                "FAIL: Monotonicity violation (sell side). size={} output={} <= prev_output={}",
                size, output, prev_output
            );
        }
        prev_output = output;
    }
    println!("  [PASS] Sell side monotonicity");

    // Convexity check: marginal price must worsen with size
    println!("  Checking convexity...");
    let eps = 0.001;

    // Buy side convexity: marginal X output per Y input should decrease
    let mut prev_marginal = f64::MAX;
    for &size in &trade_sizes {
        let out_lo = nano_to_f64(executor.execute(0, f64_to_nano(size), rx, ry, &storage)?);
        let out_hi = nano_to_f64(executor.execute(0, f64_to_nano(size + eps), rx, ry, &storage)?);
        let marginal = (out_hi - out_lo) / eps;

        if marginal > prev_marginal + 1e-9 {
            anyhow::bail!(
                "FAIL: Convexity violation (buy side). At size={}, marginal={:.9} > prev={:.9}",
                size, marginal, prev_marginal
            );
        }
        prev_marginal = marginal;
    }
    println!("  [PASS] Buy side convexity");

    // Sell side convexity: marginal Y output per X input should decrease
    prev_marginal = f64::MAX;
    for &size in &trade_sizes {
        let out_lo = nano_to_f64(executor.execute(1, f64_to_nano(size), rx, ry, &storage)?);
        let out_hi = nano_to_f64(executor.execute(1, f64_to_nano(size + eps), rx, ry, &storage)?);
        let marginal = (out_hi - out_lo) / eps;

        if marginal > prev_marginal + 1e-9 {
            anyhow::bail!(
                "FAIL: Convexity violation (sell side). At size={}, marginal={:.9} > prev={:.9}",
                size, marginal, prev_marginal
            );
        }
        prev_marginal = marginal;
    }
    println!("  [PASS] Sell side convexity");

    println!("\nAll validation checks passed!");
    Ok(())
}
