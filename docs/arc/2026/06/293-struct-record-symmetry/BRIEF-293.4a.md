# BRIEF — 293.4a: method members in `defsurface` (parse + satisfy)

> **⊘ CORRECTION (mid-build, 2026-06-28 — builder + orchestrator; amend-with-recognition):** the suggested
> `Method { arg_types: Vec<TypeExpr>, … }` shape below was WRONG. There is exactly ONE canonical representation of a
> typed binder list — `crate::argspec::ArgSpec` (what `parse_argspec_triples` returns; the Field path already uses it;
> the arc's own DESIGN decision #2 = "defsurface = a named ArgSpec"). A second `arg_types: Vec<TypeExpr>` is the
> decomplection this arc exists to kill. **As built:** `SurfaceMember::Method { name, args: ArgSpec, ret, type_params }`
> — the member carries the full `ArgSpec`, NOT a flatten. (The original `parse_defprotocol_form` flatten —
> `spec.fixed_params.iter().map(|(_,ty)| ty).collect()` — is the original sin; mirrored its *parse*, dropped its
> *flatten*.) The `arg_types: Vec<TypeExpr>` text in § "the one contract decision" and § "Read in order" is kept below
> with recognition; the ArgSpec shape is what shipped. **Second delta:** STOP-1 (CheckEnv unreachable) did NOT fire —
> `assignable` only had `&TypeEnv`, so the executor threaded `&CheckEnv` THROUGH `assignable` (17 call sites,
> +35/-17 check.rs); the correct seam, broader than this brief drew.

**The work, in one paragraph.** Today a `defsurface` member list is field-only — `parse_defsurface`
(`src/types/surface.rs:48`) runs the whole member vector through `argspec::parse_argspec_triples`, which only
understands `[name <- :T]` field triples; a method member `(area [self] -> :f64)` (a list element inside the vector)
fails to parse. Make `defsurface` carry **method members** alongside field members, and make a type **satisfy** a method
member by exposing a matching `defn :T/<name>`. This is "methods are accessors": the surface lists required accessors;
the satisfier backs each with a **field** (free accessor) OR a **method** (a `defn`), invisibly to the surface. NO
dispatcher (`:Shape/area s`) here — that is 293.4b; this slice is **parse + structural satisfaction** only.

## The one contract decision (pinned — `DESIGN-293.4-strike.md` § 293.4a)
A `defsurface` member is **a field member `name <- :T`** OR **a method member `(name [self …] -> :ret)`**. Satisfaction
is **structural + width-open**: a type `T` satisfies the surface iff for EVERY member —
- **field member** → `T` has a field `name` with an assignable type (today's `struct_satisfies_surface` logic), OR
- **method member** → a `defn :<T>/<name>` exists whose signature is assignable to the member's declared sig.

No `:satisfies`, no declaration at the satisfier. A field can satisfy a *field* member; a `defn` can satisfy a *method*
member. (The full field-OR-method-back-either-accessor symmetry — a field satisfying a method member, or vice versa — is
NOT required for 293.4a; the acceptance demo's `color` (field member backed by a field, and by a method on the Vector)
is reached across 293.4a+c. Keep 293.4a to: method members parse + a method member is satisfied by a `defn`.)

## Read in order (the rooms — grounded 2026-06-28)
1. **`src/types.rs:233`** — `pub struct SurfaceDef { name, type_params, members: Vec<(String, TypeExpr)>, holder }`.
   Change `members` to carry field-or-method. Mint a `SurfaceMember` shape in the **types layer** (do NOT make `types`
   depend on `value::ProtocolMethodSig` — that risks a layer cycle; `value` already imports `types::TypeExpr`). Suggested:
   ```rust
   pub enum SurfaceMember {
       Field  { name: String, ty: TypeExpr },
       Method { name: String, arg_types: Vec<TypeExpr>, ret: TypeExpr, type_params: Vec<String> },
   }
   ```
   (structurally mirrors `value::ProtocolMethodSig` but types-local). Let the compiler waterfall every `SurfaceDef.members`
   reader (the fail-count is the meter): `parse_defsurface`, `struct_satisfies_surface`, any reflection/Display, the
   `register_*`/`from_symbols` paths that serialize a `SurfaceDef`.
2. **`src/types/surface.rs:48` (`parse_defsurface`)** — the member vector now MIXES field triples and method lists, so it
   can no longer go straight through `parse_argspec_triples`. Walk the member `items` yourself: a run of `name <- :T`
   (Symbol, `<-` Symbol, type) → a `Field`; a `WatAST::List(...)` element → a `Method`, parsed by **mirroring
   `parse_defprotocol_form`'s per-sig logic** (`src/runtime.rs:5844` — `(name [self <- :Self  arg <- :T …] -> :R)`,
   `split_name_and_type_params` for `name<T>`, arg-types from the argspec, ret keyword). You may factor the protocol
   method-sig parse into a shared helper both call, OR copy its shape into `surface.rs` (judge by which keeps both homes
   clean — `parse_defprotocol_form` returns a `RuntimeError`; `parse_defsurface` returns a `TypeError`, so a shared helper
   must be error-type-agnostic or you adapt). Field-triple parsing can still reuse `parse_argspec_triples` over the
   field-only sub-runs.
3. **`src/types/surface.rs:26` (`struct_satisfies_surface`)** — extend to method members. Today it takes
   `(struct_fields, surface, is_assignable)`. A method member can't be checked from `struct_fields` alone — it needs to
   look up `defn :<T>/<name>`. ADD a method-resolver closure, e.g.
   `resolve_method: FnMut(&str /* accessor fqdn like ":t::Sq/area" */) -> Option<(Vec<TypeExpr>, TypeExpr)>` (the defn's
   arg-types + ret), supplied by the caller from `CheckEnv`. A `Method` member is satisfied iff the resolver returns a sig
   whose arg-types/ret are assignable to the member's (receiver arg = the satisfying type).
4. **`src/check.rs:14380`** (the `struct_satisfies_surface` call site) — this is where `CheckEnv` is in scope. Build the
   `resolve_method` closure here: given the candidate type's FQDN `:<T>` and a member name `m`, look up the function
   `":<T>/" + m` in the env's function/scheme table (grep how `:T/field` accessors + `defn` fns are stored — the same
   table the existing accessor-call type-checks against). Pass the candidate type's name through so the closure can form
   the `:<T>/<name>` key.

