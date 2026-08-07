# BRIEF — #88, mint `(:wat::rete::core::defn …)`: the rete language's DECLARED UNIT

**Design (ratified, do NOT re-derive):** `DESIGN-STONE-the-rete-defn.md` — four questions 4×YES,
the name intueri-cast, the mechanism corrected 2026-08-06 by grounding. Read it first; this brief
is the strike path only.

## The work, in one paragraph

An ordinary `defn` called from a `where` is rete-admissible **by accident of its current body**:
nobody declared it, so nothing can be broken, so nothing warns. Edit one op inside such a helper and
the failure names the *rule*, with not one frame naming the helper. Mint a sibling declaration form
`(:wat::rete::core::defn …)` that does everything `:wat::core::defn` does — same parse, same
registration in `sym.functions`, same symbol binding — **plus** it checks the body at the definition
site against the four axes the fence already measures. Record that fact as a typed marker on the
function, and change `head_ok`'s admission of a user fn from *walk the body* to *consult the marker*.

## Rooms — read in this order

| room | why you are here |
|---|---|
| `src/value/environment.rs:35` | `Function`, exactly nine fields, none metadata. The marker's home. |
| `src/value/symbol_table.rs:33` | `functions: HashMap<String, Arc<Function>>` — how `head_ok` reaches the marker by name. |
| `src/rete/purity.rs:961` | `head_ok(head, axis, sym, seen)` — `sym` is already in hand. |
| `src/rete/purity.rs:997` | **THE MEMBRANE.** `if sym.functions.contains_key(head) { return classify_fn(...) }` — this one branch changes. |
| `src/rete/purity.rs:1010` | the admission branch below it, for STOP-2 (see below). |
| `src/rete/vocabulary.rs:1385` | `RETE_MODULES` — contains `":wat::rete::core::"`. Your new name lands INSIDE it. |
| `src/rete/vocabulary.rs:1392` | `rete_op_for` — EXACT match, never a prefix scan. |
| `src/runtime.rs:2110` / `:2193` | `register_runtime_defs` / `..._form` — where a declaration form becomes a registration. |
| `wat/fix.wat:23-53` | the BOOTSTRAP / STASH-DANCE, which this strike needs (a checker change + a corpus codemod ship together). |

## The one contract decision, pinned

**The marker is a typed field on `Function`.** Not `SymbolTable.binding_metadata` (user-writable —
`check.rs:4690` tells users to write metadata maps and `wat/spawn.wat:303` does, so a marker there is
forgeable), and not a side `HashSet` on `SymbolTable` (a second source of truth that can drift from
`sym.functions`; the contract is a property of the function, not of the registry). Both alternatives
are recorded here **so they are not re-derived as cleanup**.

Shape it so #87 does not pay a second cascade:

```rust
// src/value/environment.rs — beside Function
/// The rete contract a `(:wat::rete::core::defn …)` declaration attests, checked AT THE
/// DEFINITION SITE. `#87` hangs the bound (`depth` / `nodes` / `fold_nesting`) here — adding a
/// field to THIS struct costs nothing, where a second field on `Function` would re-cascade.
pub struct ReteContract { /* #87 fills this */ }

