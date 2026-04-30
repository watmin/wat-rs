# Arc 115 — INSCRIPTION

## Status

Shipped 2026-04-30. Two threads: data-first diagnostics
foundation + `:Vec<:String>` compile-time rejection. The arc's
slice 1 was completed mid-arc as the substrate work for arc 116;
slice 2 (the actual `:Vec<:String>` rejection) shipped on the
return pass; slice 3 (sweep) was empty because arc 112's earlier
sweeps already cleaned the live instances. Slice 4 is this
closure.

cargo test --release green throughout: 5 new arc-115 type-resolver
tests + 3 diagnostic-module tests carried over + 5 wat-cli
integration tests + 740 lib tests + 97 integration test result
rows; 0 failures.

Pushed:
- Slice 1 (data-first foundation): `2b397cc` (Diagnostic struct +
  --check + --check-output edn|json + 5 wat-cli tests + 3
  diagnostic tests) + `6c85ec7` (single `hint` field collapse)
- Slice 2 (the actual rejection): `83203bf` (`InnerColonInCompoundArg`
  variant + parse_type_inner detection + surface parse errors in
  let* bindings + 5 type tests)
- Slice 4 (closure): this INSCRIPTION + USER-GUIDE update + 058 row

## What this arc adds

Two interlocking foundations:

### 1. Data-first diagnostics (slice 1)

`src/diagnostic.rs` — the substrate's first formal Diagnostic
schema. Renderers are layered consumers; substrate is the source
of truth.

```rust
pub struct Diagnostic {
    pub kind: String,                                   // variant name
    pub fields: Vec<(String, DiagnosticValue)>,         // order-preserving
}
pub enum DiagnosticValue {
    String(String),
    Int(i64),
}
pub fn render_edn(diag: &Diagnostic) -> String;         // arc 092 v4 wire
pub fn render_json(diag: &Diagnostic) -> String;
```

Methods on the existing error types:

- `CheckError::diagnostic() -> Diagnostic` — every variant maps
  to a structured record mirroring its Rust struct fields.
- `CheckErrors::diagnostics() -> Vec<Diagnostic>` — one record per
  error.
- `StartupError::diagnostics() -> Vec<Diagnostic>` — Check arm
  flattens via the above; other variants yield one Diagnostic each.

Migration hints (arc_111_migration_hint, arc_112_migration_hint)
compose into a single `hint` field — collapsed from per-arc field
naming after user direction:

> i think we should just have 'hint' and we can use it whenever
> we need to?

The hint field is stable across the substrate's lifetime; helpers
retire silently when no consumer code emits the old shape.

### 2. CLI: `wat --check` + `--check-output edn|json`

```bash
wat --check file.wat                          # exit 0/1; freeze-only
wat --check --check-output edn  file.wat      # one EDN record per error
wat --check --check-output json file.wat      # one JSON record per error
```

Cargo-check ergonomics for wat — verify a `.wat` source freezes
cleanly without invoking `:user::main`. Editor save hooks, sweep
cycles, agent iteration loops use this without program side
effects.

### 3. The `:Vec<:String>` compile-time rejection (slice 2)

A new `TypeError::InnerColonInCompoundArg` variant. Detection at
the top of `parse_type_inner`: the OUTERMOST `parse_type_expr`
strips the legitimate leading `:`; any leading `:` that survives
into `parse_type_inner` means we're inside a compound (`<>`,
`()`, fn args, fn ret), where the colon prefix is illegal.

Self-describing error:

```
malformed :wat::core::let* form: binding 'xs': type expression
:Vec<:String> contains an illegal leading ':' on the inner argument
:String: inside `<>`, `()`, or `fn(...)`, type arguments are bare
Rust symbols. The colon prefix marks wat keywords and lives at
the OUTERMOST type position only. Drop the leading ':' on the
inner: write :Vec<String> instead.
```

Catches all five compound-position cases:

- `:Vec<:String>` (parametric arg)
- `:Result<:Option<i64>,...>` (nested parametric)
- `:fn(:i64)->bool` (fn arg)
- `:fn(i64)->:bool` (fn ret)
- `:Vec<:wat::core::String>` (FQDN inside compound)

### 4. Surface parse errors in let* bindings

`src/check.rs::process_let_binding` previously dropped
`TypeError` silently on parse failure (`Err(_) => return`).
Pre-arc-115 the substrate accepted malformed-but-recognizable
type strings and let users discover them via downstream
"expects X; got Y" mismatches. After arc 115, the parse error
pushes a `CheckError::MalformedForm` into the errors vec
preserving the parser's self-describing reason — surfaces
directly at the binding site.

## Why

User direction (2026-04-30):

> a common problem i see you making is having a quoted symbol
> within a quoted symbol... :Vec<:String> vs :Vec<String> the
> first is an illegal form we can catch at compile time?...
>
> in 115 - we need a new mode of wat... we need a "does this
> compile" - something who just loads the file and doesn't run
> :user::main
>
> the thing building the error context needs to compose it as
> data.. and then we can choose how to render this.. raw text,
> edn, json
>
> we are data first - always

Three threads, one arc:

1. The recurring inner-colon mistake the user kept catching in
   review now becomes a self-describing compile error.
2. The freeze-only mode (cargo-check ergonomics) means iteration
   loops don't run program side effects.
