# BRIEF — Arc 233 Stone 233.2.b — tag at keyword/from-string producer (minimum-viable proof)

## What we're doing

`eval_keyword_from_string` wraps its return value in `Value::Tracked` with `Provenance::RuntimeBuilt { producer: ":wat::core::keyword/from-string", call_span }`. Probe 6 (committed at `b866305` as design substrate) demonstrates that the resulting `NotCallable` error message now includes producer info — closing the load-bearing runtime-built case from INVENTORY § O three-case table.

Also extend `ValueSnapshot::Display` to render `Provenance::RuntimeBuilt` inline (currently Display just shows `{type_name} `{rendered}``). Format: `{type_name} `{rendered}` (built by {producer} at {file}:{line}:{col})`.

This is the MINIMUM-VIABLE PROOF that producer tagging works end-to-end. 233.2.c sweeps additional producers; 233.2.d adds AST-derived provenance for let-bindings + literals.

## Design substrate (READ FIRST; MANDATORY)

1. **`tests/probe_diagnostic_value_snapshot_in_errors.rs` Probe 6** (commit `b866305`) — the failing probe. Currently FAILS because eval_keyword_from_string doesn't tag. After Stone 233.2.b ships, PASSES.

2. **`src/runtime.rs:7240`** — `fn eval_keyword_from_string(args, list_span: &Span, env, sym) -> Result<Value, RuntimeError>` — the sweep target. Takes `list_span` (the call span). Returns `Value::wat__core__keyword(Arc<String>)` (a bare keyword). Sonnet wraps the return in Tracked.

3. **`src/runtime.rs:1634`** — Provenance enum from 233.2.a. The `RuntimeBuilt { producer: &'static str, call_span: Span }` variant is the one we use.

4. **`src/runtime.rs` Value::Tracked** (added in 233.2.a) — `Tracked { inner: Box<Value>, provenance: Provenance }`. Sonnet constructs at the return site.

5. **`src/runtime.rs:1749`** — `impl std::fmt::Display for ValueSnapshot`. Currently writes `{type_name} `{rendered}``. Extend to render Provenance when not Unknown.

6. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.a.md`** — the precedent; Provenance + Tracked scaffolding shipped at 7cfeff1.

## Implementation surface

### Step 1 — Wrap eval_keyword_from_string return value

`src/runtime.rs:7240-7280` (end of `eval_keyword_from_string`). After constructing the bare `Value::wat__core__keyword(...)`, wrap it:

```rust
// Existing (sketched):
let kw = Value::wat__core__keyword(Arc::new(format!(":{}", s)));
Ok(kw)

// After 233.2.b:
let kw = Value::wat__core__keyword(Arc::new(format!(":{}", s)));
Ok(Value::Tracked {
    inner: Box::new(kw),
    provenance: Provenance::RuntimeBuilt {
        producer: ":wat::core::keyword/from-string",
        call_span: list_span.clone(),
    },
})
```

Sonnet picks the exact placement; the principle is: the Value that escapes eval_keyword_from_string carries provenance.

### Step 2 — Extend ValueSnapshot::Display to render Provenance

`src/runtime.rs:1749`. Current shape:

```rust
impl std::fmt::Display for ValueSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} `{}`", self.type_name, self.rendered)
    }
}
```

Extend to render Provenance when it's NOT Unknown:

```rust
impl std::fmt::Display for ValueSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} `{}`", self.type_name, self.rendered)?;
        match &self.provenance {
            Provenance::Unknown => Ok(()),
            Provenance::RuntimeBuilt { producer, call_span } => {
                write!(
                    f,
                    " (built by {} at {}:{}:{})",
                    producer, call_span.file, call_span.line, call_span.col
                )
            }
            Provenance::Literal { span } => {
                write!(f, " (from {}:{}:{})", span.file, span.line, span.col)
            }
            Provenance::SymbolBound { binding_span, head_span } => {
                write!(
                    f,
                    " (bound from {}:{}:{} at {}:{}:{})",
                    binding_span.file, binding_span.line, binding_span.col,
                    head_span.file, head_span.line, head_span.col
                )
            }
        }
    }
}
```

Display covers all 4 Provenance variants even though only RuntimeBuilt is populated in 233.2.b — keeps the impl symmetric for 233.2.c/d.

## Out of scope (affirmative scope-bounding)

- Other producers (from-holon, EDN-read, mailbox-recv, etc.) — Stone 233.2.c
- AST-derived provenance for let-bindings + literals — Stone 233.2.d
- Errors-as-EDN extension — Stone 233.3
- Cross-boundary provenance transport — future arc
- Performance tuning — v2
- holon-rs — NOT touched
- wat-edn — NOT touched

## Verification flow

```
cargo build --release -p wat                          # 0 errors
cargo test --release --lib -p wat --no-fail-fast      # baseline maintained ≥ 827
cargo test --release --test probe_value_tracked_transparency
                                                       # 8/8 transparency contracts still PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors
                                                       # 6/6 PASS — Probe 6 now passes
