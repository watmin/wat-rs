# STONE O — one declaration feeds both doors

> Builder, 2026-08-28: *"draw the apply stone - one declaration feeding both doors"* — and, restating
> the thesis so it could not be lost: *"Two calling conventions are forced by the language. Two
> registrations are not."*

> ## ⛔⛔ CORRECTED 2026-08-28, BEFORE ANY STRIKE — READ THIS BEFORE THE BODY BELOW
>
> The builder refused this design's framing:
>
> > *"ok but like…. `(+) => 0` / `(+ 1) => 1` / `(+ 1 1) => 2` / `(+ 1 1 1) => 3` right?…..
> > this "+ needs two args" is baffling"*
>
> **He was right, and running the question properly found a THIRD defect bigger than either
> below.** Two things this design got wrong:
>
> **1. `:wat::core::+` is already Clojure-compliant. Measured:**
> `(+)`→0 · `(+ 1)`→1 · `(+ 1 1)`→2 · `(+ 1 1 1)`→3 · `(*)`→1 · `(- 5)`→-5 · `(- 10 1 2)`→7.
> Identity element, unary, variadic left-fold — all correct. What the arity probe quoted was
> **`:wat::i64::+`, a defclause LEAF that is arity-2 by design**. Presenting a leaf's contract as
> the language's made a correct design read as a broken one, and put a wrong baseline in the
> acceptance rows. `[[feedback_an_adjacent_implementation_is_not_the_subject]]`
>
> **2. There is a THIRD broken door, and it is the whole user-facing surface.**
> `:wat::core::apply` **cannot apply a defclause at all**:
> ```
> (:wat::core::apply :wat::core::+ [1 2 3])  →  err ":wat::core::apply: expected wat::core::keyword,
>                                                     got wat::core::clauses <clauses::wat::core::+/25>"
> ```
> **29 defclauses exist, 22 of them production** — `+ - * / reduce sort sort-by into filterv mod
> quot rem run! reductions nth-spec` — and **not one can be applied**. `dispatch_keyword_head`
> HAS a clauses arm (`runtime.rs:6758` → `eval_call_to_defclause`); `eval_apply`'s Step 6 demands
> `Value::wat__core__keyword` and stops at a `Value::wat__core__clauses`. `(apply reduce …)` and
> `(apply sort …)` — the reason `apply` exists — are refused.
>
> **So `apply` has FOUR doors and THREE are broken.** Probe:
> `wat-scripts/scratch-pad/255-stone-o-apply-has-three-broken-doors.wat`
> ```
> DOOR 1  defclause head                     REFUSED "expected keyword, got clauses"   22 production verbs
> DOOR 2  registered intrinsic, no value door "unknown function"                       337 of 380
> DOOR 3  registered intrinsic, value door    works — and wrong arity PANICS            44
> DOOR 4  plain fn / defn                     correct                                   —
> ```
>
> ★ **The root the body below names is RIGHT and it is now proven three times, not two:** a second
> dispatch path reimplementing the first from a private picture, and every time the thing it cannot
> express is *"I hold Values, not ASTs."* Door 1 is that impedance mismatch one layer up — the
> clauses arm needs an entry that takes `&[Value]`, exactly as the intrinsic arm did.
>
> **The strike order below is superseded by "The four strikes" section.** The one-declaration
> machine — the builder's stone, and still the thesis — is no longer first, because a live panic and
> the entire user-facing arithmetic surface both outrank it and neither depends on it.
>
> ⚠ **A separate compliance finding, surfaced by the same probe, NOT drawn here:**
> `(:wat::core::/ 1 2)` → `0` and `(:wat::core::/ 4)` → `0`, where Clojure gives `1/2` and `1/4`.
> `(:wat::core::/ 4.0)` → `0.25` is right. wat HAS rationals (`:wat::rational::`, 4 registered
> arms), so the type to return exists. Changing integer `/` to produce a rational changes a return
> type across the corpus — **it is the builder's ruling, not this design's**, and it belongs to
> road step 5 (EDN/Clojure compliance), not to arc 255.

> Drawn against `9b25f3bbf`. Every number below was produced by an instrument printed here.

