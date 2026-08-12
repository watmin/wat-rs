# FINDING — the closure walker reads DATA as CODE (it descends into `quote`)

**Arc 278, 2026-08-12. Builder-ruled a bug: *"a walker that resolves symbols inside a quote is
misreading data as code — that's a bug."*** Measured, isolated rete-free, and the fix is smaller
and sharper than the one this note's predecessor proposed. **Supersedes the diagnosis in
`FINDING-fn-forms-cannot-walk-a-rete-dsl-body.md`** — that note's *symptom* is real, its *mechanism*
was one layer short.

## The defect, isolated with NO DSL anywhere near it

`wat-scripts/scratch-pad/probe-arc278-fnforms-walks-into-quoted-data.wat` — two arms, differing in
exactly one thing:

```clojure
(:wat::core::defn :probe::clean [] -> :wat::core::i64 42)                 ; no quote
(:wat::core::defn :probe::quoted-junk [] -> :wat::WatAST
  (:wat::core::quote (mystery-symbol another-nonexistent-name)))          ; quoted data
```

Both shipped through the same `fn-forms` call. Result:

```
CONTROL closure forms=1
SUBJECT #wat.runtime/MalformedForm {:message "malformed :wat::kernel::fn-forms form:
  …probe-arc278-fnforms-walks-into-quoted-data.wat:29:23:
  free symbol `mystery-symbol` does not resolve to a parent define or substrate primitive" …}
```

`:29:23` is **inside the `(:wat::core::quote …)`**. `mystery-symbol` is a plain bare Symbol,
deliberately **not** `?`-prefixed, appearing nowhere else in the file. **The walker descends into
quoted data and demands it resolve.** No rete, no pattern variables, no DSL.

## Why the rete symptom was a red herring

`defrule` expands to a zero-arg defn that **quotes** its `:when`/`:then` (`wat/rete.wat:2385-2400`):

```clojure
(:wat::core::defn :usr::hot-rule [] -> :wat::rete::Rule
  (:wat::rete::make-rule "usr::hot-rule" (:wat::core::quote [<cond1> …]) …))
```

So `?c` is quoted data, and the walker raised on it for the same reason it raises on
`mystery-symbol`. **Nothing about rete, pattern variables, or the `?` spelling is load-bearing** —
which is exactly the builder's requirement: *"this must be agnostic to rete — any user defined dsl
must be tolerable."* A fix keyed on `?` would be the rete-specific hardcode being ruled out.

## ★ THE FIX IS A DOOR THAT ALREADY EXISTS — the walker is a THIRD derivation that bypasses it

`grep -n quote src/closure_extract.rs` → **zero hits.** The walker has no concept of `quote`.

But `src/resolve/boundary.rs` does, and its own doc claims the title:

```rust
/// This is the ONE place the boundary-head set is encoded. Both the call-head
/// resolution walk and the symbol-ref normalization pass route through it.
pub(crate) fn quote_boundary(head: &str) -> Boundary {
    ":wat::core::quote" | ":wat::core::forms" | ":wat::core::define" | ":wat::holon::literal" => AllData,
    ":wat::core::quasiquote" => Quasiquote,   // data EXCEPT unquote escapes, which are code
    ":wat::form::matches?"   => MatchesSubject,
    ":wat::core::match"      => Match,
    ":wat::rete::make-rule"  => MakeRule,
    _ => Ordinary,
}
```

Two passes route through it. **`closure_extract.rs` is a third derivation of the same question that
does not** — it carries its own ad-hoc head list (`let`/`fn`/`match`/`defstruct`/`structtype`/
`defenum`/`defmacro`) and knows no data boundary at all. This arc's signature defect once more: N
derivations of one question, and the bug lives in the newest, weakest one — the same shape as the
five registries and `109/NOTE-two-resolvers-over-the-five-registries.md`.

**The door already did the thinking a hand-rolled fix would have had to redo**, and would likely
have gotten wrong. Its `MakeRule` doc:

> *"`make-rule`, not `defrule`: a census of rule producers (defrule's template, sift-rules-defsvc's
> generator, hand-built rule literals, and direct make-rule calls) found **make-rule is the one door
> all four funnel through**; hooking `defrule` alone would silently miss the other three."*

— and it specifies that `:when` is data **except** each `(:wat::rete::where …)` body, which is code.