cargo clippy --release --lib -p wat -- -D warnings    # 52 warns (baseline match)
git -C /home/watmin/work/holon/holon-rs/ status --short # empty
```

## STOP triggers (REJECTION criteria — never permission-to-defer)

- **STOP-1:** unexpected compile errors
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** 60 min elapsed (small scope; should ship fast)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning
- **STOP-6:** scope creep — tagging OTHER producers (that's 233.2.c) or AST-derived provenance (233.2.d) or anything else
- **STOP-7:** Probe 6 still FAILS post-stone (the load-bearing flip)
- **STOP-8:** Stone 233.1 probes or Stone 233.2.a transparency tests regress
- **STOP-9:** Display impl breaks existing format expectations (existing assertions on error messages)

If any STOP fires: ship NOTHING beyond the clean-stoppable state; surface as honest delta in SCORE.

## Trap-door audit

Per the discipline + the pre-spawn audit pattern from 233.2.a:

- **NO invented syntax** — `Provenance::RuntimeBuilt { producer, call_span }` is the variant defined in 233.2.a; sonnet matches its shape exactly
- **NO made-up types** — Span exists; Provenance exists; Value::Tracked exists; ValueSnapshot exists
- **NO phantom dependencies** — eval_keyword_from_string has `list_span: &Span` in scope (verified at runtime.rs:7242)
- **Display format symmetry** — extend Display for ALL 4 Provenance variants in this stone, even though only RuntimeBuilt is populated. Keeps 233.2.c/d's work focused on PRODUCERS, not Display surgery

### Specific trap from pre-spawn audit (2026-05-23 night)

**Existing tests asserting RuntimeError message format may break.**

The change to Display adds parenthetical suffixes for Provenance::RuntimeBuilt — any existing test that pattern-matches on error message strings + uses CONTAINS-checks for the type_name + rendered will still pass (the prefix is unchanged). Any test using EXACT-match (`assert_eq!(error_msg, "...")`) on a RuntimeError-displayed message that came from a runtime-built keyword would break.

Sonnet's verification: run full lib + integration tests; if any test breaks, audit whether it's pattern-matching error format. If YES — update the test assertion to use CONTAINS instead of exact-match (small fix; documents the new format). If NO — investigate; may be a real regression.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly)
- HARD CUT — no aliases. No "if you can't render Provenance, fall back to old format" backward compat
- Per `feedback_inscription_immutable`: SCORE is a new file
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.md` — sub-DESIGN; Shape C
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.a.md` — Provenance + Tracked scaffolding precedent
- `tests/probe_diagnostic_value_snapshot_in_errors.rs` Probe 6 — design substrate (commit `b866305`)
- `src/runtime.rs:7240` — eval_keyword_from_string (sweep target)
- `src/runtime.rs:1749` — ValueSnapshot::Display (extend for Provenance rendering)
- `src/runtime.rs:1634` — Provenance enum
- `feedback_sonnet_writes_substrate` — protocol; sonnet writes substrate
