# Arc 277.1c-fix — the concat→format auto-fix (bare-symbol slots only; the four-questions line)

> **STATUS: SHIPPED (2026-06-17).** Bare-symbol concat -> format auto-fix; compound stays report-only (naming deferred to arc-278 RETE map). Weighed on own build: gate 2/2 (eyeballed: `(string::concat "x: " a " y: " b)` -> `(:wat::core::format "x: {a} y: {b}" :a a :b b)`), deftest 262/1, deporder 0, lib 929/36. The eligibility check shipped as a nested-if disjunction (the smell this tool abolishes — builder caught it); cleaned to `(not (or ...))` (see REALIZATIONS R2 — it names the generalized boolean-ladder rule). RED probe `tests/probe_arc277_1c_concat_format_autofix.rs`
> (`#[ignore]`'d): bare-symbol concat must rewrite to `format`; compound concat must stay report-only.
> Gives 277.1c's report-only concat-abuse rule a real fix — but ONLY where naming is mechanical.

## The four-questions decision (why bare-symbol only)

`format` is **named-only** — the auto-fix must name every slot. A naming HEURISTIC (derive a name from a
compound value's leaf token) FAILS the four-questions: **Obvious? NO** (a reader can't predict the name);
**Simple? NO** (it braids leaf-extraction + accessor-casing + dedup + synthetic-fallback — multiple
concepts). And the RETE direction (the detection moves to a pure rules engine; `pure in → pure out`, a
map of things to fix) forbids smuggling **judgment** (a compound slot's name) in as **fact**.

So the honest line: a slot is auto-nameable **iff it already has a name** — a bare symbol `count` → the
placeholder `{count}` + kwarg `:count count`. That passes all four (Obvious: name *is* the value; Simple:
no heuristic; Honest: the name cannot lie; Good-UX: self-documenting). A **compound** slot has no honest
derivable name → the concat stays **report-only** (`fix = None`); its naming is a judgment deferred to the
arc-278 RETE map-consumer (where the engine emits the decomposition + a name-hole, never a guess).

## The fix (THE CONTRACT) — extends the 277.1b machinery

Reuses `FixEdit` + `apply-fixes` + `fix-text-span-len` (277.1b) + `ast-span`/`ast-end-span` (281). Only
`make-concat-finding` changes: compute the `fix` field.

`concat-format-fix [form] -> Option<FixEdit>`:
1. `args = (drop (ast->children form) 1)` — the concat operands.
2. **Eligibility:** every NON-string-literal arg must be `ast-kind == "symbol"`; AND no string-literal
   arg may contain `"`, `{`, or `}` (keeps the template build Simple + safe — escaping is out of scope
   for this stone; such a concat stays report-only). If ineligible → `None`.
3. **Build template + kwargs** by folding `args` in order:
   - string literal → append its content (`ast-name` of the string node — returns the inner text) to
     `template`.
   - symbol → let `nm = (ast-name sym)`; append `"{" nm "}"` to `template`; add the pair `nm → sym` to an
     ordered kwarg map (DEDUP: the same symbol name maps once — format allows N `{nm}` against one `:nm`).
4. **Emit new-text** (a String, spliced by `apply-fixes`):
   `"(:wat::core::format \"" + template + "\"" + (for each deduped kwarg: " :" + nm + " " + nm) + ")"`.
   e.g. `(:wat::core::format "x: {a} y: {b}" :a a :b b)`.
5. `fix = Some(FixEdit start-line start-col end-line end-col new-text)` (extent = `ast-span` +
   `ast-end-span` of the concat `form`, exactly like the ladder fix).

`make-concat-finding` sets `fix = (concat-format-fix form)` (was `None`). The detection
(`concat-abuse?` / `rule-concat-abuse-form`) is unchanged — it still reports every concat-abuse; only the
bare-symbol ones now also carry a fix.

## Proof

- `tests/probe_arc277_1c_concat_format_autofix.rs` (un-ignore):
  - `(concat "x: " a " y: " b)` → `lint-fix-file` → `(:wat::core::format "x: {a} y: {b}" :a a :b b)`, no concat.
  - `(concat "n=" (i64::to-string n))` → UNCHANGED (compound slot → report-only, no format).
- deftest in `wat-tests/lint.wat` (Case 8): same two cases + a same-symbol-twice dedup case.
- Floors: lib 929/36, deftest (+1 → 262/1), deporder 0. The ladder fix + concat report stay green.

## Out of scope (rejected, not deferred-vaguely — bounded to arc 278)

- **Compound-slot naming** — a JUDGMENT; the arc-278 RETE map-consumer resolves it (engine emits
  decomposition + name-hole). NOT synthetic `{arg0}` noise. `violation->finding` waits for that layer.
- **Brace/quote escaping in literal text** — a literal containing `"`/`{`/`}` makes the concat
  report-only here; the escaping path can land when a real corpus case needs it.
- **The corpus sweep** — this stone ships the bare-symbol *mechanism*; the sweep is its own stone.

## Four questions (of the shipped scope)

- **Obvious?** YES — `{a}`/`:a a` for a slot named `a` reads exactly as itself.
- **Simple?** YES — no heuristic; one fold building template + kwargs; eligibility is a flat predicate.
- **Honest?** YES — fixes only where the name is a fact; declines (report-only) where it would be a guess;
  the decline is the honesty.
- **Good UX?** YES — the auto-fixed `format` is self-documenting; the un-fixed compound concat still gets
  a finding (the human/RETE-layer handles it), never a faked name.

## Blast radius

`wat/lint.wat` — `concat-format-fix` helper + `make-concat-finding` sets `fix` (was `None`). A
`wat-tests/lint.wat` deftest. Un-ignore the probe. No Rust changes (rides FixEdit/apply-fixes/ast-end-span).
