# BRIEF — first delta is the facts vector; `seen` is filled once

## The work

SETUP clones 40,200 facts into a `Vec` and again into `seen`.
The first worklist is `wm.facts`. Fill `seen` once. Extract
the alpha step-1 body so both worklists call it.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — 2b landed; this is SETUP.
2. `DESIGN-STONE-setup-seen-once.md`.
3. `kernel.rs` ~3731–3745 (SETUP), ~3807–3867 (alpha loop).

## Sketch

```rust
fn alpha_activate_fact(fact: &Value, ...) -> Result<(), EvalBreak> { /* step 1 body */ }

let input = match &wm.facts {
    Value::wat__core__PersistentVector(pv) => pv.clone(),
    _ => rpds::VectorSync::new_sync(),
};
let mut seen = HashSet::with_capacity(input.len());
for f in input.iter() { seen.insert(f.clone()); }

// round 1
for fact in input.iter() { alpha_activate_fact(fact, ...)?; }
// later
for fact in &owned_delta { alpha_activate_fact(fact, ...)?; }
```

## STOP

1. **STOP-1** — inputs not in `seen` before derived production.
2. **STOP-2** — rete differential red.
3. **STOP-3** — a hasher crate in this diff.

## Done

- One clone+hash of inputs. First alpha pass walks the PV.
- Census still green on fold/snapshot. rete + clippy.

Leave dirty.
