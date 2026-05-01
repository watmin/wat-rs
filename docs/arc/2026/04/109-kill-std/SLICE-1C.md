# Arc 109 Slice 1c — Retire bare primitive types in user code

**Compaction-amnesia anchor.** Read this first if you're picking up
slice 1c mid-flight. The state below is durable; the conversation
context is not.

## What this slice does

Retire bare primitive types (`:i64`, `:f64`, `:bool`, `:String`,
`:u8`) in favor of FQDN forms (`:wat::core::i64`, etc.) at every
type-position site in user code. Per arc 109 § A
(`INVENTORY.md`), substrate-provided primitives live under
`:wat::core::*`; the bare spelling retires.

## The protocol

Pattern 3 from `docs/SUBSTRATE-AS-TEACHER.md` — dedicated
`CheckError` variant + walker. The substrate's diagnostic stream
IS the migration brief. Sweep tools consume the diagnostic stream
directly; no grep-with-context, no sed-with-guesses.

**Data-driven through and through:**

1. The walker parses each keyword as a `TypeExpr` *without
   canonicalization* (`parse_type_expr_audit`), then walks the
   structure (Path / Parametric / Fn / Tuple) and flags `Path` nodes
   matching the retired primitives. No textual scanning. Source
   spelling preserved in TypeExpr; FQDN distinct from bare by
   string identity.
2. The sweep consumes structured diagnostics
   (`wat --check --check-output edn|json`) — each
   `BareLegacyPrimitive` record carries `{primitive, fqdn,
   location}`. Sweep tooling rewrites at exact (file, line, col)
   byte positions. The substrate teaches; the sweep listens.

## What's done (commit `f2b5dd4` — 2026-05-01)

### `src/check.rs`

- `CheckError::BareLegacyPrimitive { primitive: String, fqdn:
  String, span: Span }` variant.
- `Display` impl: self-describing migration brief naming the rule,
  the canonical FQDN, the offending site.
- `diagnostic()` arm producing structured `BareLegacyPrimitive`
  records consumable via `--check-output edn|json`.
- `validate_bare_legacy_primitives` walker in `check_program`,
  invoked alongside `validate_scope_deadlock`.
- `walk_for_bare_primitives` (recursive AST walk).
- `walk_type_for_bare` (recursive TypeExpr walk).
- `BARE_PRIMITIVES` const: the five retired primitives + their
  FQDN replacements.

### `src/types.rs`

- `parse_type_expr_audit(kw) -> Option<TypeExpr>` — public
  non-canonicalizing parse for the audit walker.
- `parse_type_inner` / `parse_tuple_body` / `parse_fn_body` /
  `parse_type_list` gained a `canonicalize: bool` parameter
  threaded through.
- `parse_type_expr` (the type-checker entry) calls with
  `canonicalize=true` (preserves existing internal-form
  invariants); `parse_type_expr_audit` calls with `false`.

### Verified

- Bare-detection probe: `:i64` outer, `:Vec<i64>` inner,
  `:HashMap<String,i64>` (two-bare) — all fire.
- FQDN silence probe: `:wat::core::i64`, `:Vec<wat::core::i64>` —
  silent.
- False-positive resistance: `:my::pkg::String` (user path with
  `String` suffix) — silent. The walker uses Path string identity,
  not substring containment.

## What's left

### A. Substrate stdlib sweep (DO FIRST per SUBSTRATE-AS-TEACHER)

The substrate's bundled wat (loaded at every wat invocation) has
~57 remaining bare primitives. Until clean, every `wat` invocation
trips errors at startup before user code runs.

Files (from `grep -rE ':?(i64|f64|bool|String|u8|usize)\b' wat/
crates/*/wat/` minus `wat::core::` minus comment lines):

