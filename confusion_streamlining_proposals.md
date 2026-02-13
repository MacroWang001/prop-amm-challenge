# Confusion Streamlining Proposals

This file maps open confusion IDs in `/Users/dan/Projects/amm-puzzle/prop-amm-challenge/confusion_log.md` to concrete improvements.

## Implemented In This PR

- Safe submission scaffolding and auto-generated native glue (covers prior unsafe/manual FFI confusion).
- Starter template and README now aligned with safe-Rust submission flow.
- `prop-amm run` supports `--seed-start` and `--seed-stride`, and prints seed range.
- CLI run output now separates compile/load time, simulation time, and total time.
- Validation messages now use concavity terminology consistently.
- `prop-amm validate` now performs a deterministic native/BPF parity check and emits diagnostics.

## Remaining Documentation + Product Clarifications

### D1. Clarify server holdout policy boundaries (addresses C1)
- Update README with an explicit "Reproducibility" section:
  - local default seed schedule is deterministic (`0..n_sims-1`),
  - server uses a different evaluation seed schedule,
  - exact scores are expected to differ slightly.
- Keep holdout seeds private, but describe policy shape clearly.

### D2. Publish high-level server evaluation invariants (addresses C1)
- Document (without revealing seed values):
  - sims per submission,
  - whether evaluation seed schedule is fixed per checker version,
  - whether final score is from a single block or averaged blocks.

## Optional Future Enhancements

1. Add a machine-readable validation report format (JSON) that includes parity metrics.
2. Expose checker/version metadata in submission responses for easier result attribution.