pub struct Function {
    // … the nine existing fields …
    /// `Some` iff declared by `:wat::rete::core::defn`. `head_ok` consults this INSTEAD of
    /// walking the body — that substitution is the membrane.
    pub rete: Option<ReteContract>,
}
```

Adding the field turns **35 `Function { … }` construction sites** into compiler errors. That cascade
is the method, not a crisis — each site is a `None`.

## The strike path

1. **The form.** `(:wat::rete::core::defn :name [params] -> :Ret body)`, parsed and registered
   exactly as `:wat::core::defn` is, reusing that path rather than a parallel one.
2. **The definition-site check.** Run the body through the four existing walks in `src/rete/purity.rs`
   — `is_pure_expr` / `is_deterministic_expr` / `is_total_expr` / the law-A rete-primitive check.
   Reuse them; a second copy is the stone's own law violated (STOP-1).
3. **The marker.** On success, register the `Function` with `rete: Some(ReteContract{})`.
4. **The membrane.** `purity.rs:997` — a fn with `rete: Some(_)` is admitted on the strength of its
   declaration; a fn with `rete: None` is refused with a located error naming **the helper**, not the
   rule. That inversion is the whole point of the stone.
5. **The migration.** The corpus now screams; those screams are the worklist (R52/R65). Re-head each
   screaming `(:wat::core::defn …)` → `(:wat::rete::core::defn …)` via a **recorded wat-fix codemod**
   at `wat-scripts/fixes/reheading-rete-callees.wat`, driven by an explicit LIST of fn names (a blind
   prefix rename would re-head the whole corpus — only the rete callees move). Dry-run on a `/tmp`
   copy and `diff` before applying. Follow `wat/fix.wat:23-53`'s stash-dance: this ships with a `src/`
   change that makes the old form illegal.

   **ENUMERATE EXTENSIONS — a `-name '*.wat'` glob is not the corpus.** R65/24t cost hours on exactly
   this: `.wat.bad` · `.wat.disabled` · `.wat.expr` · `.wat.intueri` all hold real forms, and a
   single-extension glob silently excluded 243 files / 23 name-holders last time. This strike ships
   its own `.wat.bad` (the gate above), so the trap is live from move one. And note the two surfaces
   a form-tree codemod structurally **cannot** reach: a keyword built or parsed inside a **string
   literal**, and inline wat inside **Rust test strings** — both have to be hand-checked, and the
   floor is what surfaces the second (2026-07-24's class-4 lesson).

## The RED gate — already on the disk, already green, and it is your acceptance test

**`tests/rete/probe_arc278_rete_defn_gap.{rs,wat.bad}` + `..._control.wat`** — committed and run
before this brief was finished (`cargo test --release --test rete -- rete_defn_gap` → 3 passed).
Do not rebuild it; inherit it.

What it already pins:

- **the gap** — the fixture's body is *already law-A clean* (`:wat::rete::core::i64::>` is a minted
  `RETE_OPS` row), so the only reason it fails is the unminted head. Today that surfaces as
  2 × `MalformedForm` — *":wat::core::i64 is a TYPE keyword, not a value"* — because an
  unrecognised head is treated as a **call** and its signature evaluated as **arguments**.
- **the non-vacuity control** — the sibling `.wat` is byte-identical minus that one form and MUST
  load. Without it the RED would prove "something in the file is bad", not "exactly this form is
  missing" (R59).
- **the acceptance criterion, written before the code** — `the_gap_diagnostic_does_not_name_the_helper`
  asserts that nothing in today's diagnostic mentions `:probe::declared`. **It is designed to FAIL
  when you land the strike**, and its panic message tells you exactly what to rewrite it into. That
  is the flip from *allowing* the diagnostic to improve to *forcing* it.

**Your job on this gate:** repoint the fixture's body at a NON-rete op so it exercises the membrane
rather than the missing form, and rewrite the third test to assert the helper **is** named. Then
prove it can still go RED by mutation (legal rete body → green; back → red). A gate that cannot be
shown red is a claim, not a proof — this arc shipped eleven such gates before (`91bbb8cd`).

One more ground fact, so you do not mis-read the failure: an arbitrary unknown head
(`:no::such::form`) gives `UnresolvedReference`, **not** the MalformedForm above. So
`:wat::rete::core::` is already handled specially — expect the form to need real registration, not
merely a name.

## Blast radius

`src/value/environment.rs` (+`ReteContract`), `src/rete/purity.rs` (one branch), the `defn`
registration path in `src/runtime.rs`, the 35 mechanical `Function { … }` sites, one new
`wat-scripts/fixes/*.wat`, one new `tests/rete/` gate, and the re-headed corpus files. No new types
beyond `ReteContract`. `RETE_OPS` is **not** touched.

## STOP triggers — each is a rejection, and you report rather than route around

1. **STOP-1 — the body check must call `purity.rs`'s existing walks.** If they are not callable from
   the definition-site path as they stand, STOP and report the signature that blocks it. Do not write
   a second implementation of any axis.
2. **STOP-2 — the name lands inside an admitted module.** `RETE_MODULES` contains
   `":wat::rete::core::"`, so `rete_vocabulary_admitted(":wat::rete::core::defn")` is already `true`,
   and `head_ok`'s admission branch will default-deny it (no `RETE_OPS` row, and `rete_op_for` is an
   exact match). That is believed benign — `defn` is top-level and never appears in expression
   position. **Confirm it by a run.** If a `defn` head can reach `head_ok`'s admission branch, STOP.
3. **STOP-3 — `defn` is NOT a `RETE_OPS` row.** That table is what may appear *inside* a predicate;
   its `params`/`ret`/`meta` columns are meaningless for a declaration. If the design seems to need a
   row, STOP.
4. **STOP-4 — the migration is a RE-HEADING.** Every screaming site should need only its head
   changed, because it was already law-A clean (that is *why* it was admitted). **If any site needs a
   BODY change, STOP and surface it** — the stone's "already admitted ⇒ already clean" reasoning has a
   hole, and that hole is a finding worth more than the migration.
5. **STOP-5 — a rete-defn stays callable from ordinary wat.** It is a fn carrying extra guarantees,
   like Postgres `IMMUTABLE`. If it starts becoming a separate callable namespace, STOP.

## The count

The stone says 27 sites. **I did not enumerate them and you should not grep for them.** The checker
enumerates once the membrane is armed (R52 `QVOD LEX ACCENDIT`, R65 `SCVTVM IDEM INDEX`) — a grep
over this corpus has produced a wrong count in this arc repeatedly. Report the number the checker
gives; if it is not 27, the checker is right.

## Shape to copy

`BRIEF-inline-constraint-admits-non-rete.md` + its codemod
`wat-scripts/fixes/inline-constraint-per-type-spelling.wat` — the nearest prior strike: same arc,
same fence, a `src/` check plus a recorded corpus migration.