## ⛔ THE DEFECT — `apply` ANSWERS FROM ITS OWN PICTURE, NOT THE REGISTRY

`:wat::core::apply` reports **"unknown function"** for verbs the registry knows perfectly well. Not a
subset of exotic ones — most of them. Measured live against `target/release/wat` at `9b25f3bbf`, by
the disconfirming probe committed with this design
(`wat-scripts/scratch-pad/255-stone-o-apply-lies-about-what-exists.wat`):

```
:wat::i64::+       [HASVAL]  DIRECT=ok:42                    APPLY=ok:42
:wat::f64::max-of  [no val]  DIRECT=ok:Some [41.0]           APPLY=err:unknown function: :wat::f64::max-of
:wat::string::to-uppercase   DIRECT=ok:"WAT"                 APPLY=err:unknown function: :wat::string::to-uppercase
:wat::vector::length         DIRECT=ok:3                     APPLY=err:unknown function: :wat::vector::length
:wat::math::sqrt             DIRECT=ok:4.0                   APPLY=err:unknown function: :wat::math::sqrt
```

Four registered, working verbs reported ABSENT. The one that answers is the one carrying a
`value = <path>` slot from Stone N. **44 of 380 registered names carry that slot** — so `apply` can
reach 11.5% of the language and calls the other 88.5% nonexistent.

★ **This is `walk.rs:268` wearing the opposite mask.** The blanket-accept says YES to everything
including what does not exist; `apply` says NO to what does. Both are a dispatch path answering from
a private picture instead of from the registry, and arc 255 exists to make the registry the sole
authority for what exists. A registry that two dispatch paths disagree about is not an authority.

⚠ **And it bites hardest exactly where splat matters.** `max-of` is VARIADIC. The verb that most
wants `apply` is one `apply` cannot see.


## ⛔ THE SECOND DEFECT, FOUND WHILE DRAWING THIS — THE VALUE DOOR HAS NO ARITY CHECK, AND PANICS

The 44 verbs `apply` *can* reach are not safe either. **Wrong arity through the value door is a Rust
panic, not an error.** Measured at `9b25f3bbf`, with the AST door as the control — same verb, same
wrong arity, two doors:

```
(:wat::i64::+ 20)                                        →  err: ":wat::i64::+: expected 2 args, got 1"   ← AST door
(:wat::core::apply :wat::i64::+ [20])                    →  PANIC  runtime.rs:11605  "arity-checked"      ← value door
(:wat::core::apply :wat::vector::concat [one-vector])    →  PANIC  vector.rs:214     "arity-checked"
```

`arith_i64_i64_inner` and every hand-written value twin open with the same two lines:

```rust
let a = vals.first().expect("arity-checked");
let b = vals.get(1).expect("arity-checked");
```

**`.expect("arity-checked")` names a check that happens on the OTHER door.** The generated shim
arity-checks the AST path (`wat_intrinsic.rs:545`, `ArityMismatch`); `dispatch_substrate_impl` calls
`handler(vals)` with nothing in between. Censused: **25 unchecked-index sites across 5 intrinsic
files, plus the shared `arith_*_inner` fns — and NO value handler anywhere checks `vals.len()`.**
So the panic is reachable for **all 44**, from ordinary wat source, with no unsafe, no FFI, no
misuse — just an `apply` with the wrong number of arguments.

★ **Both defects have ONE root: the value door was bolted on BESIDE the AST door instead of being
generated WITH it.** It inherited neither the registration (so `apply` cannot see 337 verbs) nor the
arity check (so the 44 it can see panic). One declaration feeding both doors is not tidiness — it is
the only shape in which the second door cannot be born missing what the first one has.

`[[feedback_a_slot_with_two_implementations_is_two_slots]]` — this is that lesson's sharpest
instance yet: the two implementations diverged on a SAFETY property, silently, and the comment on
each one asserted the property it did not have.
## ⛔ THE ONE CONTRACT DECISION — THE DECLARATION IS THE ALGEBRA, AND THE MACRO GENERATES THE SHELL

Today a verb that wants both doors is written twice:

