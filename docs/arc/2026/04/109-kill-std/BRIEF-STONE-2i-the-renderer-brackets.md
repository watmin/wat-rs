# BRIEF — ②-i: the type renderer brackets its args, and gains a head-spelling mode

Design: `DESIGN-STONE-2-the-codemod.md`. This is the **first and smallest** cut of step ②: `src/` +
the goldens that follow from it. **No `.wat` corpus migration happens here** — that is ②-ii/iii/iv,
and it cannot be written until this lands.

**Your role: you write the text. The orchestrator builds, floors, and clippies.** No `cargo`, in any
form. `./target/release/wat` is prebuilt and will NOT reflect your Rust changes. Foreground
everything; ending your turn ends you. Do not commit, push, stash, or revert.

## The work in one paragraph

`type_expr_to_clojure_form` (`src/edn_shim.rs:1200`) renders a `TypeExpr` into a faithful-Clojure
form. It is wrong in two ways for the migration ahead: it splices parametric type-args **FLAT**
(`(wat.type/Vector wat.type/i64)`) where the builder ruled a **bracketed** vector on 2026-07-24
(`(wat.type/Vector [wat.type/i64])`), and it always renders the head in the **`wat.type/`** spelling
when step ② needs the rust-ish `:wat::core::` one. Make the bracketing unconditional, add a
head-spelling mode, and expose the new mode as a sibling wat verb.

## Room 1 — bracket the args. UNCONDITIONAL, both modes.

`src/edn_shim.rs`, the `TypeExpr::Parametric` arm at **`:1249-1253`**:

```rust
let mut items = vec![WatAST::Symbol(Identifier::bare(sym), unk.clone())];
for a in args {
    items.push(type_expr_to_clojure_form(a)?);   // ← spliced FLAT
}
WatAST::List(items, unk)
```

The args become **one `WatAST::Vector`** in the list's second position: `(Head [a b])`.

⚠ **Do NOT touch the `TypeExpr::Fn` arm immediately below (`:1255-1262`).** It splices into a
`WatAST::Vector` with a `:->` keyword — that is the `[A :-> B]` function-type form and it is
**correct**. The two loops read identically at a glance; only the Parametric one is the defect. (I
mis-attributed this myself once already.)

★ This closes `300/NOTE-the-type-converter-emits-the-superseded-form.md`, which names this exact arm
as a blocker for 300.1. ⚠ That note's line numbers have drifted (it says `:1183`/`:1232`; the fn is at
`:1200` and the arm at `:1249`) — confirm by matching code, and update the note.

## Room 2 — the head-spelling mode

The `TypeExpr::Path` arm (`:1204`) and the `Parametric` arm (`:1225`) each carry a **4-way ladder**:
core FQDN → bare primitive → user type (`::`) → type-var. Both render `wat.type/…` symbols today.

Add a mode. Two behaviours, one renderer:

```
CLOJURE  (today's)   (wat.type/Vector [wat.type/i64])          ← the later flip
COLON    (new)       (:wat::core::Vector [:wat::core::i64])    ← step ② needs this
```

In COLON mode a core FQDN renders as a **`WatAST::Keyword`** (`:wat::core::Vector`), not a Symbol —
that is a node-kind difference, not just a string. Decide and state what COLON does for the other
three ladder rungs; the type-var rung in particular has no colon form.

⚠ **Parameterize; do not fork.** A second copy of this renderer is how the two spellings drift apart.

## Room 3 — expose COLON as a wat verb

`eval_keyword_to_type_form` (`src/edn_shim.rs:1284`) is the existing wrapper, registered in **three**
places — mirror all three for the sibling:

```
src/check.rs:18800        TypeScheme  (WatAST -> WatAST)
src/runtime.rs:5214       dispatch arm
src/macros/eval.rs:667    the macro-eval allowlist
```

Name it in the existing family's style. The existing verb keeps its name, its arity and its
CLOJURE behaviour — only its *bracketing* changes, per Room 1.

## ⚠ THE GOLDENS MOVE — 36 occurrences across 29 files. This is expected, not a regression.

Bracketing changes what the renderer emits, so every fixture asserting the flat form goes red:

```
tests/reflection/**       9 files   signature-of / extract-arg-types
tests/resolve/**         16 files   probe_arc251_keyword_to_type_form + parametric targets
tests/wat_lang/**         3 files
tests/function/**         1 file
```

They are the deliverable as much as the arm — **each must be updated to the bracketed form and each
must still assert the same thing.** A golden "fixed" by weakening its assertion is the failure mode
this stone can hide behind.

⚠ **`tests/resolve/probe_arc251_keyword_to_type_form*`** is the CONTRACT SUITE for this exact verb —
eight `contract-NN` fixtures covering parametric, nested-parametric, type-var, multi-arg, tuple,
nested-tuple. Read it before editing anything; it tells you what the verb is supposed to guarantee,
and it is the strongest check that your change is right rather than merely green.

## Blast radius

`src/edn_shim.rs` · `src/check.rs` (one register) · `src/runtime.rs` (one arm) ·
`src/macros/eval.rs` (one allowlist entry) · the 29 golden files. **No `wat/` corpus migration.**

## STOP triggers — each rejects; none is a fallback

1. A golden cannot be updated without weakening what it asserts. STOP; report which and why.
2. The `TypeExpr::Fn` arm needs changing. STOP — it is correct; `[A :-> B]` is the function-type form.
3. COLON mode has no sensible rendering for a ladder rung (type-var is the candidate). STOP; report.
4. A `wat/` file needs editing. STOP — the corpus migration is ②-ii and later.

## Acceptance criteria

- Parametric args render as `(Head [a b])` in **both** modes; nested parametrics nest.
- A COLON-mode call yields `(:wat::core::Vector [:wat::core::i64])` — colon-quoted head, keyword node.
- The existing verb keeps its name, arity and `wat.type/` head spelling.
- The sibling verb is registered in all **three** places.
- All 29 golden files updated, none weakened; `probe_arc251_keyword_to_type_form`'s eight contract
  fixtures still assert their original contracts.
- No `wat/` file touched.
