# BRIEF — argspec WARD R3 — annihilate the R2 vigilia findings (earn the stamp)

You are sonnet. The `src/argspec/` home's R2 vigilia re-cast did NOT converge — 8
spells surfaced 1 doc-lie L1 + an L2 cluster (several introduced or surfaced by the
R2 refactor itself). This sweep annihilates every must-fix so a clean R3 re-cast can
earn the `vigilatum` stamp.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after the
re-cast converges. If you think a commit is needed, STOP and say so. (#1 rejection
trigger.)

Work ONLY in `/home/watmin/work/holon/wat-rs/`. Files in scope:
`src/argspec/error.rs`, `src/argspec/parse.rs`, `src/argspec/mod.rs`, `src/ast.rs`,
`src/macros.rs`, `src/function/parse.rs`, and the probe
`tests/probe_arc241_stone1_argspec_canonical.rs` if an assertion needs the new
wording.

---

## FIX 1 (L1 — doc lie) — `parse_triple`'s doc is false post-R2

`src/argspec/parse.rs` `parse_triple` doc (currently ~lines 146-147) says:
> "The `&[WatAST; 3]` type enforces the length precondition at the call site —
> callers convert via `try_into()`."

This was true before R2. After R2, `parse_triple`'s SOLE caller
(`parse_argspec_triples`) receives the `&[WatAST; 3]` from `extract_triple`, which
does the `try_into().expect()`. `parse_triple`'s callers do NOT convert. The doc lies.

Fix — reword to the truth, e.g.:
```
/// Parse a single `name <- :T` triple. The `&[WatAST; 3]` type makes the
/// length precondition structural — `extract_triple` performs the `try_into`
/// before handing the fixed-size reference here.
```
(Keep the second sentence listing the per-slot failure variants as-is.)

---

## FIX 2 (L2 — Pattern A purity) — `MalformedTypeKeyword` stores a redundant span

`src/argspec/error.rs` — the variant is `MalformedTypeKeyword { inner: Box<TypeError> }`.
Only `inner.kind` is ever read (in `reason()`); `inner.span` is dead — it's the same
`kw_span` already carried by the outer `ArgSpecError.span`. A Pattern A kind enum must
carry ONLY variant-specific data; smuggling a redundant span inside `inner` violates
that.

Fix:
- Change the variant to `MalformedTypeKeyword { inner: Box<TypeErrorKind> }` (keep it
  boxed to keep `ArgSpecErrorKind` small — avoids clippy::large_enum_variant).
- `reason()` arm: `format!(...)` now reads `inner` directly (it IS the
  `TypeErrorKind`, which Displays span-free post-R2). See FIX 3 for the new wording.
- `src/argspec/parse.rs` `parse_keyword_type` construction site: change
  `inner: Box::new(inner)` → `inner: Box::new(inner.kind)` (extract the kind from the
  `TypeError` returned by `parse_type_expr_with_span`).
- Confirm `TypeErrorKind` is `pub` and imported in error.rs (it already is — error.rs
  imports `TypeErrorKind`).

If the probe constructs `MalformedTypeKeyword`, update it to the new shape.

---

## FIX 3 (L2 — cernere: doubled "malformed") — reword the MalformedTypeKeyword reason

After FIX 2, `reason()` renders `format!("type keyword is malformed: {}", inner)`
where `inner` (a `TypeErrorKind`) Displays e.g. `"malformed type expression …"`. Result:
`"type keyword is malformed: malformed type expression …"` — doubled "malformed".

Fix the outer wording to drop the duplicate:
```rust
ArgSpecErrorKind::MalformedTypeKeyword { inner } =>
    format!("invalid type keyword: {}", inner),
```

---

## FIX 4 (L2 — cernere: "slot" jargon) — `NameNotSymbol` reason

`error.rs` `NameNotSymbol` reason is the lone user-facing message still using "slot"
(every other message dropped it). Change:
```rust
ArgSpecErrorKind::NameNotSymbol =>
    "name must be a plain symbol (not a keyword, literal, or nested form)".into(),
```

---

## FIX 5 (L2 — struere: span-fallback asymmetry) — push the fallback into extract_triple

`src/argspec/parse.rs` — the two `extract_triple` call sites compute the
`IncompleteTriple` span asymmetrically: the fixed-param path passes
`args_vec[cursor].span().clone()` unconditionally; the rest-binder path pre-computes a
conditional `rest_span` (element-span-or-form_span). The conditional belongs INSIDE
`extract_triple`.