```rust
#[wat_intrinsic(":wat::f64::+", value = eval_f64_add_value)]   // door 1 names door 2
pub(crate) fn eval_f64_add(a: &WatAST, b: &WatAST, env: …, sym: …, span: …)
    -> Result<Value, EvalBreak> { … }                          // AST door
fn eval_f64_add_value(vals: &[Value]) -> Result<Value, EvalBreak> { … }   // value door
```

Two fns, two registrations, one verb. **After this stone the ALGEBRA is the declaration and the AST
door is GENERATED:**

```rust
#[wat_intrinsic(":wat::vector::length")]
fn vector_length(v: &Value) -> Result<Value, EvalBreak> { … }   // ONE fn. BOTH doors.
```

The macro already sniffs the argument shape (`sniff_args`: `&WatAST` ⇒ Exact, `&[WatAST]` ⇒ Variadic)
and the return shape (`sniff_return`, Stone G). **This stone adds the third sniff on the same
mechanism — the LEADING PARAM TYPE decides the kind:**

| leading params | kind | doors generated | `apply` reaches it |
|---|---|---|---|
| `&Value` × N, or `&[Value]` | **ALGEBRA** | value door = the fn itself; AST door = generated shell | **YES** |
| `&WatAST` × N, or `&[WatAST]` | **BINDING** | AST door only | no — and it SAYS so (O-ii) |

The generated AST shell is exactly the two lines every shell handler writes by hand today —
`eval_inner` each arg, call the fn:

```rust
fn __wat_intrinsic_shim_vector_length(args, list_span, env, sym) -> Result<TrackedValue, EvalBreak> {
    if args.len() != 1 { /* ArityMismatch, as today */ }
    let a0 = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    vector_length(&a0).map(TrackedValue::from)
}
```


★ **AND THE ARITY CHECK IS GENERATED ONCE, FOR BOTH DOORS.** The value adapter the macro emits opens
with the same `ArityMismatch` the AST shim already raises (`wat_intrinsic.rs:545`) — same error kind,
same `op` name, same shape — so a wrong-arity `apply` returns the error the direct call returns
instead of killing the process. `.expect("arity-checked")` stops being a claim about somewhere else
and becomes true where it is written. **This is the rung above the fix**: the panic is not patched at
25 sites, it is made unreachable, because a hand-written value door — the only thing that can be born
without a check — no longer exists for these verbs.

**Two calling conventions survive — they are forced by the language.** `apply`'s arguments have no
syntax: `(apply :wat::i64::+ (:mk::pair))` evaluates to 42 while the form's AST children are
`[apply, the verb, (:mk::pair)]`, with no node for `20` or `22` anywhere. The arity is decided at
runtime; there is no AST to hand a `NativeHandler`. **What does not survive is two REGISTRATIONS.**

## ★ WHY THIS IS SAFE WHERE STONE N SAID "DELIBERATELY NOT MERGED" — read this before doubting it

`i64.rs`'s Stone N comment refuses exactly this merge, and it is RIGHT about what it refuses:

> *"`apply` hands already-evaluated `Value`s with no arg-level `Span`s, so it goes through
> `arith_i64_i64_inner` … error spans are synthesized there, real argument spans here — a
> pre-existing difference this stone does not change … deliberately NOT merged into
> `eval_i64_arith`/`i64_add_op` above, which would drop apply's ability to ever gain real spans."*

That rationale is about the **19 arithmetic verbs** whose native path runs `eval_i64_arith` /
`eval_f64_arith` — fns that hold real per-argument spans. **It does not describe the shell
population**, and the disk says so plainly. A shell already delegates to a span-free value fn:

```rust
pub(crate) fn eval_persistentvector_length_home(v: &WatAST, env, sym, _span) -> Result<Value, EvalBreak> {
    let v = eval_inner(v, env, sym)?.value_owned();
    crate::collection::eval::persistentvector_length_inner(&v)     // ← takes &Value. No span.
}
pub(crate) fn persistentvector_length_inner(v: &Value) -> Result<Value, EvalBreak> {
    match v { … other => Err(RuntimeError::new(crate::rust_caller_span!(), TypeMismatch{…})) }
}
```

