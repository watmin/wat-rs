# EXPECTATIONS v2 — the depth budget and the quote door

⚠ Supersedes v1. The mutation row is different: the probe now asserts **our** refusal.

| what | command | expected |
|---|---|---|
| ONE shared budget, not per-function | read the diff | every mutually-recursive `lower_*` calls the same method and shadows its depth |
| the bound is bound, not restated | `grep -c '512' src/rete/expr_ir/mod.rs` | **0** — `EXPANSION_DEPTH_LIMIT` is imported |
| refusal is a value | `grep -c 'panic!\|unwrap()\|expect(' <the new code>` | 0 |
| ⛔ **the abort is GONE at 2 MiB** | `bash -c 'ulimit -s 2048; wat <3000-deep quoted fixture>'` | a clean refusal naming the depth. **NOT rc=134** |
| the deep case is refused on 8 MB too | same fixture, normal stack | refusal, not silent acceptance |
| **the compile path is unchanged** | 509-deep *source* (not quoted) fixture | still compiles — this is STOP-2's tripwire |
| 510-deep source still refused by the expander | as before | `ExpansionDepthExceeded`, not our new error |
| the probe names the refusal | `grep -c 'depth' tests/rete/probe_arc278_lower_depth_shield.rs` | ≥1, and it asserts the kind, not just "an error" |
| **MUTATION — the guard is load-bearing** | raise the `LowerCx` bound to `100_000`, re-run the probe under `ulimit -s 2048` | **rc=134 / abort returns.** Restore ⇒ clean refusal |
| floor | `scripts/floor.sh` | 5480 + the probe's tests, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

45–70 min. Two floors dominate; the guard itself is small.

## Trap doors

- **A per-function counter.** The contract decision, and `export.rs` says why in prose it wrote
  after being bitten.
- **Copying `MAX_IMPORT_DEPTH = 300`.** It would refuse source that compiles today — a regression
  wearing a guard's clothes — and its headroom was measured on `unpack_expr`'s frames, which are
  **half** as deep-going as `lower`'s (3,000–5,000 vs the ~1,530 measured here).
- **Testing only on the 8 MB main stack.** It accepts **50,000** there. A probe that does not
  constrain the stack proves nothing about the abort.
- **Asserting "it failed".** A deep malformed fixture fails for many reasons. Assert the depth
  refusal by kind.
- **Believing this brief.** Everything here was measured once, by one hand, on one box. Reproduce
  the 1520/1539 pair before you lean on it; if it moves, say so — that is worth more than the strike.