3. The substrate's diagnostic surface becomes data-first; text /
   EDN / JSON renderers all consume one source.

The third thread had the largest blast radius — arc 116 (the
phenomenal-cargo-debugging arc) immediately layered on top,
extending the data path into the test runner. Together arc 115
+ arc 116 give wat tooling consumers (CI, agents, editor LSPs)
structured access to every error class without parsing text.

## What this arc closes

- **The malformed-but-typechecks loophole.** Pre-arc-115,
  `:Vec<:String>` parsed into a slightly-different TypeExpr that
  caused cryptic downstream "expects :Vec<:String>; got
  :Vec<String>" mismatches. Post-arc-115, the parser rejects at
  the source with the rule named.
- **The let* binding silent-drop.** Pre-arc-115, the type-checker
  at `process_let_binding` line 3273 silently swallowed parse
  errors. Post-arc-115, every parser-level error reaches the
  user-facing diagnostic stream.
- **The flatten-to-string boundary.** Pre-arc-115, every
  diagnostic became a String at the point of consumption. Post-
  arc-115, the substrate exposes structured Diagnostics; renderers
  produce text / EDN / JSON from one source.

## Slice walkthrough

### Slice 1 — data-first foundation (`2b397cc`, refined `6c85ec7`)

`src/diagnostic.rs` minted the Diagnostic struct + render helpers.
`CheckError::diagnostic()`, `CheckErrors::diagnostics()`,
`StartupError::diagnostics()` produce structured records.
`crates/wat-cli/src/lib.rs` gains the `--check` and `--check-output`
flag handling. Five wat-cli integration tests + three
diagnostic-module unit tests.

The single-`hint`-field refinement collapsed `arc_111_hint` and
`arc_112_hint` field name proliferation: migration helpers
compose into one stable schema field, retire silently per the
substrate-as-teacher lifecycle.

### Slice 2 — `:Vec<:String>` rejection (`83203bf`)

`TypeError::InnerColonInCompoundArg` variant; detection at top
of `parse_type_inner`; let* binding parse errors surface via
`CheckError::MalformedForm`. Five type-resolver unit tests + six
canonical-legal-form regression tests.

### Slice 3 — sweep (empty)

The substrate-stdlib + lab + tests grep turned up zero live
instances of the malformed shape — arc 112's earlier sweeps
(slice 2a fixture sweep + slice 4 demo sweep) already cleaned
them. The only matches were in arc-115's own docstrings + tests
(deliberate: documenting the rule by example).

### Slice 4 — closure (this slice)

INSCRIPTION + USER-GUIDE update + 058 row.

## The four questions (final)

**Obvious?** Yes. The error names the rule, shows the offending
fragment, suggests the canonical form. New wat authors learn the
rule from compiler output; experienced ones get a safety net for
the recurring drift.

**Simple?** Yes. ~30 LOC for `InnerColonInCompoundArg` + detection
+ Display impl. ~20 LOC for the let* parse-error surfacing. The
data-first foundation (~250 LOC for diagnostic module + integration)
is more work but lands a substrate capability that downstream arcs
build on (arc 116 already; LSP / GitHub Actions consumers next).

**Honest?** Yes. The substrate already had the data; arc 115
stops flattening it. The malformed-form acceptance was a real
silent-drop bug; arc 115 closes it.

**Good UX?** Phenomenal in tooling integration:
`WAT_TEST_OUTPUT=json cargo test` (arc 116) | `jq` →
GitHub Actions annotations or LSP diagnostics in ~30 LOC
because the data is structured at the source. The `--check` flag
gives editor hooks a fast freeze-only verification path. The
`--check-output edn|json` mode means agent sweep loops never
parse text.

## Cross-references

- `docs/arc/2026/04/116-phenomenal-cargo-debugging/INSCRIPTION.md`
  — the immediate consumer of arc 115 slice 1's data-first
  foundation; `Failure::diagnostic()` + `WAT_TEST_OUTPUT`
  env var.
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` —
  the substrate-as-teacher pattern arc 115's error surface
  mirrors at the type-syntax level.
- `docs/arc/2026/04/112-inter-process-result-shape/INSCRIPTION.md`
  — arc 112's `arc_112_migration_hint` + `arc_111_migration_hint`
  composers feed the new `hint` field.
- `feedback_wat_colon_quote.md` (memory) — the user's
  long-standing rule the new error class enforces.

## Queued follow-ups

- **Hint retirement (task #168 + arc-112 follow-up)** — when
  consumer wat code stops emitting old-shape errors anywhere,
  the corresponding `arc_N_migration_hint` helper deletes from
  `src/check.rs`. The hint field stays; its content goes empty
  for that arc's class. Same retirement-redirect pattern arcs
  109 / 111 set up.
- **GitHub Actions annotations** — ~30-LOC consumer of
  `WAT_TEST_OUTPUT=json` translating each Diagnostic to a
  `::error file=...,line=...,col=...::message` line. Drops
  inline failure annotations on PRs.
- **Editor LSP integration** — ~25-LOC consumer of the same
  JSON stream mapping to `PublishDiagnosticsParams`. Editor
  shows red squigglies inline at the source location.
- **Diagnostic field schema growth** — slice 1 minted minimal
  fields (kind + named String/Int values). Future arcs may add
  Vec / nested Diagnostic for chained-cause backtraces (arc 113)
  or pattern-coverage failure (match exhaustiveness).
