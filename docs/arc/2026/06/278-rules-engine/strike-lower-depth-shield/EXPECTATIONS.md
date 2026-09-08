# EXPECTATIONS — recording and gating the depth shield

| what | command | expected |
|---|---|---|
| lowering logic untouched | `git diff -U0 -- src/rete/expr_ir/mod.rs` | every `+`/`-` line begins `//` |
| no new depth counter | `grep -cE 'depth' src/rete/expr_ir/mod.rs` | >0 in comments, **0** in code |
| the shield is named | `grep -c 'EXPANSION_DEPTH_LIMIT' src/rete/expr_ir/mod.rs` | ≥1 |
| the bypass is named | `grep -c 'MAX_IMPORT_DEPTH\|unpack_expr' src/rete/expr_ir/mod.rs` | ≥1 |
| the probe does not pin 509 | `grep -c '509\|510' tests/rete/probe_arc278_lower_depth_shield.rs` | **0** |
| the probe binds the constant | `grep -c 'EXPANSION_DEPTH_LIMIT' tests/rete/probe_arc278_lower_depth_shield.rs` | ≥1 |
| the refusal is named, not just "an error" | `grep -c 'ExpansionDepthExceeded' tests/rete/probe_arc278_lower_depth_shield.rs` | ≥1 |
| the probe passes | `cargo nextest run --release -E 'test(lower_depth_shield)'` | green, both arms |
| **MUTATION — the gate can fail** | raise `EXPANSION_DEPTH_LIMIT` to `1024`, re-run the probe | **RED**, naming the accepted-too-deep arm. Restore, green |
| floor | `scripts/floor.sh` | 5481 (5480 + the new probe's tests), **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

30–45 min, most of it the two floors.

## Trap doors

- **Pinning 509.** The contract decision. A test that hard-codes it reddens on any harmless
  re-wrapping of the generated source and teaches the next hand to bump a number instead of asking
  what moved. Bind `EXPANSION_DEPTH_LIMIT` and express the wall relative to it.
- **Asserting "it was refused" without naming the kind.** The deep case can fail for a dozen
  unrelated reasons — a malformed generated form most likely. A green that does not name
  `ExpansionDepthExceeded` proves nothing about the shield.
- **Building a 15 KB `.wat` fixture.** It would also land in a gated tree and has to *fail* to load,
  which is a fight with two other gates. Generate the source in the test.
- **Believing the row.** `2W1` says this path aborts. It does not. If your driving disagrees with
  this brief rather than with the row, STOP — the brief is the thing that was measured, but it was
  measured once, by one hand, and it is a claim until you reproduce it.
