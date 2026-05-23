# BRIEF — Arc 233 Stone 233.1 — ValueSnapshot sweep across RuntimeError

## What we're doing

Mint a `ValueSnapshot` type that carries `{ type_name, rendered, provenance }`. Sweep three `RuntimeError` variants — `NotCallable`, `TypeMismatch`, `BadCondition` — promoting their `&'static str` `got`/`expected` fields to `ValueSnapshot`. Render the actual Value at error-construction time.

v1 of `Provenance` is just `Unknown` — a placeholder enum variant. Stone 233.2 (separate; arc 233) fills in `Literal { span }` / `SymbolBound { binding_site, head_site }` / `RuntimeBuilt { producer, head_site }` later.

Result: error messages start carrying the OFFENDING VALUE'S RENDERED CONTENT alongside its type name. The `NotCallable { got: "wat::core::keyword" }` we hit in arc 232.0 becomes `NotCallable { got: ValueSnapshot { type_name: "wat::core::keyword", rendered: ":wat::core::i64::+'2", provenance: Unknown } }` and its Display output includes the keyword content.

## Design substrate (READ FIRST; MANDATORY)

1. **`tests/probe_diagnostic_value_snapshot_in_errors.rs`** (commit `[TBD-this-stone]`) — the probe file. 2 currently-FAILING probes covering `NotCallable` with literal-bound + runtime-built keyword heads. They become regression guards: after Stone 233.1 ships, they PASS. The probe's module-doc names which RuntimeError variants are in scope + the rendering approach.

2. **`src/runtime.rs:1628-1700`** — the `RuntimeError` enum definition. Sweep targets:
   ```rust
   NotCallable { got: &'static str, span: Span }
   TypeMismatch { op: String, expected: &'static str, got: &'static str, span: Span }
   BadCondition { got: &'static str, span: Span }
   ```
   These are the THREE variants in 233.1 scope. Other `RuntimeError` variants (`ArityMismatch`, `MalformedForm`, etc.) carry different field shapes; out of scope for 233.1.

3. **`src/runtime.rs:17382`** — `fn render_value(v: &Value, depth: usize) -> String` — the EXISTING rendering primitive. Handles all Value variants, has SHOW_MAX_DEPTH guard for recursion. Use as-is; don't reinvent.

4. **`src/runtime.rs:1882`** — the existing `RuntimeError::NotCallable` Display path (in `impl Display for RuntimeError`). Reference for how Display currently formats the type name; needs to be updated to include the rendered value.

5. **`docs/arc/2026/04/109-kill-std/INVENTORY.md` § O** — the backlog entry that drove this stone; particularly the "Span coverage vs runtime-derived gap" table showing why inline rendering matters for the runtime-built case.

## The type design

Mint at the top of `src/runtime.rs` (or a sibling module like `src/diagnostic.rs` if cleaner — sonnet picks the honest home):

