# AMEND — STONE 255.83: the whole-body template is checked too

**Drawn 2026-10-02.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `04cb3317f` (grok's
STOP-1). Commit locally on `main`; **do not push**.

## The ruling on STOP-1

Grok's census is accepted as measured: 9 Rust functions and 5 wat predicates disagree across the two spellings of one
identity (`SCORE-STONE-255.83-…` § 2); four were cured (§ 3, ledger 146 → 139).

**`is_quasiquote_form` is not a place where the two spellings should differ. The keyword side is the defect.** It is a
router (255.11's note, `src/macros/expand.rs:1734`): a macro body that is **wholly** a quasiquote template skips
`validate_macro_definition`, which holds the F5 default-deny purity gate (arc 249). Grok's own pairs show the gap:

- `macro-impure-inner` (a quasiquote **nested** in a body): `(unquote (:wat::kernel::println …))` is **refused**.
- `macro-impure-qq` (the template **is** the body), keyword spelling: startup succeeds and returns 1. An impure call
  runs at expand time past a ruled default-deny gate.

So the whole-body template is the one route whose unquote escapes are never validated. Keeping the keyword behaviour
would keep that hole; widening the router alone would extend it to the symbol spelling (255.11's warning); sending the
template through the program-body check is the wrong check for a template. **The cure:** the template route validates
its own unquote escapes, the same `validate_quasiquote_template` a nested template gets, and the router decides by
identity (`canonical_identity_of`), so both spellings take the template route and both refuse `println`. The keyword
program turning from `1` into `MalformedDefmacro` is the gate closing, **not STOP-2**.

## The work

1. **The template route checks its escapes:** a whole-body quasiquote template runs `validate_quasiquote_template` (or the
   check a nested template already gets: name it) at definition; `is_quasiquote_form` reads `canonical_identity_of`.
   `tests/resolve/probe_arc255_83_quasiquote_router.rs` changes from pinning the difference to pinning the refusal for
   both spellings, and a pure whole-body template (e.g. `` `(:wat::core::i64::+ ,a 1) ``-shaped) still defines and expands
   in both.
2. **Census the corpus first** for whole-body templates whose escapes the new check refuses (`census.sh` pre/`--diff`,
   and the floor): each is a macro that ran impure code at expand time. **STOP-4** if any exist outside tests written to
   prove this gap: list them for the builder.
3. **The rest of the live rows** (§ 2's table): `lower_call`, `walk_for_bare_primitives` (the legacy `let*`/`lambda`/
   `Char`/`Uuid`/`unit` arms, plus the `:fn(` prefix, probed), `parse_verify_algo`, `parse_payload_interface`,
   `step_list`, `classify_expr`, `source_has_config_setter`, and `walk.rs:338`'s `stem.replace('.', "::")`, each through
   the identity door, each with its differential pair as a driven test, keyword side unchanged.
4. **The wat-side five** (`:wat::lint::if-head?`, `is-defmacro-form?`, `concat-head?`, `:wat::deporder::def-form?`,
   `:wat::fix::defmacro?`): no `canonical-identity` verb exists in wat, so add **one** door verb exposing
   `canonical_identity` (one name, registered like its siblings, `@example` doc) and route the five through it. Census
   `wat/**` for any other head-by-text decision while there.
5. Then the remaining A/B ledger rows (§ 2 did not probe them all): differential-probe or cure each; drive A and B to 0.
6. The brief's gates and the floor.

## STOPs

The brief's STOP-1 (a site where the spellings **should** differ: none is ruled so today), STOP-2 (a keyword program's
behaviour changes, **except** the whole-body-template refusal ruled above), STOP-3, plus **STOP-4** above. A STOP means
STOP. Append to the SCORE, commit, **do not push**.