So the fix is **route the walker through `quote_boundary`**, not "add a quote arm". That is what
makes it agnostic *structurally*: the walker stops having an opinion about which forms are data and
defers to the one place that owns the question. It also closes `:wat::core::forms` — separately
flagged as an invisible class in `FINDING-fn-forms-cannot-walk-a-rete-dsl-body.md`'s table.

⚠ **Blast radius, stated honestly:** adopting `Match` / `MatchesSubject` / `MakeRule` changes walker
behaviour BEYOND quote — it will stop collecting from positions it currently walks. Most of that is
the point (a pattern is not a reference); some could drop a dependency that was being picked up
incidentally. The floor arbitrates; the union-closure gate and `probe_arc272_6b` are the near gates.

**SCOPE IS A RULING** — (i) minimal: `AllData` + `Quasiquote` only, smallest diff, but leaves the
walker a FOURTH partial derivation; (ii) full: route every head, delete the arms the door subsumes.
The recommendation on the disk is (ii): a partial adoption re-creates exactly the condition that
produced this bug.

The evidence this cut is at the right depth is the **three special cases already in the file**, each
added because the walker misread a binding position as a reference:

- `:865` — *"struct and enum forms also require special handling… their field/variant names are
  BINDING positions, not references — the plain-list recursive path would incorrectly treat them as
  free symbols, **causing UnresolvedSymbol failures at extraction time**"* (arc 170 Gap H)
- `:1044` — *"field names are binding positions… would be **misclassified as UnresolvedSymbol**"*
- plus the `let` / `fn` binder handling

Those are patches on the stem: one form learned per arc, a list that can never close in a language
with user macros. **`quote` is the substrate's own "this is data" marker** — honouring it makes the
walker agnostic *by construction*, with no list to maintain, for every DSL that quotes its forms.
Whether those three special cases become unnecessary afterwards is worth checking during the strike
(they may be reachable through non-quoted paths); if they are, deleting them is how the fix pays
for itself rather than becoming a fourth mechanism beside three.

## ⚠ THE OTHER MEASUREMENT REFUTED THE CLAIM IT WAS TESTING

The objection to answer before touching a guard was *"then who catches a genuinely broken rule?"*
The builder's answer: *"our rete solution will run compile and it will raise if compile faults and
the user is given a detailed message on the mistake."* **Measured, and it does not.**

`wat-scripts/scratch-pad/probe-arc278-who-diagnoses-a-bad-rule.wat` — a control rule and a broken
one, identical but for a single unbound variable (`?missing` consumed in `:then`, never bound in
`:when`):

```
"CONTROL rule built"
"CONTROL compiled OK — the well-formed rule passes its own gate"
"BROKEN rule built — now compiling it"
"BROKEN COMPILED WITHOUT RAISING — the DSL did NOT diagnose the unbound variable; the claim is REFUTED"
```

**`:wat::rete::compile` accepts a rule whose RHS references an unbound variable, silently.**

This does **not** weaken the quote fix — it strengthens it. The walker's raise was never a
rule-correctness guard: it fires on `?c` (**valid**) and would fire identically on `?missing`
(**invalid**). It cannot tell them apart, because it does not know what a rule is. It is not a
guard; it is noise that happens to be loud. Removing it from quoted data loses nothing that was
ever protecting anyone.

**But it surfaces a NEW defect, and a familiar one.** An unbound `?` variable in a `:then` compiles
clean — the sibling of **#63** (*"a `:then` kwargs item may under-supply fields and SILENTLY
construct a corrupt record"*, closed) and squarely in **#80**'s territory (*"EVERY rete row must be
TOTAL"*). Filed as its own item; it is not this strike's job.

**BOUNDED HONESTLY — what I did NOT measure:** whether *firing* the broken rule raises, or produces
a corrupt derived fact. I measured **compile only**. "Compiles silently but fails loudly at fire" and
"never diagnosed at all" are materially different dispositions and I will not assert which one this
is. That measurement is owed before the new defect gets a fix.

## ✅ FIXED (builder ruled scope **(ii)**) — the walker routes through the door

`closure_extract.rs`'s `walk_free_symbols` now dispatches every list head through
`quote_boundary`, exhaustively:

| boundary | traversal |
|---|---|
| `AllData` (`quote`/`forms`/`define`/`holon::literal`) | return — nothing inside is a reference |
| `Quasiquote` | descend the template, walk **only** unquote escapes (real deps live there) |
| `Match` | **not the door's** — answered by the binder arm; `match` on an enum is core to the language and this walker needs its arm patterns' binders |
| `MatchesSubject` · `MakeRule` | ⛔ **REFUSED — see below** |
| `Ordinary` | plain recursion, unchanged |

