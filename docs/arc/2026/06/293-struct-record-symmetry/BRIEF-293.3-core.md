# BRIEF — arc 293.3-core: `defsurface` + structural satisfaction (the keystone)

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents. Do NOT use git worktrees. Do NOT commit.** If the work exceeds these rooms or hits a STOP trigger,
STOP and report — do not improvise a workaround.

Build/test: `cargo build --release -p wat`, `cargo test --release -p wat …`. Trust **forced clean builds**
(`cargo clean -p wat && cargo build --release -p wat`) if results look stale.

## The work, in one paragraph

Add **`defsurface`** — a declaration that names a **structural surface** (a set of required accessors,
`[name <- :type …]`) — and make a **struct structurally satisfy it** by *having* the fields (row-polymorphic
**width subtyping**: extra fields are fine; no `:satisfies`, no `:parent`, no declaration). Mechanically: a
surface is a new **`TypeDef::Surface`** (a sibling of `TypeDef::Struct`/`Record`/`Enum`), parsed via the
existing `parse_argspec_triples`; and the type-checker's **`assignable`** gains an arm — when the *expected*
type resolves to a `TypeDef::Surface`, the *actual* type (a struct) satisfies it iff its `StructDef.fields`
contains every surface member with a field-type assignable to the member's type. This is **pure-TypeEnv**
(a `StructDef.fields` is typed) — **no SymbolTable / CheckEnv threading.** The keystone proves the genuinely
new machinery (surface registration + structural matching) end to end.

**THE GATE = the committed RED probe goes GREEN:** `tests/probe_arc293_structural_surface.rs`
(`record_structurally_satisfies_a_defsurface` — name is historical; it uses a `defstruct` candidate). It is
`#[ignore]`'d; **remove the `#[ignore]` when it passes.**

## Decision pinned (do NOT re-litigate)

- A surface is a **`TypeDef::Surface`**, NOT a `TypeExpr` variant. `:geo::Shape` in type position is already a
  `TypeExpr::Path(":geo::Shape")`; the checker resolves the Path → its `TypeDef` (exactly how `Struct`/`Record`
  resolve). So there is **no parser-bracket change** and **no new `TypeExpr` variant** — a `Path` that resolves
  to `TypeDef::Surface` IS the surface in type position.
- **Scope = STRUCTS only.** `StructDef.fields: Vec<(String, TypeExpr)>` is typed, so the match is TypeEnv-only.
  A `Record::def` record has `RecordDef.field_types = None` (its field types live in accessors, not the
  TypeEnv) — **records are OUT OF SCOPE here** (they ride a later strike). If you find yourself needing the
  function table / accessor signatures, you have drifted into the record/method case — **STOP.**

## Rooms — read in order (exact file:line; re-ground before editing)

1. **`src/types/defstruct.rs`** (whole) — THE WORKED PATTERN to copy. `parse_defstruct` parses
   `(:wat::core::defstruct :Name [fields])` → `TypeDef::Struct(StructDef)` using the argspec/field parser.
   Your `parse_defsurface` mirrors it.
