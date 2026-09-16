# BRIEF — the fourth wall

Add a graph wall to `import_export` so a network the engine cannot legally walk is refused at the
door instead of returned as a runnable `Session`. The wall proves three things about the already-
unpacked node map: every child id names a node, every ref-alpha id names a node **that is an
`Alpha`**, and every child id **exceeds its parent's** (the ascending-id topological order the fire
passes require). It refuses with `malformed` — it never repairs. Read `DESIGN.md` beside this file
first; its ★ ONE CONTRACT DECISION is the line that governs every judgment call here.

## Read in order, and why

1. **`src/rete/export.rs:2010-2033`** — `import_export`'s own phase list. Your wall is a new phase
   between 3 and 5. Add it to this doc comment in the same edit.
2. **`src/rete/export.rs:60-75`** — the module header's *"three walls"*. It becomes four. The
   header and the code must agree; that they disagreed is how this defect stayed invisible.
3. **`src/rete/export.rs:2112-2128`** — phase 3. `network_pairs` is built here and consumed by
   `PMap::from_pairs` on the last line. **Your pass goes after `network` is bound and before the
   `compiled_conds` loop.** Note `max_id` is already tracked in this loop — you may not need it,
   but know it is there.
4. **`src/rete/export.rs:170-180`** — `malformed(span, op, reason)`, the refusal the other three
   walls use. Use it. `IMPORT_OP` is the op string already in scope.
5. **`src/rete/kernel/node.rs:86,119,130,193`** — `kind_of`, `node_ref_alpha_id`, `node_children`,
   `sorted_node_ids`. These are the accessors; do not re-derive their logic. `node.rs:193` is where
   the ascending-order requirement is stated.
6. **`src/rete/kernel/node.rs:37-47`** — `NodeKind`'s nine variants, so you can name `Alpha`.
7. **`tests/rete/probe_arc278_export.rs:135-175`** — `import_refuses_abi_mismatch`, the shape a
   refusal test takes in this file. Your probe copies it. `import_one`, `poke_named` and
   `seq_values` are its helpers, already present at `:271-310`.
8. **`strike-import-graph-wall/probe.rs.txt`** — the disconfirming probe, already written and
   already proven RED at HEAD `d024afb2e` (its captured output is in `DESIGN.md`). It needs one
   helper, `export_field`, which is included in the same file.

## Implementation sketch

```rust
// after:  let network = Value::wat__core__PersistentMap(PMap::from_pairs(network_pairs));
// before: let mut compiled_conds = HashMap::new();

check_node_graph(&network, span)?;
```

```rust
/// WALL 4 — the graph must be one the fire passes can legally walk.
fn check_node_graph(network: &Value, span: &Span) -> Result<(), EvalBreak> {
    let ids: HashSet<i64> = /* sorted_node_ids(network) */;
    for id in /* ascending ids */ {
        let node = /* get_node */;
        for kid in node_children(node) {
            if !ids.contains(&kid) { return Err(malformed(span, IMPORT_OP, format!(...))); }
            if kid <= id            { return Err(malformed(span, IMPORT_OP, format!(...))); }
        }
        if let Some(aid) = node_ref_alpha_id(node) {
            // must resolve AND be an Alpha
        }
    }
    Ok(())
}
```

Each refusal message names the parent id, the offending edge, and which of the three rules broke —
a reader who gets this error should not have to open the wire to know what was wrong with it.

## Blast radius

`src/rete/export.rs` and `tests/rete/probe_arc278_export.rs`. No new types, no pack-side change,
no wire-format change, no version bump.

## STOP triggers — halt and surface, do not improvise

1. **If any locally compiled network in the corpus violates `child > parent`, STOP.** Before
   enforcing that rule, prove it holds on real networks: run the existing export/import tests and
   the strat-neg and accum fixtures with the wall in place. A single legitimate violation means the
   rule is wrong, not the network — surface it and do not weaken the wall to fit. **This is the
   step to do FIRST, before writing the refusal.**
2. **If `node_ref_alpha_id` returns `Some` for a kind that is not Negation/Exists/Accumulate,
   STOP** — the accessor's kind mapping and this wall's assumption disagree, and that is a finding
   about `node.rs:119`, not something to code around.
3. **If the probe passes before you write the wall, STOP** — the fixture is not producing a
   dangling edge and the probe is measuring nothing. Re-derive the truncation.
4. **If a refusal message would name a node id the user never wrote, STOP.** The vigilia found
   exactly that shape elsewhere today (a diagnostic naming a fact type absent from the source);
   do not ship a second one.

## The verification order, which is the point of the strike

Apply the probe **first**, run it, **confirm RED and confirm it fails for the dangling edge** —
not for a truncation artefact. Only then write the wall. The probe going RED→GREEN across your
change is the mutation proof, obtained for free; a gate written after its fix has only ever been
green, which is the shape R59 names.

## A prior comparable result to copy for shape

`import_refuses_abi_mismatch` (`tests/rete/probe_arc278_export.rs:135`) — a tampered export, an
expected refusal, and an assertion on the refusal's *contract* rather than its wording. That test
is the model for both what you write and how tightly you assert it.
