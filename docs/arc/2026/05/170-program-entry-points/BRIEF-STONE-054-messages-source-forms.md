# BRIEF — retain `:messages` records' source forms (finish FONTEM SERVO); delete the bracket dedup hack

**Root (grounded, this session):** a `defsurface :nature :Peer'` registers its `:messages` records
(`src/types.rs:1913-1919`), but `env.source_forms.insert(...)` at `:1940` stores **only the surface's**
source form — never the member records'. So when a `:messages` record is *also* pulled standalone (a
work-fn's `Address'<Echo::Op,Echo::Reply>` drags `EchoRequest`/`EchoResponse` into the topo-sort),
`closure_extract`'s ship loop (`src/closure_extract.rs:367-370`) finds **no source form** for it and ships
it via `type_def_to_ast` **reconstruction** — which **drifts** from the surface's embedded original (the
FONTEM-SERVO reconstruction-drift). The surface ships its *retained* form (which contains the record); the
standalone ships the *reconstructed* form; the two are **not byte-equivalent**, so arc-054 idempotency
(`src/types.rs:534`, `existing == def → no-op`) can't collapse them → `DuplicateDefine` in the child
(`src/runtime.rs:1146`).

**Proven:** the *byte-equivalent* case already works — `scratchpad/probe-054-fn-idempotency.wat` (a record
declared standalone AND in a surface's `:messages`, identical text) → `"ok"`. So making the two shipped
forms identical (retained) makes arc-054 no-op them, by construction.

## The fix (two edits)

1. **`src/types.rs`** — in the `:messages` extraction loop (`:1913-1919`), for each **user** message record
   (not reserved-prefix), **retain its source form**, mirroring the surface's retention at `:1940`. Sketch
   (clone the form before `parse_type_decl` consumes it):
   ```rust
   for msg_form in extract_surface_message_forms(sform) {
       if let Some(msg_head) = classify_type_decl(&msg_form) {
           let msg_span = msg_form.span().clone();
           let msg_form_clone = msg_form.clone();                 // ← retain
           let msg_def = parse_type_decl(msg_head, msg_form, msg_span, env)?;
           if !crate::resolve::is_reserved_prefix(msg_def.name()) {
               env.source_forms.insert(msg_def.name().to_string(), msg_form_clone);  // ← FONTEM SERVO, finished
           }
           d.push(msg_def);
       }
   }
   ```
   Now `closure_extract` ships the record via its **retained** form → byte-equivalent to the surface's
   embedded copy → arc-054 no-ops the double-registration. (Confirm the exact variable names / borrow shape
   against the real code; the checker teaches — one located error at a time.)

2. **`wat/bracket.wat`** — **DELETE** `dedup-surface-records` (the defn at `:181`) and its call site
   (`:303`, the `(:wat::bracket::dedup-surface-records forms)` wrapping) — restore the plain `forms`. The
   double-ship is now harmless (arc-054 collapses it), so the consumer-side dedup is dead.

## Blast radius
`src/types.rs` (the one `source_forms.insert` for `:messages` records) + `wat/bracket.wat` (remove the dedup
defn + call). No other src/. No behavior change to any consumer — only the *shipped form* of a `:messages`
record becomes retained-instead-of-reconstructed (identical semantics), and `defservice` is untouched
(it already ships surfaces; this only makes the member-record form consistent).

## STOP triggers
- **STOP-1** — if, after both edits, `probe-m1-pool-dial.wat` re-crashes with `DuplicateDefine`, the retained
  form is NOT byte-equivalent to the surface's embedded copy (or `type_def_to_ast` is reached via another
  path). STOP and report the two forms — do NOT re-add a dedup.
- **STOP-2** — if retaining `:messages` source forms breaks a `defservice` cross-fork test, STOP and report
  (it shouldn't — the form is identical semantics).

## Gate / Expectations (report each with its real result)
| what | command | expected |
|---|---|---|
| M1-pool dial, WITHOUT the bracket dedup | `./target/release/wat scratchpad/probe-m1-pool-dial.wat` (after `cargo build --release --bin wat`) | `["echo:a" "echo:b" "echo:c"]` (no `DuplicateDefine`) |
| byte-equiv still no-ops | `./target/release/wat scratchpad/probe-054-fn-idempotency.wat` | `"ok"` |
| bracket suite | `cargo nextest run --release -p wat --test kernel -E 'test(bracket)' --test-threads=1` | all pass |
| services suite (defservice cross-fork) | `cargo nextest run --release -p wat --test services --test-threads=1` | all pass (no regression) |
| whole floor | `cargo nextest run --release` | 0 NEW failures |

Return: the two probe outputs, the bracket + services suite results, the floor summary line, the
`types.rs` + `bracket.wat` diffs, and any STOP. Do NOT commit — the orchestrator weighs by its own re-run.