Fix:
- Change `extract_triple`'s signature from `span: Span` to `fallback_span: &Span`.
- Inside, compute the attributed span:
  ```rust
  let span = if start < args_vec.len() {
      args_vec[start].span().clone()
  } else {
      fallback_span.clone()
  };
  ```
  and use it in the `IncompleteTriple` error.
- Both call sites pass `form_span` as `fallback_span`. Delete the now-dead
  `elem_span` local (fixed path) and the `rest_span` conditional (rest path). The
  rest path keeps `rest_start` only if still needed for `trailing_start`; otherwise
  use `cursor` directly. Keep it readable.

---

## FIX 6 (L2 — solvere: home purity) — evict `is_bare_symbol` to `ast.rs`

`is_bare_symbol` is a pure `WatAST` predicate (no argspec dependency) squatting in
`argspec/parse.rs` and re-exported as argspec API. Two of its callers
(`macros.rs:~430`, `function/parse.rs:~81`) use it for the `->` token, which is NOT an
argspec concern. A warded home holds only its own residents.

Fix:
- Add to `src/ast.rs` as an inherent method on `WatAST`:
  ```rust
  impl WatAST {
      /// Returns true if this is a bare `Symbol` whose name equals `name`.
      /// Used to detect structural tokens (`<-`, `->`, `&`) without allocating.
      pub(crate) fn is_bare_symbol(&self, name: &str) -> bool {
          matches!(self, WatAST::Symbol(ident, _) if ident.name == name)
      }
  }
  ```
  (If an `impl WatAST` block already exists in ast.rs, add the method there rather
  than opening a new block.)
- Delete `is_bare_symbol` from `argspec/parse.rs` and its `pub(crate) use` re-export
  from `argspec/mod.rs`.
- Update all call sites to the method form `x.is_bare_symbol("...")`:
  - `argspec/parse.rs` internal uses (the `&` and `<-` checks)
  - `macros.rs` (the `->` check)
  - `function/parse.rs` (the `->` check)
  - grep `is_bare_symbol` across `src/` to catch every site.

---

## FIX 7 (L2 — intueri: doc staleness)

- `src/argspec/mod.rs` — the line stating the failure class "is closed WHEN the four
  old parsers retire" is stale future-tense; they HAVE retired (Stones 241.1–241.5
  DONE). Reword to past/present: e.g. "the class is closed: all four old parsers were
  migrated through Stones 241.1–241.5 and retired."
- `src/argspec/error.rs` — the `From<>` impls comment attributing the set to
  `AUDIT.md "Recommendation for 241.1"` over-attributes (AUDIT recommended three;
  MacroError was added later at the defmacro migration). Reword to not claim AUDIT
  prescribed all four — e.g. drop the stone-specific attribution: "Wire each
  call-site's native error class to the canonical `ArgSpecError`."

---

## FIX 8 (L3 → comment only) — truer MacroError head-drop comment

`error.rs` `From<ArgSpecError> for MacroError` drops `e.head`. This is CORRECT and is
NOT a rune (nothing to solve): `MalformedDefmacro` is defmacro-specific BY ITS VARIANT
NAME, so the head is structurally implied, not lost — unlike the generic
`MalformedForm`/`MalformedDecl` variants that serve many heads and therefore need the
field. Replace the existing comment with the truer reason:
```rust
// e.head is not threaded: MalformedDefmacro is defmacro-specific by variant name,
// so the form identity is structural here (unlike the generic MalformedForm/
// MalformedDecl variants that carry head because they serve many forms).
```
No structural change.

---

## DO NOT TOUCH (L3 — leave per let-need-reveal / runes-illegal-when-nothing-to-solve)

- temperare allocations (parse-time, no proven cost) — LEAVE.
- `IncompleteTriple` available-item count — LEAVE (speculative).
- `ParseOptions` rename — LEAVE.
- `ArgSpecError` gaining its own `Display`/`Error` impl — LEAVE (single-source via the
  four From impls is intentional).
- Do NOT add any `rune:` line for the MacroError head-drop — it's correct, FIX 8 is a
  plain comment.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890+ / 0)
- `cargo test --release --test probe_arc241_stone1_argspec_canonical` (expect all pass)
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat` (must compile)
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (report the count; must not
  REGRESS vs ~876 baseline — the Box<TypeErrorKind> change must not add a
  large_enum_variant warning)

Report: every file touched + one-line description each; the new `MalformedTypeKeyword`
variant decl + its `reason()` arm; the new `extract_triple` signature + body; the
`is_bare_symbol` method + confirmation every call site was updated (paste the grep);
the five gate numbers; explicit confirmation of ZERO git mutations. Raw report.