## Implementation sketch (the strike path — fill it, don't invent the shape)
- `SurfaceMember` enum in `types.rs`; `SurfaceDef.members: Vec<SurfaceMember>`.
- `parse_defsurface`: walk member items → `Vec<SurfaceMember>` (triples → Field, lists → Method via the lifted sig parse).
- `struct_satisfies_surface(struct_fields, surface, is_assignable, resolve_method)`: per member, Field→field-match,
  Method→`resolve_method(":<T>/"+name)` returns an assignable sig.
- `check.rs:14380`: build `resolve_method` from `CheckEnv`, thread the candidate type name, call the extended fn.

## Blast radius (bounded)
`src/types.rs` (the struct + enum), `src/types/surface.rs` (parse + satisfy), `src/check.rs` (the one call site +
the resolver), plus the mechanical cascade of `SurfaceDef.members` readers the enum change surfaces. **No new
user-facing forms, no runtime dispatch, no `defprotocol` touch.** If the cascade reaches `value/` serialization of a
`SurfaceDef`, that's in-scope (the enum must round-trip), but do not redesign the repr.

## STOP triggers (halt + surface; do NOT improvise)
- **STOP-1 (the method-resolver seam):** if, at `check.rs:14380`, you cannot reach the `defn :<T>/<name>` signatures from
  the `CheckEnv` in scope (the function/scheme table isn't available there, or accessor fns are stored somewhere a closure
  can't reach) — STOP and report the seam. Do NOT fake satisfaction by name-existence-without-sig-check.
- **STOP-2 (member-walk ambiguity):** if a member vector shape appears that is neither a clean `name <- :T` triple-run nor
  a `(name [args] -> :R)` list (e.g. a bare symbol, a stray keyword) — STOP and name it; do not guess a third member kind.
- **STOP-3 (the parser shared-helper error-type clash):** if factoring `parse_defprotocol_form`'s sig-parse into a shared
  helper forces an ugly error-type bridge (RuntimeError vs TypeError) — copy the ~20-line sig-parse shape into `surface.rs`
  instead (a clean copy beats a contorted shared helper); note it in the SCORE.

## The gate (the disconfirming probe, committed RED)
`tests/types/probe_arc293_4a_method_members.rs` + `.wat` — a `defsurface` mixing a field member (`color <- :String`) and a
method member (`(area [self] -> :f64)`); a `defrecord :t::Sq` backing `color` with a field and `area` with a
`defn :t::Sq/area`; a consumer `(:t::accept [s <- :t::Shape])` passed `(:t::Sq "red" 3.0)`. RED at HEAD (the method member
fails `parse_argspec_triples` — verified: `MalformedDecl "triple is incomplete"`). **UN-IGNORE it; it goes GREEN when
293.4a lands.** Do NOT touch the bigger `probe_arc293_acceptance_demo` (that is 293.4d's gate — stays `#[ignore]`'d).

## You are a LEAF
Anchor cwd `/home/watmin/work/holon/wat-rs`; `pwd` first; reject any `.claude/worktrees/` path. Do NOT spawn subagents. Do
NOT commit (the orchestrator weighs + commits). Build incrementally (`cargo build --release -p wat`, let the
exhaustive-match cascade waterfall you). Read every diff end-to-end. Self-verify the gate (the EXPECTATIONS scorecard).
If a STOP fires or the work exceeds this brief, halt and report.