### ⛔ THE DOOR ITSELF CARRIES LIBRARY PRIVILEGE — refused, not inherited

The builder caught this on review: *"i saw a `MakeRule` in the closure extract… this means we cannot
support user defined dsls… we cannot make ourselves special."* He is right, and the first cut of
this fix made it worse by spreading the privilege to a **second** consumer.

`quote_boundary`'s entries are two different kinds:

- **the language declaring its own grammar** — `quote`, `quasiquote`, `match`, `define`, `forms`.
  Legitimate; every compiler knows its own special forms.
- **a LIBRARY's grammar inside the compiler** — `:wat::rete::make-rule` → `MakeRule`, plus
  `is_where_form()`, an entire function for one library form (`:wat::rete::where`). rete got to
  edit the compiler's list; **a user's DSL cannot.** That is precisely what makes a user DSL
  second-class. (`:wat::holon::literal` also sits in `AllData`, but holon-rs is slated to merge INTO
  wat-rs and stop being a dep — builder's call — so it is not the same class.)

So this walker honours the language facts and **refuses the library ones**. `walk_make_rule_when`
is deleted; the `is_where_form` import is gone. The `MatchesSubject | MakeRule => {}` arm is a
**self-deleting marker**: it exists only because those variants exist, and the exhaustive match
makes it a compile error the moment they die. It cannot outlive the defect it marks.

**MEASURED — refusing them cost nothing on the raise path.** `fn-forms` over a rule fn still returns
`closure forms=5` with no `MakeRule` handling at all, because `make-rule`'s `:when` **is itself a
`quote` form** — plain recursion meets it and the `AllData` arm stops there. The special case was
never buying raise-avoidance; `quote` already did. It bought only dep-collection inside `where`
bodies, so refusing it trades a false POSITIVE (a refused valid program, unfixable by the user) for
a false NEGATIVE (a missing dep the child names at startup, legible and located).

**The cure, for the door's own ruling:** the language already has the universal mechanism —
`quote` for data, `quasiquote`/`unquote` for data-with-code-holes. Every library entry exists
because that form did **not** use it. rete's `:when` expressed as a quasiquote with `where` bodies
as unquote escapes needs **zero** compiler knowledge, and `quote_boundary` shrinks to core forms.

**One honest refinement to the ruling.** (ii) said "delete the arms the door subsumes." Grounding
found the existing arms answer a **different question** — `let`/`fn`/`match`/`defstruct`/`defenum`
are about *what is in scope* (binders), not *what is data*. The door subsumes none of them, so the
diff is ADDITIVE where I had said it would be subtractive. `match` was briefly routed through the
door and then backed out: it is core to the language and the walker already knew it.

`is_unquote_escape` widened `pub(super)` → `pub(crate)`. Its own doc calls the escape set *"a
language fact, encoded here exactly once so the two descents cannot drift"* — there are three
descents now, and a copy would have broken the property that doc protects.

**MEASURED:**

- the RED probe flipped — both arms extract; **it is now a standing REGRESSION gate**, and its
  verdict line was rewritten, because it still read *"the claim is REFUTED"* (true when reaching
  that line WAS the disconfirmation, a lie the moment the fix landed).
- **`fn-forms` over a rule fn: `closure forms=5`.** This is the specific blocker that reverted the
  child-entry strike, and it is measured rather than inferred — the floor could not have shown it
  either way, since the floor was green with the strike reverted.
- floor **4391/4391 passed, 0 failed, 262 skipped** — baseline count, no regressions.

**STILL NOT CLAIMED:** that the child-entry strike now passes. Its other reds (the rotted
`wat-scripts` consumer of `service-forms`; the UNCHARACTERIZED `dead_child_speaks` arm) are
untouched by this fix, and the strike remains reverted. Re-attempting it is a separate act.

## Disposition

- **The quote bug: RULED a bug, isolated rete-free, ready to strike.** `src/closure_extract.rs`,
  one missing case, plus a check on whether the three stem-patches survive it.
- **The unbound-`?` silence: NEW, filed, not this strike.** Fire-time behaviour unmeasured.
- Both probes are on disk under `wat-scripts/scratch-pad/`, loader-gated so they cannot rot into a
  graveyard that reads like live code.