- `wat/std/hermetic.wat`
- `wat/std/sandbox.wat`
- `wat/std/stream.wat`
- `wat/holon/Circular.wat` (and other holon/*.wat)
- `wat/services/*.wat`
- `crates/wat-lru/wat/lru/*.wat`
- `crates/wat-holon-lru/wat/holon/lru/*.wat`
- `crates/wat-telemetry/wat/telemetry/*.wat`
- `crates/wat-telemetry-sqlite/wat/telemetry/*.wat`

### B. Lib test embedded wat strings

`<test>:N:M` source-name in the diagnostic stream means the bare
primitive lives inside an embedded wat string in:

- `src/check.rs::tests` (the test fns construct synthetic programs)
- `src/freeze.rs::tests`
- Other `tests/wat_*.rs` files with embedded program strings

These trip during `cargo test --release --lib -p wat`.

### C. User code sweep

35 files identified earlier:

- `wat-tests/**/*.wat` (~10 files)
- `examples/**/*.wat` (~5 files)
- `crates/*/wat-tests/**/*.wat` (~15 files)
- `crates/*/examples/**/*.wat` (~5 files)

Full list in scratch from prior session — re-derive via:
```bash
grep -rEln ':?(i64|f64|bool|String|u8|usize)\b' \
  wat-tests/ examples/ crates/*/wat-tests/ crates/*/examples/ \
  | xargs grep -lE ':(i64|f64|bool|String|u8|usize)\b|[<,(](i64|f64|bool|String|u8|usize)\b'
```

## How to drive the sweep — delegate to sonnet

The established pattern across arcs 110/111/112/113/114: **the
substrate's diagnostic stream is the brief; sonnet reads it and
applies the renames.** No custom Rust tooling, no sed-with-grep.

### The brief shape

```
"Run `cargo build --release` (or `./target/release/wat --check
<file>`); read the BareLegacyPrimitive errors. Each names a
file:line:col + the bare→FQDN rename. Apply per-site. Iterate
until green. Substrate stdlib first (it loads at every wat
invocation; until clean, every probe trips errors at startup).
Then lib test embedded wat strings. Then user wat-tests/ and
examples/."
```

That's the entire delegation. The diagnostic IS the work order.

### Confirming structured emission (optional pre-flight)

```bash
./target/release/wat --check --check-output edn <file>
```

Each `BareLegacyPrimitive` emits one EDN record with
`{primitive, fqdn, location}`. The text-mode form (default
`cargo build`/`wat --check` output) is what sonnet reads in
practice — the structured form is the data-faithful equivalent
when an agent or CI consumer wants to parse rather than read.

### Order

1. **Substrate stdlib first** — substrate boots clean before
   anything downstream makes sense.
2. **Lib test embedded strings** (`<test>:N:M` source-name in
   diagnostics).
3. **wat-tests/** + **crates/\*/wat-tests/**.
4. **examples/** + **crates/\*/examples/**.

After each tier, run `cargo test --release --workspace`; the
`grep -c "BareLegacyPrimitive"` count is the progress meter
(SUBSTRATE-AS-TEACHER § "Substrate is the progress meter").

## Order of operations

1. **Substrate stdlib** (`wat/`, `crates/*/wat/`) — sweep first;
   binary must boot clean before anything else makes sense.
2. **Lib test embedded strings** (`src/*.rs::tests`) — surface via
   `cargo test --release --lib -p wat`.
3. **`wat-tests/`** — top-level test files.
4. **`crates/*/wat-tests/`** — per-crate test files.
5. **`examples/`** + **`crates/*/examples/`** — example programs.
6. Once `cargo test --release --workspace` is green with zero
   `BareLegacyPrimitive` errors, slice 1c is complete.

## What does NOT belong in slice 1c

- **`:()` (the unit type annotation)** — that's slice 1d, separate
  arc 109 work. The walker does not flag `:()` (it's a Tuple
  literal in TypeExpr, not a Path).
- **Substrate Rust internal `Path(":i64")` literals** — these are
  the canonical-form internal representations from the
  type-checker. They stay; only the source-form bare primitives in
  `.wat` files retire. The canonical internal form may flip in a
  future slice, but that's a separate change.
- **The `parse_type_expr` (canonicalizing) call sites** — they
  preserve the canonicalize=true invariant. Don't migrate them to
  audit.

## Closure (slice 1c step 5)

When the sweep is structurally complete:

1. Update `INVENTORY.md` § A — strike the "today" column for the
   five primitives.
2. Update `J-PIPELINE.md` — mark slice 1c done.
3. Add a row to `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
   summarizing the rule + what swept.
4. Optional: retire the bare form at the parser level
   (parse_type_expr returns `TypeError::BareLegacyPrimitive`) so
   future user code can't reintroduce. The walker stays as the
   structural rule; the parser-level rejection is belt-and-
   suspenders.
5. Optional: write `INSCRIPTION.md` for slice 1c separately or
   bundle into the eventual arc 109 closure inscription.

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` § "Three migration patterns" —
  Pattern 3 is the discipline for slice 1c.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § A — the five
  primitives' before/after table.
- `docs/arc/2026/04/109-kill-std/J-PIPELINE.md` — landing order;
  slice 1c is independent of § J substrate work.
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md`,
  `docs/arc/2026/04/115-no-inner-colon-in-parametric-args/INSCRIPTION.md`,
  `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — Pattern 3 precedents.
- `src/check.rs::validate_bare_legacy_primitives` — the walker
  entry point.
- `src/types.rs::parse_type_expr_audit` — the data-driven parse
  the walker consumes.
