# Confusion Log

Purpose: track user-facing confusion points while developing strategies, with status based on current `main` behavior.

## Open Confusions

### C1. Server holdout seed schedule is intentionally undisclosed
- What is confusing: users can now control local seed ranges, but cannot verify if the server's hidden schedule resembles their local test windows.
- Current status: expected and acceptable for anti-overfitting, but still a discoverability gap for users trying to estimate variance.

## Resolved On Main / This PR

### R1. Unsafe/manual FFI template mismatch
- Previous confusion: older workflows used manual `compute_swap_ffi` and `after_swap_ffi` with unsafe blocks.
- Current status on `main`: resolved by safe submission scaffolding.
- Evidence:
  - README now documents safe-Rust submissions and auto-generated native adapter exports.
  - compile path auto-injects native shim exports in `/Users/dan/Projects/amm-puzzle/prop-amm-challenge/crates/cli/src/commands/compile.rs`.
  - starter template uses SDK helpers and no manual native FFI exports.

### R2. Unsafe workaround in local tooling
- Previous confusion: local workaround was needed before main updates.
- Current status on `main`: no workaround needed; keep submissions safe-Rust and rely on scaffolded adapters.

### R3. No CLI seed controls for out-of-sample runs
- Resolution: `prop-amm run` now supports `--seed-start` and `--seed-stride`, and prints the effective seed range.

### R4. Requirement wording mismatch (concavity vs convexity)
- Resolution: validator and docs now use concavity language consistently for shape checks.

### R5. Missing native/BPF parity check in validation
- Resolution: `prop-amm validate` now runs a deterministic native vs BPF parity batch and fails on delta above tolerance.

### R6. Run timing ambiguity
- Resolution: CLI run output now breaks out compile/load time, simulation time, and total time.