```rust
/// Snapshot of a value attached to a runtime error for diagnostic richness.
///
/// Carries the value's type name (cheap; static) AND a rendered form
/// (heap-allocated; constructed at error-creation time via `render_value`).
///
/// `provenance` is `Unknown` in 233.1. Stone 233.2 fills it with real
/// variants (Literal / SymbolBound / RuntimeBuilt) once Value-level
/// provenance tracking lands.
#[derive(Debug, Clone)]
pub struct ValueSnapshot {
    pub type_name: &'static str,
    pub rendered: String,
    pub provenance: Provenance,
}

/// Provenance of a Value — where it came from.
///
/// Stone 233.1 ships only `Unknown`. Stone 233.2 adds variants:
/// `Literal { span }` — the value appeared as a literal in source
/// `SymbolBound { binding_site, head_site }` — bound via let; trace via span
/// `RuntimeBuilt { producer, head_site }` — built by `keyword/from-string`,
///   `from-holon`, mailbox payload, etc.
#[derive(Debug, Clone)]
pub enum Provenance {
    Unknown,
}

impl ValueSnapshot {
    /// Construct from a runtime Value at error-creation time. Uses
    /// existing `render_value` for the rendered field; `Provenance::Unknown`
    /// always in v1.
    pub fn of(v: &Value) -> Self {
        ValueSnapshot {
            type_name: v.type_name(),
            rendered: render_value(v, 0),
            provenance: Provenance::Unknown,
        }
    }
}

impl std::fmt::Display for ValueSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Type name followed by rendered content in backticks.
        // Example: "wat::core::keyword `:wat::core::i64::+'2`"
        write!(f, "{} `{}`", self.type_name, self.rendered)
    }
}
```

Sonnet picks the exact module placement + visibility. The shape above is the SUBSTRATE-CANONICAL shape; deviate only with explicit justification.

## The RuntimeError sweep

Change three variants:

```rust
// BEFORE:
NotCallable { got: &'static str, span: Span }
TypeMismatch { op: String, expected: &'static str, got: &'static str, span: Span }
BadCondition { got: &'static str, span: Span }

// AFTER:
NotCallable { got: ValueSnapshot, span: Span }
TypeMismatch { op: String, expected: &'static str, got: ValueSnapshot, span: Span }
BadCondition { got: ValueSnapshot, span: Span }
```

**Note on `TypeMismatch.expected`:** keep as `&'static str` — the EXPECTED side is a TYPE NAME (not a value); the actual-value snapshot lives in `got`. `expected` stays static. (If a future stone wants type expressions in `expected`, that's separate scope.)

### Update all construction sites

`grep -n "RuntimeError::NotCallable\|RuntimeError::TypeMismatch\|RuntimeError::BadCondition" src/runtime.rs` finds the construction sites. At each:

- Where `got` was a `&'static str` (the type-name string from `Value::type_name()`), thread the actual Value through and call `ValueSnapshot::of(&v)` to construct
- Where the Value isn't already in scope (rare — most error paths have the offending Value), audit + add it. If genuinely unavailable, honest-fall-back to a synthetic `ValueSnapshot { type_name: <static-str>, rendered: "<unavailable>".into(), provenance: Unknown }` per case
- Update Display impl for each variant — format the ValueSnapshot via its Display

### Update Display output format

The Display path is in `impl Display for RuntimeError` (around `src/runtime.rs:1882`). Currently:

```
not callable: got wat::core::keyword at <span>
```

Target:

```
not callable: got wat::core::keyword `:wat::core::i64::+'2` at <span>
```

The format above (type-name SPACE backtick-rendered-backtick) matches the `ValueSnapshot` Display sketch above. Sonnet may adjust spacing if needed for readability, but the rendered content MUST appear.

## Out of scope (affirmative scope-bounding)

- **Provenance variants beyond `Unknown`** — Stone 233.2 lands these
- **Errors-as-EDN extension** — Stone 233.3
- **Other RuntimeError variants** — `ArityMismatch`, `MalformedForm`, `EvalForbidsMutationForm`, etc. carry different field shapes. Out of 233.1 scope. If a future stone wants a uniform diagnostic-richness sweep across ALL variants, it's separate
- **`CheckError::TypeMismatch`** — `src/check.rs` has its own TypeMismatch enum variant with different shape (operates at type level, not value level). Out of 233.1 scope. Check-errors are a different territory; substrate-error renaming there is Arc 233's later concern (likely Stone 233.3 if at all)
- **Performance optimization** — `render_value` is called only at error-creation time (cold path). No "production mode" needed for v1
- **holon-rs** — NOT touched
- **wat-edn** — NOT touched

## Probe additions (in-scope for sonnet)

The orchestrator's probe file has 2 probes (both `NotCallable`). The runtime trigger shapes for `TypeMismatch` and `BadCondition` are non-trivial to construct from this orchestrator's vantage (check-pass catches most static cases). **Sonnet adds 2-3 more probes covering**:

- `RuntimeError::TypeMismatch` runtime trigger (likely via polymorphic dispatch, runtime-built values, or apply-spread mismatches)
- `RuntimeError::BadCondition` runtime trigger (if reachable; if all paths are caught at check time, document the gap as honest delta)

If `BadCondition` or `TypeMismatch` runtime triggers genuinely can't be constructed (every path caught at check time), report as honest delta in SCORE — the sweep still happens at the Rust enum level even if the wat-level probe coverage has gaps.

## Verification flow

```
cargo build --release -p wat                          # 0 errors
cargo test --release --lib -p wat --no-fail-fast      # baseline maintained
                                                       # (existing tests asserting error
                                                       # message contents may need
                                                       # updates — part of the sweep)
cargo test --release --test probe_diagnostic_value_snapshot_in_errors
                                                       # both existing probes PASS
                                                       # (was FAIL); new probes PASS
cargo clippy --release --lib -p wat -- -D warnings    # 52 warns (baseline match)
git -C /home/watmin/work/holon/holon-rs/ status --short # empty
```

## Trap-door audit (per the discipline inscribed during arc 232)

Per `feedback_sonnet_writes_substrate` + FM 2-bis + the trap-door lessons from 232.0:

- **NO invented syntax.** All wat snippets use canonical inline `-> :T` (cite `Result/expect`'s shape in BRIEF as the precedent if relevant)
- **NO made-up primitive names.** `render_value` exists at `src/runtime.rs:17382` (verified by grep). Don't invent new ones unless explicitly required
- **NO Phantom dependencies.** This stone references only existing types + `render_value`. ValueSnapshot is the only new type
- **NO wat-colon-quote violation** (per `feedback_wat_colon_quote`): if any probe or doc-example uses parametric types like `:Option<HolonAST>`, write `:wat::core::Option<wat::holon::HolonAST>` — NO inner colon inside `<>`
- **NO `[-> :T]` bracket-vector syntax anywhere** — canonical wat uses INLINE `-> :T` only

## STOP triggers (REJECTION criteria — never permission-to-defer)

- **STOP-1:** unexpected compile errors beyond the expected sweep ripple
- **STOP-2:** baseline test count regresses below 827 + the new probes
- **STOP-3:** 180 min elapsed (upper-bound; sweep across many sites)
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** new clippy warning beyond pre-existing 52
- **STOP-6:** scope creep — Provenance variants beyond `Unknown`, or any other RuntimeError variant beyond the three in scope, or any CheckError touch
- **STOP-7:** existing probes still FAIL (the load-bearing flip)
- **STOP-8:** Display output no longer contains the rendered value (the whole point)

If any STOP fires: ship NOTHING beyond the clean-stoppable state; surface as honest delta in SCORE.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly)
- HARD CUT — no aliases. No old-field-shape backward compat
- Per `feedback_inscription_immutable`: do NOT edit past SCORE / FINDING / INSCRIPTION docs; this is forward work in NEW files
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN.md` — the umbrella; this is Stone 233.1 of 4
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § O — the backlog entry that motivated this work
- `tests/probe_diagnostic_value_snapshot_in_errors.rs` — the design substrate (failing probes that flip to PASS)
- `src/runtime.rs:1628-1700` — RuntimeError enum (the sweep target)
- `src/runtime.rs:17382` — `render_value` (the rendering primitive)
- `src/runtime.rs:1882` — Display impl for RuntimeError (Display path to update)
- arc 064 — assert-eq renders values + surfaces location (precedent for value-render in diagnostics)
- arc 138 — errors carry point-in-code coordinates (precedent for substrate-wide error sweep)
- arc 211b — panic-as-EDN (parallel work in arc 233 Stone 233.3)
- `feedback_sonnet_writes_substrate` — protocol discipline; sonnet writes; orchestrator briefs + scores
- `feedback_wat_colon_quote` — type-syntax discipline (no inner colons inside `<>`)
