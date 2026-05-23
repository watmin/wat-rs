# BRIEF — Arc 233 Stone 233.2.c — sweep additional producers (from-holon, EDN-read, recv, try-recv)

## What we're doing

Replicate the 233.2.b pattern across 4 additional producer eval functions. Each wraps its return value in `Value::Tracked` with `Provenance::RuntimeBuilt { producer: "<verb-name>", call_span: list_span.clone() }`.

After this stone: every wat-callable producer that introduces a runtime-derived Value tags it with its origin. Errors carrying those Values surface the producer name in the diagnostic.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.b.md`** — the precedent stone (~5.8 min). This stone REPLICATES that pattern across 4 more sites.

2. **`tests/probe_diagnostic_value_snapshot_in_errors.rs`** Probes 7+8 (commit `b747ba3`) — failing probes for from-holon + edn::read. They flip FAIL → PASS post-stone. recv + try-recv covered via grep verification (wat-level probe would require channel setup; scope-out of v1).

3. **`src/runtime.rs:1634`** — Provenance enum from 233.2.a. `RuntimeBuilt { producer: &'static str, call_span: Span }` is the variant.

4. **`src/runtime.rs:1749`** — `impl std::fmt::Display for ValueSnapshot` (extended in 233.2.b). Already renders RuntimeBuilt. No further Display work needed in this stone.

## Producers to tag (with grep-verified locations)

### Producer 1 — `:wat::holon::from-holon`

- **Dispatch arm:** `src/runtime.rs:4734` — `":wat::holon::from-holon" => eval_holon_from_holon(args, list_span, env, sym)`
- **Eval fn:** `src/runtime.rs:14229+` — `eval_holon_from_holon`; has `list_span: &Span` in scope
- **Producer string:** `":wat::holon::from-holon"`
- **Wrap site:** at every Ok return (the fn has multiple return paths via type-dispatch; wrap each)

### Producer 2 — `:wat::edn::read`

- **Dispatch arm:** `src/runtime.rs:5205` — `":wat::edn::read" => crate::edn_shim::eval_edn_read(args, env, sym)`
- **Eval fn:** `src/edn_shim.rs:191+` — `eval_edn_read`; OP constant already there
- **Producer string:** `":wat::edn::read"`
- **Wrap site:** at the Ok return; needs `list_span` plumbed in (currently dispatch arm doesn't pass it). Sonnet adjusts the dispatch arm signature OR uses an alternative span source (e.g., Span::unknown() with a note in the SCORE if the dispatch doesn't carry one cleanly)

### Producer 3 — `:wat::kernel::recv`

- **Dispatch arm:** `src/runtime.rs:5283` — `":wat::kernel::recv" => eval_kernel_recv(args, env, sym, list_span)`
- **Eval fn:** `src/runtime.rs:19543+` — `eval_kernel_recv`; has `list_span` in scope
- **Producer string:** `":wat::kernel::recv"`
- **Wrap site:** at the Ok return (after channel recv succeeds)

### Producer 4 — `:wat::kernel::try-recv`

- **Dispatch arm:** `src/runtime.rs:5284` — `":wat::kernel::try-recv" => eval_kernel_try_recv(args, env, sym, list_span)`
- **Eval fn:** `src/runtime.rs:19611+` — `eval_kernel_try_recv`; has `list_span` in scope
- **Producer string:** `":wat::kernel::try-recv"`
- **Wrap site:** at the Ok return (after try-recv succeeds; the Some(value) case in the Option<Value> result)

### Out of v1 scope (deferred to follow-up if value-add justifies)

- `:wat::kernel::select` — produces a Value but the "producer" is one of N candidate channels; trickier shape (which channel produced it? requires additional bookkeeping). Defer.
- `:wat::io::IOReader/read*` — produces String from external IO. Bytes/io sources are typically the read FN itself but the actual bytes provenance is harder. Defer to v2.
- `:wat::core::keyword/to-string` — produces String from keyword. Less interesting (the keyword's content IS the resulting string). Defer.

## Implementation surface

For each producer (template; sonnet replicates 4×):

```rust
// BEFORE (sketch):
fn eval_<producer>(args, ..., list_span: &Span, ...) -> Result<Value, RuntimeError> {
    // ... existing logic ...
    Ok(result_value)
}

// AFTER:
fn eval_<producer>(args, ..., list_span: &Span, ...) -> Result<Value, RuntimeError> {
    // ... existing logic ...
    Ok(Value::Tracked {
        inner: Box::new(result_value),
        provenance: Provenance::RuntimeBuilt {
            producer: ":wat::<canonical-verb-name>",
            call_span: list_span.clone(),
        },
    })
}
```

### Special case: `:wat::edn::read`

The dispatch arm at runtime.rs:5205 doesn't currently pass `list_span` to `eval_edn_read`. Sonnet picks ONE of:

- **Option A:** modify the dispatch arm + eval_edn_read signature to thread `list_span` through. Cleanest; matches the other 3 producers.
- **Option B:** use `Span::unknown()` and document as honest delta. Less ideal; provenance loses its span coords.

Recommend Option A. Mechanical change; affects only the dispatch arm signature + eval_edn_read.

## Out of scope (affirmative scope-bounding)

- AST-derived provenance for let-bindings + literals (Stone 233.2.d)
- Errors-as-EDN extension (Stone 233.3)
- select, IO readers, keyword/to-string (deferred per "Out of v1 scope" above)
- Cross-boundary provenance transport
- Performance tuning
- holon-rs — NOT touched
- wat-edn — wat-edn ITSELF not touched; only the wat-rs side eval_edn_read

## Verification flow

```
cargo build --release -p wat                          # 0 errors
cargo test --release --lib -p wat --no-fail-fast      # baseline ≥ 827
cargo test --release --test probe_value_tracked_transparency  # 8/8 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors
                                                       # 8/8 PASS — Probes 7+8 flip
cargo clippy --release --lib -p wat -- -D warnings    # 52 warns (baseline)
git -C /home/watmin/work/holon/holon-rs/ status --short # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** 90 min elapsed (medium scope; 4 producers)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning
- **STOP-6:** scope creep — select / IO readers / non-listed producers
- **STOP-7:** Probes 7 or 8 still FAIL post-stone
- **STOP-8:** Stone 233.1 probes (1-6) regress (the floor must hold)
- **STOP-9:** Stone 233.2.a transparency tests regress

If any STOP fires: ship NOTHING beyond the clean-stoppable state; surface as honest delta in SCORE.

## Trap-door audit

Per arc 232.0/233.2.a/233.2.b's lessons:

- **NO invented producer names** — use the EXACT dispatch-arm string (e.g., `":wat::holon::from-holon"`, not `"from-holon"` or `"holon/from-holon"`)
- **NO list_span shortcuts** — if a producer's signature doesn't already carry list_span, plumb it (Option A for edn::read); don't fall back to Span::unknown() unless documented honest delta
- **Existing tests asserting RuntimeError message format** may need CONTAINS-not-EXACT updates (in scope; same as 233.2.b)
- **Multiple return paths in from-holon** — sonnet wraps at EACH Ok-path; don't miss one

### Specific trap from pre-spawn audit (2026-05-23 night)

**eval_holon_from_holon has multiple Ok-return paths** because the fn dispatches on the holon's variant. Each path must wrap its return. Sonnet's verification: grep the fn body for `Ok(` after edits and confirm each occurrence wraps in Tracked. The honest-delta SCORE row notes count of Ok-paths confirmed wrapped.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly)
- HARD CUT — no aliases
- Per `feedback_inscription_immutable`: SCORE is a new file
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.b.md` — the pattern this stone replicates
- `tests/probe_diagnostic_value_snapshot_in_errors.rs` Probes 7+8 — design substrate (commit `b747ba3`)
- Dispatch arms at `src/runtime.rs:4734, 5205, 5283, 5284` — verified
- Eval fns at `src/runtime.rs:14229, src/edn_shim.rs:191, src/runtime.rs:19543, 19611` — verified
- `feedback_sonnet_writes_substrate` — protocol