`rust_caller_span!()` is synthesized on **both** doors. Arg-eval failures still raise inside
`eval_inner`, at the argument's own real span, before the algebra is ever called. **For the shell
population the two doors already share one implementation and one span behaviour** — the second
registration buys nothing and costs a lie. Stone N's caution stands, unamended, for its 19.

`[[feedback_read_the_epitaph_before_you_build_on_prior_art]]` — the epitaph was read; it scopes
itself, and this stone stays outside that scope.

## The population — measured, with the instrument

A handler is a **SHELL** iff, after deleting its argument-eval calls and its comments, its body names
neither `env` nor `sym`: it is *(eval each arg) → value-fn*, so one declaration can generate both
doors. The classifier lives at `wat-scripts/hunt/stone-o-shell-census.awk`; it excludes the
SIGNATURE (which always names `env`/`sym` — the shim forces that) and is controlled both ways.

```
                        no value door    has value door     total
SHELL  (collapsible)         112               25            137
BINDING (AST door only)      224               19            243
                                                             380  ✓ THE TOTAL — see the correction below
```

- **112** verbs gain `apply` for the first time — minus **1** (`eval_holon_from_holon`, the single
  shell that returns `TrackedValue`; see the contract cut below) = **111**.
- **25** verbs written today as TWO fns collapse to ONE. This is the builder's *"two registrations"*,
  literally.
- **243** stay BINDING and honestly cannot be splatted — they need `env`/`sym`. **They are the reason
  O-ii exists**: after this stone `apply` still cannot serve them, and it must say the true thing.
- After O-i + O-iii, `apply` reaches **155 of 380**, and every one of the remaining 225 gets an
  honest diagnostic instead of a lie.

⛔ **CORRECTED 2026-08-28 by the O-iii rider — 380 IS THE TOTAL, AND THE CLASSIFIER WAS RIGHT.**
This section used to read *"380, not 381 — the classifier reads one fewer than the registry does…
O-i's row 0 is to name that one handler."* **There is no such handler.** The `381` was my own
baseline grep counting a DOC COMMENT as a registration: `src/intrinsic/holon/mod.rs:9` reads
``//! `#[wat_intrinsic(":wat::holon::…")]` handlers under the SAME names, here.`` — prose about a
migration, using `…` as a placeholder — and the `grep -v '<fqdn>'` filter knew only ONE spelling of
"this is a placeholder" and let the other through. **Counting comments as code, for the third time
in one day, in the very instrument I had written to make a number outlive itself.** The rider
verified the correction three ways: the name-diff isolates exactly one spurious entry; per-file
attribute counts match the awk's per-file rows with zero discrepancy anywhere; and an
attribute-ANCHORED grep returns 380 with a list identical to the awk's, name for name.

