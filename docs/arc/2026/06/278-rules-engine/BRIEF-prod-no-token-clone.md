# BRIEF — production walks `d_beta`, does not clone Tokens

## The work

Production clones 40k Tokens into a Vec it only reads. Walk
`d_beta[parent]` instead.

## Read in order

1. `DESIGN-STONE-join-extend-no-leftover.md` weigh.
2. `DESIGN-STONE-prod-no-token-clone.md`.
3. `kernel.rs` production pass (~4930–5018).

## Sketch

```rust
for pid in pids {
    let Some(ts) = d_beta.get(pid) else { continue };
    seen.reserve(ts.len().saturating_mul(forms.len()));
    for tok in ts {
        for compiled in forms {
            let derived = exec_compiled_rhs(compiled, &tok.bindings, sym)?;
            if seen.insert(derived.clone()) { ... }
        }
    }
}
```

## STOP

1. **STOP-1** — skip a parent (condition `:or`).
2. **STOP-2** — rete differential red.
3. **STOP-3** — intern class `String` in this diff.

## Done

- No Token clone-collect. Census printed. rete + clippy.

Leave dirty.