2. **`src/argspec/parse.rs`** — `parse_argspec_triples(args_vec, head, form_span, options)` → `ArgSpec` with
   `fixed_params: Vec<(Identifier, TypeExpr)>`. This is how you parse `[name <- :type …]`. Use
   `ParseOptions { allow_rest_binder: false }`. Map `fixed_params` → `members: Vec<(String, TypeExpr)>`
   (the `Identifier`'s string + the `TypeExpr`).
3. **`src/types.rs:39`** — `pub(crate) mod defstruct;` + the `pub(crate) use defstruct::parse_defstruct;`
   re-export. Add `pub(crate) mod surface;` + `pub(crate) use surface::parse_defsurface;` beside it.
4. **`src/types.rs:124–135`** (`StructDef`) — copy its shape for a new **`SurfaceDef { name: String,
   type_params: Vec<String>, members: Vec<(String, TypeExpr)> }`** (drop `restrictions`; keep `type_params`
   empty for v1 — monomorphic, like the probe).
5. **`src/types.rs:214–222`** — the `TypeDef` enum. Add **`Surface(SurfaceDef)`** beside `Struct`/`Record`/etc.
   Then the compiler will list every exhaustive `match TypeDef { … }` that now needs a `Surface` arm — fix
   each (name accessor `TypeDef::name()`, any `Display`/registration/reflection match). These are the
   **bounded cascade** — ride it to zero; most arms are trivial (`Surface(s) => &s.name`, or a no-op).
6. **`src/types.rs:1620–1672`** — `classify_type_decl` (head → kind keyword) + the parse dispatch. Add
   `":wat::core::defsurface" => return Some("defsurface")` (beside `:1625` defstruct) AND
   `"defsurface" => parse_defsurface(iter.collect(), decl_span)` (beside `:1662`).
7. **`src/freeze.rs:1533` + `:1569`** — two lists that recognize type-decl heads (each contains
   `| ":wat::core::defstruct"`). Add `| ":wat::core::defsurface"` to BOTH.
8. **`src/check.rs:14184`** — `assignable(actual, expected, subst, types)`. This is where the structural
   match lands (see sketch). It already has `types: &TypeEnv` — that is all you need.

## Implementation sketch (fill it; do not reinvent the shape)

```rust
// src/types/surface.rs — mirror defstruct.rs
pub struct SurfaceDef { pub name: String, pub type_params: Vec<String>, pub members: Vec<(String, TypeExpr)> }
pub fn parse_defsurface(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    // args = [:Name, [name <- :T  name <- :T …]]   (arity: name + one field-vector; mirror parse_defstruct's
    //   name + metadata? + fields shape — v1: name + fields, no metadata)
    // parse the name (parse_declared_name("defsurface", …)); extract the Vector items; call
    // parse_argspec_triples(items, ":wat::core::defsurface", span, ParseOptions{allow_rest_binder:false});
    // map ArgSpec.fixed_params (Identifier, TypeExpr) -> members (name.to_string(), ty). Empty members is legal.
    Ok(TypeDef::Surface(SurfaceDef { name, type_params: vec![], members }))
}

// src/check.rs — inside assignable, BEFORE the final `unify(...)` fallthrough:
//   resolve the EXPECTED type to a TypeDef::Surface; if so, structurally match the ACTUAL type's fields.
let e = reduce(&walk(expected, subst), subst, types);
if let TypeExpr::Path(ep) = &e {
    if let Some(crate::types::TypeDef::Surface(surf)) = types.get(ep) /* the TypeEnv's name->TypeDef lookup */ {
        let a = reduce(&walk(actual, subst), subst, types);
        if let TypeExpr::Path(ap) = &a {
            if let Some(crate::types::TypeDef::Struct(sd)) = types.get(ap) {
                // width subtyping: every surface member must be present in the struct with an assignable type.
                return surf.members.iter().all(|(mname, mty)| {
                    sd.fields.iter().any(|(fname, fty)| fname == mname && assignable(fty, mty, subst, types))
                });
            }
            // actual is not a struct (e.g. a record / scalar) → surfaces are struct-scope here; fall through
            // (do NOT special-case records; that is a later strike).
        }
    }
}
```
(Use the TypeEnv's real lookup method — grep `impl TypeEnv` for `fn get`/`fn lookup`/`fn iter`; do NOT invent
one. If there is no by-name `get`, a small `pub fn get(&self, name: &str) -> Option<&TypeDef>` over the same
map `iter()` walks is acceptable — keep it minimal.)

## STOP triggers (halt + report; do NOT improvise)

1. **STOP if the structural match needs the SymbolTable / accessor signatures / `CheckEnv`** — that means you
   drifted to the record or method case, which is out of scope. The struct case is pure `StructDef.fields`.
2. **STOP if the `TypeDef::Surface` cascade spreads beyond trivial arms** (name/Display/register) into real
   logic in many files — report the site list before mass-editing.
3. **STOP if `assignable` cannot resolve a `Path` to its `TypeDef`** with the `&TypeEnv` it already holds —
   report the exact lookup gap; do NOT thread new state through `assignable`'s signature without reporting.
4. **STOP if making the probe green requires touching `wat/Record.wat`, `Record::def`, records, or
   `register_*_methods`** — that is 293.2, not this strike.
5. You are a LEAF. Do NOT spawn subagents. If the change exceeds these rooms, STOP and report.

## Gate (the orchestrator re-runs every line against the disk)

| what | command | expected |
|---|---|---|
| the keystone probe goes green | `cargo test --release -p wat --test probe_arc293_structural_surface -- --ignored` | 1 passed (then REMOVE the `#[ignore]` and re-run without `--ignored` → 1 passed) |
| width subtyping holds (extra field OK) | the probe's Circle has an extra `radius` — already exercised | green |
| a MISSING member is REJECTED | add a 2nd test: a `defstruct :Bare [other <- :i64]` (no `color`) passed where `:geo::Shape` is wanted → must FAIL to type-check (assert `world.is_err()`) | the surface is a real lower bound, not a rubber stamp |
| no new workspace regressions | `cargo test -p wat --no-fail-fast`, failing-test SET vs HEAD (`058f6035`) | **∅** new (weigh by SET, never absolute count) |

Runtime: 45–90 min. Trap-doors: (a) the `TypeDef::Surface` enum-variant cascade (ride it to zero — the
fail-count is the progress meter); (b) the TypeEnv by-name lookup in `assignable` (use the existing one); (c)
the `parse_defsurface` arity/shape (copy `parse_defstruct` exactly). Report the full diff + the verbatim gate
output; do NOT commit.