> **The honest one-liner anchors to attribute POSITION instead of matching text anywhere in a file:**
> ```bash
> grep -rhoP '^\s*#\[wat_intrinsic\(\s*"\K[^"]+' src/ --include=*.rs | sort -u | wc -l   # 380
> ```
> `[[feedback_a_file_count_is_not_an_item_count]]` · `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## The contract's affirmative cuts

- **An ALGEBRA fn returns bare `Result<Value, EvalBreak>`. A provenance-stamping handler
  (`-> Result<TrackedValue, _>`, Stone G) is BINDING by construction**, and the macro rejects the
  combination with a `compile_error!` naming the reason. It is not dropped silently and it is not
  half-supported: `ValueHandler` returns a bare `Value`, `eval_apply` sits below the provenance
  boundary (`runtime.rs:5360` is the `map(TrackedValue::value_owned)` discard, and the comment above it records why), so an
  ALGEBRA-with-provenance declaration would promise a stamp that the value door cannot carry.
  One shell is affected — `eval_holon_from_holon` — and it stays a hand-written BINDING handler.
- **The 19 arithmetic pairs keep their two implementations** — their span divergence is real,
  pre-existing, and Stone N's rationale governs it. **But they do NOT keep the panic.** They are
  the verbs the arity probe actually killed the process on, so O-ii gives `dispatch_substrate_impl`
  the arity check the registry can derive from `IntrinsicEntry::arity` — one guard in front of every
  value handler, generated or hand-written, so no door is reachable without one. Their span
  behaviour is out of Stone O's scope; not tracked elsewhere because nothing is broken there — both
  doors answer, and after O-ii both refuse the same wrong arity the same way.
- **`ValueHandler` is NOT widened to `TrackedValue`.** Doing so would lift the provenance boundary
  through `eval_apply` and every `dispatch_substrate_impl` caller — a different stone about
  provenance, not about the registry being one authority. Out of Stone O's scope; not tracked
  elsewhere because no consumer is served worse than today by leaving it.
- **`walk.rs:268` is untouched.** Stone O makes the registry answer `apply` honestly; it does not
  make the resolver consult it. That is the campaign's endgame and it is sized at 2,539 tests.

## The four strikes — REORDERED 2026-08-28 by what the correction found

| | strike | what it delivers | size |
|---|---|---|---|
| **O-i** | **the guard** | `dispatch_substrate_impl` arity-checks against `IntrinsicEntry::arity` before calling ANY value handler. Kills the panic for all 44, generated and hand-written alike, in ONE place. | one function, ~10 lines |
| **O-ii** | **the defclause door** | `eval_apply` accepts a `Value::wat__core__clauses` head and routes it into the clause dispatcher through a value-level entry — `(apply + …)`, `(apply reduce …)`, `(apply sort …)` start working. 22 production verbs. | one arm + one value-level entry beside `eval_call_to_defclause` |
| **O-iii** | **the machine** — *the builder's stone, and still the thesis* | the ALGEBRA sniff: one declaration generates both doors behind one arity check. Proven on `:wat::vector::` (6 verbs; 5 gain a door, 1 collapses two fns into one). | the macro + one namespace |
| **O-iv** | **the migration + the honest word** | the remaining 130 (105 new doors, 25 collapses), one commit per namespace; and `eval_apply` consults `lookup_entry` so anything still unreachable is told the truth instead of `unknown function`. | a sweep |

**Why this order, and it is a recommendation the builder rules on:**

- **O-i is a live panic.** `(apply :wat::i64::+ [20])` kills the process today, from ordinary wat
  source. Nothing depends on it and it does not wait on the machine. *We do not leave known flaws.*
- **O-ii is the user-facing surface.** `apply` refusing `+`, `reduce` and `sort` is the gap a person
  writing wat actually hits; doors 2 and 3 are leaves they rarely name directly. It is also a
  DIFFERENT mechanism from O-iii — the clauses arm, not the intrinsic registry — so ordering it
  early costs the machine nothing.
- **O-iii still carries the thesis** and is unchanged in shape: *two calling conventions are forced
  by the language, two registrations are not.* It moves to third because two defects outrank it and
  neither is blocked on it — not because the stone got smaller.
- **O-iv last**, because a mis-migrated verb should land in a world where the arity is guarded (O-i)
  and the diagnostic is honest, not in one where it panics silently.

⚠ **Each strike is independently green and independently useful.** No strike below is a prerequisite
for the one above it, so the builder can take them in any order without stranding work.


## O-iv DECOMPOSED — 2026-08-28, after O-iii proved the machine

O-i, O-ii and O-iii shipped. What the table above called "O-iv — the migration + the honest word"
is two independent kinds of work, and the sweep half decomposes cleanly by namespace. Measured with
`wat-scripts/hunt/stone-o-shell-census.awk` at `dd5494256`:

| | strike | population | why it is its own strike |
|---|---|---|---|
| **O-iv-a** | **the honest word** | 331 verbs get a true diagnostic | not a migration at all — one enum variant, one `Display` arm, one raise site. Its truth does not depend on how far the sweep gets, and it makes every later wave's residue legible instead of a lie. |
| **O-iv-b** | **the collections** | `map` 8 · `hashmap` 8 · `vec` 7 · `linkedlist` 5 · `hashset` 4 = **32** | the SAME shape O-iii already proved on `vector`, in sibling files. Highest-confidence wave; finishes the collection family. |
| **O-iv-c** | **holon** | `atom` 41 · `subspace` 10 · `engram` 10 · `reckoner` 8 · `hologram` 4 = **73** | over half the remaining population, one coherent domain, and it carries the ONE exception: `eval_holon_from_holon` returns `TrackedValue` and therefore stays BINDING by the contract cut above. |
| **O-iv-d** | **the remainder** | `uuid` 7 · `kernel/ambient` 7 · `string` 2 · `reflect` 2 · `bytes` 2 · `witness`/`time`/`regex`/`math`/`list`/`char` 1 each = **26** | scattered singles; last because a wave of one-offs is where a generator's edge cases surface, and by then the generator has 105 verbs of evidence behind it. |

**The honest word's ONE CONTRACT DECISION — a new `RuntimeErrorKind` variant, not a reused one.**

`MalformedForm { head, reason }` is the tempting reuse: `eval_apply`'s Step 7 already rejects special
forms with it. **Rejected on question 1.** The form is not malformed — a reader who sees
`MalformedForm` goes looking for a syntax error, and there isn't one. The verb is registered, the
call is well-formed, and the only true statement is *this verb has no value-level door*.

⚠ **And the diagnostic is PERMANENT, not transitional.** 243 handlers are BINDING: they need
`env`/`sym`, so they can never be splatted, and no amount of sweeping changes that. `apply` holding
`Value`s cannot manufacture the ASTs a BINDING handler consumes — and a literal AST rebuilt from an
already-evaluated value would be wrong for anything carrying identity. So this is not a message that
disappears when the sweep finishes; it is the language telling the truth about a real boundary, and
it deserves its own variant.

★ **The ripple was MEASURED, not estimated** — a throwaway variant was inserted, the workspace built,
and the compiler asked: **ONE** non-exhaustive-match site, `src/value/signal.rs:584` (the `Display`
impl). Reverted. `[[feedback_impose_the_check_and_read_the_screams]]` The EDN rendering needs no arm
at all — `RuntimeErrorKind` carries `#[derive(wat_edn::ToEdn)]` (`signal.rs:189`), so the wire form
is generated.
## The four questions

