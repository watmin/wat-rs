# DESIGN — Stone 249.5: hygiene completion (real sets-of-scopes resolution)

> Status: STRIKE-DRAWING.
> RED contract committed: `tests/probe_macro_hygiene_capture.rs` (`07ce045f`).
>
> **Home name `src/scope/`** — intueri cast `a137…`: a dir name is the domain
> NOUN, not the property it achieves ("hygiene"/"Flatt 2016" → `mod.rs` prose;
> precedent: `collection/` not `type-polymorphism/`, `remedy/` not
> `correctness/`). Structure:
> ```
> src/scope/
> ├── mod.rs          — vigilatum stamp (when earned); "hygienic per Flatt 2016" prose; re-exports
> ├── identifier.rs   — Identifier, ScopeId, fresh_scope(), add_scope() — scope-identity primitives
> └── resolution.rs   — scope-aware reference resolution + canonical renumbering — the policy
> ```

## Why

The arc-249 macros re-ward's final guard (circumspicere) surfaced a claim-vs-code
contradiction the green suite + 7 other lenses missed: `src/macros/mod.rs:33`
asserts *"variable capture is structurally impossible"* — but a 12-line probe
proves a macro's `let [tmp …]` **captures** the caller's `tmp` (`200`, not the
hygienic `105`).

Grounded (examinare lair-study):
- `walk_template` (expand.rs:681) **correctly tags** every template symbol with a
  fresh macro `ScopeId`; unquoted user symbols keep their own scopes. The tagging
  is right.
- Resolution is **name-only**: `Environment = HashMap<String, BoundEntry>`; ~30
  bind sites (`.bind(ident.name…)`, `.bind_unknown_span`) + the lookup/Symbol arms
  (runtime.rs:5185/5268) all drop `.scopes`. The tag is **inert**.
- This is **documented deferred work**, not a regression: `src/hash.rs` §
  "Hygiene-scope caveat" names *"Slice 7b (real hygienic expansion with canonical
  scope numbering)"* as the fix. `mod.rs`'s claim is the premature overclaim.

**Why it hid so long (the coverage flaw):** the deferral lived in `src/hash.rs`,
an **unwarded flat `src/` file** — exigere was never cast there; "Slice 7b" is a
**phantom tracker** (names nothing in `docs/arc/`). A warded home (`macros/`)
shipped a claim resting on facts in the unwarded flat sea, and no ward surveys the
sea. Class-fix below.

## What it delivers

Real hygiene: capture structurally impossible by construction (no author gensym —
the `fresh_scope` counter IS the automatic gensym; we wire the tag it already
mints into resolution). The RED contract flips to `105`; `mod.rs`'s claim becomes
**true**.

## The contract decision (pinned)

- **Scopes stay in the `Identifier`** (not mangled into the name). Resolution and
  hashing both read `.scopes` — ONE hygiene mechanism, not two. (Rejected:
  alpha-rename into the name — leaves `.scopes` inert, forces the hasher to parse
  scope tokens out of strings, makes runtime names cryptic.)
- **Exact-match on `(name, scope-set)` first.** `walk_template` adds one scope per
  expansion level uniformly, so a binder and its references share the exact set;
  the unquoted user symbol differs. Full Racket largest-subset matching is
  **affirmatively out of scope** until the suite (the disconfirming probe) shows a
  real wat macro needs it — if a legitimate cross-scope reference goes
  `UnboundSymbol`, that clean diagnostic is the signal to add subset-matching, in
  a named follow-on. STOP-trigger, not a silent cap.
- **Canonical scope renumbering at hash time** (DFS first-appearance, per the
  hash.rs caveat's own plan) — restores cross-run determinism so per-invocation
  scope IDs stop breaking the `hash(expanded AST) IS identity` commitment. Ships
  WITH resolution; they are coupled.

## Lift-and-ward (the approach)

Don't patch in the flat dark. **Lift the flawed concern into a durable warded
home, fix it there, let the broad flat-`src` migration finish later** (that
migration is pending around arc 170/109 — explicitly NOT this stone).

The home `src/scope/` holds: the identifier/scope primitives (lifted from
`src/identifier.rs` — 161 lines, 10 import sites, clean mint) in `identifier.rs`
+ the new **resolution policy** (scope-aware match) + **canonical renumber** in
`resolution.rs`. `runtime.rs` (33 primitive uses — the `Environment`) and
`hash.rs` (6 uses) **delegate** into the home rather than reimplement.

## Decomposition

- **249.5a — mint home + lift `identifier.rs`** (behavior-preserving). Move the
  primitives; update 10 imports (`crate::identifier` → the home). Suite stays
  green. This is the "partial move to a durable home" that STARTS the flat-`src`
  migration.
- **249.5b — the fix, in the home.** Scope-aware resolution (exact-match) +
  canonical DFS renumber; `runtime.rs`/`hash.rs` delegate. Substrate-as-teacher
  cascade across the bind/lookup sites. RED contract → `105`.
- **249.5c — ward the home** (full vigilia → `vigilatum`) + re-ward `macros/`
  (`mod.rs` claim now true) + un-ignore the probe. Two stamps earned honestly.

## Out of scope = rejected (affirmative cuts)

- Full Racket largest-subset matching — deferred to a named follow-on IF the suite
  proves a wat macro needs it (STOP-trigger). Not "later"; gated on evidence.
- The broad flat-`src/*.rs` → homes migration (runtime.rs, hash.rs, check.rs) —
  the pending arc-170/109-era work; this stone lifts ONLY the hygiene seed.
- Referential transparency (template references to *definition-site* free vars) —
  wat templates reference FQDN keywords or `~`-unquote, not bare definition-site
  vars; not exercised, not built. Re-open only on a real need.

## Rooms

- `src/identifier.rs` (161 L) — lift whole.
- imports to rewrite (10): ast.rs, form_match.rs, lib.rs, closure_extract.rs,
  parser.rs, macros/expand.rs, hash.rs, check.rs, runtime.rs, macros/tests.rs.
- `src/runtime.rs` — `Environment` (struct ~1480; `lookup` 1518; `bind`/
  `bind_unknown_span` 1559/1568) + ~30 bind sites + Symbol-eval arms 5185/5268.
- `src/hash.rs` — `write_canonical_wat` Symbol arm (157-166) + § Hygiene-scope
  caveat (44-57, to be retired when 5b lands).

## Probe (committed, RED)

`tests/probe_macro_hygiene_capture.rs::classic_macro_capture_is_prevented` —
`#[ignore]`'d, asserts `105`, currently `200`. Un-ignore in 249.5b; green = kill.
