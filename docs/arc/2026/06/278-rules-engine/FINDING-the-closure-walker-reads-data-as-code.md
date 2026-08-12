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

## Disposition

- **The quote bug: RULED a bug, isolated rete-free, ready to strike.** `src/closure_extract.rs`,
  one missing case, plus a check on whether the three stem-patches survive it.
- **The unbound-`?` silence: NEW, filed, not this strike.** Fire-time behaviour unmeasured.
- Both probes are on disk under `wat-scripts/scratch-pad/`, loader-gated so they cannot rot into a
  graveyard that reads like live code.
