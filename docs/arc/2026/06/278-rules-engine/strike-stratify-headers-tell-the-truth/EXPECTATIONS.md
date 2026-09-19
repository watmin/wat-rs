# EXPECTATIONS — stratify header corrections

| what | command | expected |
|---|---|---|
| only comments changed | `git diff -U0 -- src/rete/kernel/stratify.rs wat/rete/oracle/stratify.wat` | every `+`/`-` line begins `//`, `///` or `;;` |
| the false `Mirrors` line is gone | `grep -n 'Mirrors .stratify-sweep' src/rete/kernel/stratify.rs` | no hit |
| the six TRUE claims are untouched | `git diff -- src/rete/kernel/stratify.rs \| grep -c 'rule-stratum\|stratify-fix\|ast-name'` | 0 |
| the gate is cited by both files | `grep -rc 'stratify_numbers' src/rete/kernel/stratify.rs wat/rete/oracle/stratify.wat` | ≥1 each |
| the gate still passes | `cargo nextest run --release -E 'test(native_stratify_numbers)'` | 1 passed |
| the grid axis still passes | `cargo nextest run --release -E 'test(wat_scripts_grid_port_check)'` | 1 passed |
| floor | `scripts/floor.sh` | 5473, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

20–35 min, nearly all of it the `wat/` rebuild and the floor.

## Trap doors

- **A corrected header that is also wrong.** This is the whole risk. The replacement text names
  `:221-227`, `stratify.wat:233-234`, `stratify.rs:178-179` and the gate by path — check each
  citation resolves before landing. `rete_citation_resolves` already caught one bad citation in
  this strike family; expect it to catch another if a path is off.
- **Reflex-correcting Edit 3.** It is a POINTER. The sentence it follows is TRUE. An executor
  pattern-matching "lockstep ⇒ false" will rewrite a correct claim into an incorrect one.
- **`wat/` is `include_str!`'d.** A comment-only `.wat` edit still changes the binary; do not skip
  the rebuild and do not measure anything against a stale one.
- **Scope.** `produced_type` is rowed, not fixed. An executor who notices it diverges and "just
  fixes the comment" ships an unproven claim — the exact thing being cleaned up.
