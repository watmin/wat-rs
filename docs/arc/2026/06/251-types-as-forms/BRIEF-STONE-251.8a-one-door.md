# BRIEF — STONE 251.8a: one door for "is this symbol a reference?"

Read `DESIGN-STONE-251.8-symbol-proper.md` first; it carries the ruling, the measurements and the
intueri-cast names. This brief is the strike.

## THE WORK, in one paragraph

Four places in the substrate each decide, independently, whether a `WatAST::Symbol` is a
namespaced **reference** or a local **binder**, and all four decide it the same way: by testing
whether the identifier's string contains a `/`. Collapse those four onto one door. The door is a
pair of functions on `Identifier` — `namespace()`, which is **total** and returns the reserved
`$bound` namespace for a binder, and `reference?`, which is true exactly when the namespace is not
`$bound`. Reserve `$bound` so user source cannot define into it. Nothing else moves: no reference
changes node kind, no user form changes, and the corpus does not notice.

## ROOMS — read in this order

1. **`crates/wat-reader/src/identifier.rs:79-140`** — the whole `Identifier` surface: the struct
   (`name`, `scopes`), the sole constructor `bare` (`:94`, with its `\u{1}` debug assert and a
   `rune:struere` explaining why validation is debug-only), `add_scope` (`:115`), `as_str`
   (`:126`), `scopes` (`:137`). This is where the two new functions live. 189 lines total — read
   it whole, it is small.

2. **`src/resolve/reserved.rs`** — `RESERVED_PREFIXES` (`:14`) and `is_reserved_prefix` (`:34`).
   Note the matching shape: it strips a leading `:` then does `starts_with`, so entries are in
   **doubled-colon** form (`":wat::"`, `":rust::"`). `$bound` joins as the literal `":$bound::"`.
   The doc comment above the list explains what the reservation buys; extend it in the same voice.

3. **The four sites to collapse**, each with why you are being sent there:
   - `src/resolve/normalize.rs:81` — the match guard that decides a symbol is a namespaced ref and
     rewrites it. The canonical one; read its surrounding module doc (`:1-36`) for the
     data-position discipline it obeys.
   - `src/macros/expand.rs:537` — the same question during macro expansion.
   - `src/runtime.rs:3048` and `:3307` — the same question, twice, at eval.

4. **`wat-scripts/scratch-pad/probe-251-keyword-vs-colon-quoted-symbol.wat`** — the committed
   probe that established the spelling discriminator. It must still exit 0 and print its same
   three values when you are done.

## IMPLEMENTATION SKETCH — the shape; fill it, do not invent a different one

```rust
// crates/wat-reader/src/identifier.rs

/// The reserved namespace every non-namespaced (binder) symbol carries.
/// Reserved so user source cannot define into it — see src/resolve/reserved.rs.
pub const BOUND_NAMESPACE: &str = "$bound";

impl Identifier {
    /// The symbol's namespace. TOTAL — every symbol has one; a binder's is
    /// `$bound`. Never an absence: the uniform shape is the point (see the
    /// design's pinned contract).
    pub fn namespace(&self) -> &str { /* derived from the spelling in 8a */ }

    /// True when this symbol names something defined elsewhere, false when it
    /// is a local binder. Exactly `namespace() != BOUND_NAMESPACE` — state that
    /// identity in the doc comment; the design flags it as the one indirection
    /// a reader has to cross between `$bound` and `reference?`.
    pub fn is_reference(&self) -> bool { /* ... */ }
}
```

Then the four sites become `ident.is_reference()`. Keep `as_str()` returning exactly what it
returns today — **its meaning does not change in this stone**, and no caller of it moves.

## BLAST RADIUS

`crates/wat-reader/src/identifier.rs`, `src/resolve/reserved.rs`, and the four named sites. No
new types. No signature changes to existing functions. No `.wat` file changes. If you find
yourself editing a fifth site to make this compile, that is STOP-3.

## STOP TRIGGERS — each means ship nothing and report the gap

**STOP-1 — the scope cascade.** If the two functions cannot be added without threading a new
parameter through call sites beyond the rooms above, stop and report the count and the sites. The
recorded discipline is explicit: put state on the reference already threaded and set it at the
boundary; never thread a new parameter through the world.

**STOP-2 — hygiene interaction.** `Identifier` carries `scopes: BTreeSet<ScopeId>` for macro
hygiene, and `bare`'s debug assert exists to keep a scope from being baked into a name. If your
change makes that assert fire, or changes `env_key`'s encoding, or changes
`hash_canonical_program`, stop. Those are load-bearing for the execve boot wire and belong to a
different stone.

**STOP-3 — the four sites do not agree.** If collapsing any of the four changes its behaviour,
stop and report which one and how. They are supposed to be four spellings of one question. If
they are not, that divergence is a bigger finding than this stone and must be surfaced, not
smoothed.

**STOP-4 — `$bound` is not free after all.** The design measured it free (0 corpus hits) and
proved `$binder/x` resolves to nothing. If reserving it turns anything red, stop and report what
claimed it.

## THE GATE

1. A RED probe, written **before** the change and **mutation-tested**: assert that a binder
   identifier answers `$bound` from `namespace()` and `false` from `is_reference()`, and that a
   namespaced one answers its own namespace and `true`. Then break `is_reference` deliberately,
   confirm the probe goes red, and restore. A gate you have not watched fail is a claim, not a
   proof — report the mutation result explicitly.
2. `grep -n "contains('/')" src/` returns **zero** in the four named files.
3. `cargo clippy --release --all-targets` — zero warnings.
4. `cargo build --release` — exit 0.
5. `wat-scripts/scratch-pad/probe-251-keyword-vs-colon-quoted-symbol.wat` exits 0 and prints
   `:foo`, `:my.app/status`, `:wat.core/+` unchanged.

Run every command in the **foreground** and block on it; your turn ends when the numbers are in
your hands. The orchestrator runs the full floor centrally and weighs the kill by its own re-run —
report what you measured, and report the mutation result even if it surprises you.

## A PRIOR RESULT TO COPY FOR SHAPE

`docs/arc/2026/06/278-rules-engine/` — the `ONE DOOR for a type head's FQDN` stone (task #75, 17
hand-rolls collapsed, both defensive branches deleted). Same shape as this: many independent
re-derivations of one question, replaced by a single named door, with the old branches removed
rather than left beside the new one.