- **Obvious? YES.** One declaration, one implementation, and the signature's leading param says which
  kind it is. A reader of `fn vector_length(v: &Value)` knows it is algebra without being told.
- **Simple? YES.** No new registry field, no new table, no new lifecycle. A third sniff on a macro
  that already sniffs two axes, and a generated shim beside the one it already generates.
- **Honest? YES**, and it is the point. Today `apply` says a registered verb does not exist. After
  O-i/O-iii most of them work; after O-ii the rest are named truthfully. The wrong answer stops
  having a form: an ALGEBRA declaration cannot produce only one door.
- **Good UX? YES.** `(apply :wat::f64::max-of …)` — splat over a variadic verb — starts working, and
  a verb author writes ONE fn instead of two-plus-a-cross-reference-comment.

## Rooms

```
crates/wat-macros/src/wat_intrinsic.rs:102   sniff_args      — the arg-shape sniff to extend
crates/wat-macros/src/wat_intrinsic.rs:181   sniff_return    — Stone G's precedent for a second axis
crates/wat-macros/src/wat_intrinsic.rs:531   the shim body   — where the generated AST door is built
crates/wat-macros/src/wat_intrinsic.rs:371   value_handler_field — the `value = <path>` slot to retire
src/intrinsic/mod.rs:162,198                 NativeHandler / ValueHandler — the two conventions
src/intrinsic/vector.rs                      O-i's proof namespace (6 verbs)
src/collection/eval.rs:829                   persistentvector_length_inner — the value-fn that exists
src/runtime.rs:11561                         dispatch_substrate_impl — seven lines; unchanged by O-i
src/runtime.rs:10749                         eval_apply step (c)/(d) — O-ii's room
```

## The finding this design surfaced and is NOT chasing

While building the probe: `(:wat::core::PersistentVector :wat::core::i64 1 2 3)` is **refused by the
static checker** (`MalformedForm`, Doctrine 1 arc 242 — *"a TYPE keyword, not a value"*) and
**accepted by the runtime constructor through `:wat::eval-ast!`**, which counts the type keyword as
an element and yields length 4. Checker and runtime disagree about the same form. Recorded here
because it was measured here; it is a checker/runtime divergence, not a registry defect, and folding
it into Stone O would braid two concerns. It needs its own draw.
