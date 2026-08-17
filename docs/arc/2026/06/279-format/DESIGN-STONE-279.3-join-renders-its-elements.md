# DESIGN STONE — 279.3 · `join` renders its elements (chain-D, the join half)

**Builder's ask, 2026-08-16:** *"we need string's join to to-str its elements to get something sane
like ruby's — `ary.join(',') => "some,stringified,values"`"* … and on where it lives: *"this is a wat
defn or not?.... we fully support parametric funcs."* **Yes — a wat defn.**

```
(:wat::core::defn :wat::core::string::join<T>
  [sep <- :wat::core::String
   xs  <- :wat::core::Vector<T>] -> :wat::core::String
  (:wat::core::string::join' sep
    (:wat::core::mapv (:wat::core::fn [x <- T] -> :wat::core::String (:wat::core::str x)) xs)))
```

## ★ THE WHOLE COMPOSITION IS ALREADY PROVEN GREEN — this is a move, not an invention

`wat-scripts/scratch-pad/probe-279.3-join-renders-its-elements.wat`, committed, type-checks, runs:

```
(:user::join-ish "," (Vector :i64 1 2 3))        →  "1,2,3"
(:user::join-ish "-" (Vector :String "a" "b"))   →  "a-b"
```

Four things that run proved, none of them assumed:

1. **`T` binds inside a lambda nested in a parametric defn.** This was the load-bearing unknown.
2. **`str` is total** (279.2, `25d9d015`) — an `i64` element renders with no bound on `T`.
3. ★ **A `String` element renders BARE** — `"a-b"`, not `"\"a\"-\"b\""`. `mapv` applies `str` at
   TOP LEVEL per element, so 279.2's *"nested strings stay quoted"* rule does not fire.
   **This answers the contract question off the disk instead of by ruling** — it is already Ruby's
   `ary.join(',')`.
4. Delegation to the existing native over `Vector<String>` composes.

## Why this is 279's stone and not a new arc

`DESIGN-STONE-279.2-str-totality.md` names this consumer by name:

> *"**The forcing consumer is `wat.string/join`.** `(join "," [1 2 3])` can only render its elements
> if `str` is total. With a partial `str`, `join` needs a **bound on a type variable** — a form wat
> does not have — or it is a `join` that cannot join numbers. Make `str` total and the bound stops
> existing: there is nothing left to constrain `T` by."*
>
> (Builder, 2026-08-14: *"its either everything must have a to-str call or we only accept strings —
> this mixed state shit is crazy."*)

279.2 made `str` total. **279.3 is 279.2 cashing its own stated cheque.**

## The defect today

`src/check.rs:16598` — the element type is hardcoded:

```rust
":wat::core::string::join" => TypeScheme {
    type_params: vec![],                                   // ← no type params
    params: vec![ string_ty(),
                  Parametric { head: "wat::core::Vector", args: vec![string_ty()] } ],  // ← Vector<String>
    ret: string_ty(),
}
```

So `(join "," [1 2 3])` does not type-check, and every caller must pre-stringify. The chain doc's
words: *"`join` today is `(sep: String, pieces: Value::Vec)` with every element required to already
be a String — `string_ops.rs:455`. **That signature is what D deletes.**"*

## The shape: wat surface, native primitive — the house pattern

The public `join<T>` becomes a **wat defn**; the existing Rust intrinsic is renamed `join'` and keeps
its `Vector<String>` signature. That is the `insert-all` / `insert-all'` convention already in the
tree: wat owns the generic part, the native owns the O(n) building.

**Why wat and not a generic intrinsic.** Both are small. The `TypeScheme` edit would have been
smaller. But:

- **It is a wat function and wat can express it** — parametric defns ship, `Fn(T)->U` ships, `mapv`
  is the exemplar. A stdlib that cannot write its own `join` is teaching something false.
- **Every builtin left as a Rust arm is one more for arc 255 to carve** (535 arms remain, 10 carved).
  Moving `join` out makes that pile smaller; teaching the checker a generic signature makes it bigger.
- **The perf objection was MEASURED AWAY, not waved away.** All 19 `join` call sites are
  compile-time/macro-time — `wat/core.wat` ×6, `bracket.wat` ×3, `Record.wat` ×2, `lint.wat`,
  `service.wat`, 4 codemods, 1 test. Separators are `"::"`, `"/"`, `"-"`, `" "`, `","`; inputs are
  namespace segments, **2–5 elements**. Zero calls in the rete engine, the trading substrate, or any
  per-element loop. *(The bytecode compiler is a second reason and deliberately NOT the first —
  "we'll throw the interpreter away eventually" would excuse anything slow, and it is not why this
  one is fine.)*
- **No oracle.** The `insert-all'` differential pattern exists and is the right tool *if* a
  measurement ever demands it. Building both now would be optimizing a path measured cold — the
  exact failure task #47 records twice.

## ⚠ THE LAMBDA IS FORCED, NOT STYLE — do not let anyone "simplify" it

`(mapv :wat::core::str xs)` **does not type-check**:

```
no clause of `:wat::core::mapv` matches arity 2 with types
  [:wat::core::keyword, :wat::core::Vector<wat::core::i64>]
```

A bare intrinsic keyword is a `keyword`; a **user** fn keyword is an `Fn(T)->U`. Same syntax, two
answers, depending only on whether the callee is written in wat or Rust. Filed as
`255/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md` — **arc 255's defect, not this stone's**, and
the first of that arc's consumers a user trips over in ordinary code.

The wat source must carry a comment pointing at that note, or a later reader will delete the lambda
and break the build.

## The four questions — flat

**Obvious? YES.** `(join "," [1 2 3])` → `"1,2,3"` is what the name has always promised.

**Simple? YES.** One wat defn; one native renamed; one hardcoded `TypeScheme` deleted. No new
mechanism — `mapv`, `fn`, `str`, parametric defns all ship.

**Honest? YES.** Today's signature claims to join a `Vector` and silently means `Vector<String>`.
After: it means what it says, and the element type is free because `str` is total.

**Good UX? YES.** Ruby's semantics, and the 19 existing call sites are unchanged — `Vector<String>`
unifies with `Vector<T>` at `T = String`.

## The gate

| # | assertion |
|---|---|
| 1 | `(join "," [1 2 3])` → `"1,2,3"` — the row that does not work today |
| 2 | ★ `(join "-" ["a" "b"])` → `"a-b"` — **strings render BARE**, the non-vacuity control and the Ruby contract |
| 3 | all **19** existing call sites unchanged and green (`core.wat` ×6, `bracket.wat` ×3, `Record.wat` ×2, `string.wat` ×2, 4 codemods, `lint.wat`, `service.wat`, 1 test) |
| 4 | `join<T>` is a **wat defn** in `wat/string.wat`; the native is `join'` |
| 5 | `check.rs`'s hardcoded `Vector<String>` scheme for the public name is **gone** |
| 6 | the lambda carries a comment pointing at `255/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md` |
| 7 | floor GREEN via `scripts/floor.sh` — the **Summary line** |
| 8 | `cargo clippy --release --all-targets` → **0** |
| 9 | `#[ignore]` count **13**, unmoved |

Row 2 is load-bearing: it is both the Ruby contract and the proof that `str`-per-element did not
start re-quoting strings, which would silently corrupt all 19 existing sites.

## Out of scope — affirmative cuts

- **`Seqable` as a nameable type.** The chain wrote D as one stone; the disk says it is two.
  `collection/infer.rs:638`: *"**This IS the `Seqable` set — the type wat cannot currently spell**"*,
  with **three named blockers, none small** (no `:nature` admits a builtin container; builtins
  satisfy no surface; wat has no ad-hoc unions, deliberately — R7). That is its own stone, and it
  also owns the twelve `<verb>-stream` twins. `join` over `Vector<T>` does not need it.
- **The `wat.string/*` rename.** That is chain-E. This stone keeps `:wat::core::string::join`.
- **Making intrinsics first-class values.** Arc 255. The lambda stands until then.

---

# ⛔ CORRECTION — 2026-08-17. `join` STAYS AN INTRINSIC. The wat-defn breaks stdlib bootstrap.

The stone above ruled a **wat defn**. A rider built it exactly as specified, hit **STOP-2**, and
stopped without chasing it. The premise was wrong and the disk says so.

## The break, verbatim

```
#wat.macro/ProgramBodyEvalFailed — macro :wat::core::defrecord — program body eval failed
  at wat/core.wat:1885
  cause: #wat.runtime/UnknownFunction "unknown function: :wat::core::string::join"
         at wat/Record.wat:172
```

`wat/core.wat:1885` self-invokes `(:wat::core::defrecord :wat::kernel::Location …)` **while
`core.wat` is still loading**. `defrecord`'s macro body computes a namespace prefix at
`wat/Record.wat:172` — `(:wat::core::string::join "::" ns-lead)` — and that runs at **expansion
time**, during the load. A wat-defn `join` does not exist yet.

## ★ MEASURED: it is not one file, and it is not fixable by reordering

Load positions in `src/stdlib.rs` of every `wat/` file that uses `join`:

```
stdlib.rs:40    wat/core.wat      ─┐
stdlib.rs:131   wat/Record.wat    ─┼─ THREE users load BEFORE string.wat
stdlib.rs:169   wat/bracket.wat   ─┘
stdlib.rs:278   wat/string.wat       ← where join<T> would live
stdlib.rs:326   wat/lint.wat
stdlib.rs:333   wat/service.wat
```

And `string.wat` **cannot move earlier**: it calls `:wat::core::defn` and `:wat::core::keyword`
(both `core.wat`) plus `mapv` (`seq.wat`).

## ★★ THE STRUCTURAL FACT — the intrinsic is a CYCLE-BREAKER

**`core.wat` ↔ `string.wat` is a genuine dependency cycle.** `core.wat`'s macro bodies need `join`;
`string.wat` needs `defn`. The graph is acyclic today **only because `join` is a Rust intrinsic** —
available from expression zero, before any wat exists.

That is not a preference about where code should live. It is a property of the substrate, and it is
**undeclared and unenforced** anywhere: nothing in `stdlib.rs`, nothing in the wat files, nothing in
the docs says that a verb consumed by an early macro body may not be defined in wat. Filed separately;
the next Rust→wat move rediscovers it at the cost of a rider flight.

## The four questions — run flat on all four options, 2026-08-17

- **A — generic intrinsic.** Obvious YES (one verb, one impl, available from expression zero, no
  load-order knowledge required) · Simple YES (two edits, no new concept, no second name) ·
  Honest YES (the signature says `Vector<T>` and means it; and it stops claiming wat can define what
  wat cannot) · UX YES (Ruby semantics, 19 sites unchanged, no choice to make). **ALL FOUR.**
- **B — two-tier (`join'` early, `join<T>` late).** Obvious NO (which verb you may call depends on
  your file's line number in `stdlib.rs`) · Simple NO (invents an unenforced rule) · **Honest NO** —
  presents `join<T>` as public while the stdlib's own early files cannot use it, and records that
  nowhere · UX NO.
- **C — reorder `string.wat` earlier.** All four NO, and moot: **impossible**, measured — it needs
  `defn` from `core.wat`.
- **D — hand-roll the ~11 early call sites.** Obvious NO · Simple NO · Honest NO (deletes real uses to
  route around an undeclared rule *without recording the rule*) · UX NO.

**Builder: "A has been reasoned."**

## The corrected stone

`join` **stays a Rust intrinsic** and becomes generic:

```rust
":wat::core::string::join" => TypeScheme {
    type_params: vec!["T".into()],                                        // ← was vec![]
    params: vec![ string_ty(),
                  TypeExpr::Parametric { head: "wat::core::Vector".into(),
                                         args: vec![TypeExpr::Path("T".into())] } ],
    ret: string_ty(),
    rest_param_type: None,
}
```

…and `eval_string_join` renders each element through the **total** `str` (279.2, `25d9d015`) instead
of demanding `Value::String`.

> ### ⊘ AMENDED 2026-08-17 — the snippet above was WRONG TWICE and would not compile
>
> As first written this section said `type_params: vec!["T"]` and `args: vec![TypeExpr::Var("T")]`.
> Both are wrong, and a rider copying them hits a type error before it reaches the real work:
>
> | as written | on disk | why |
> |---|---|---|
> | `vec!["T"]` | `vec!["T".into()]` | `type_params: Vec<String>` |
> | `TypeExpr::Var("T")` | `TypeExpr::Path("T".into())` | **`Var(u64)`** — `types.rs:92-98`: *"Fresh unification variable — synthetic, **NEVER produced by parsing**"*, allocated by `InferCtx`. A scheme's own type parameter is a `Path`: `types.rs:73-76` — *"Lexically-scoped type variables (`:T`, `:K`, `:V`) also appear as `Path` … the type checker distinguishes them via the enclosing scheme's `type_params`."* |
>
> Live exemplar to copy instead — `:wat::eval-ast!`, `check.rs:17058`, a `Path("T")` inside a
> `Parametric`'s args under `type_params: vec!["T".into()]`. That is this stone's exact shape.
>
> I wrote a Rust snippet into a stone without compiling it, one day after
> `[[feedback_a_rendered_example_is_not_a_measurement]]`. Kept visible rather than edited away.

## ★ THE LOAD-BEARING DETAIL — the door must carry the top-level-String-bare arm

`str`'s totality is **two arms**, not one (`runtime.rs:23359-23366`):

```rust
Value::String(s) => (**s).clone(),                                   // top level → BARE
other            => value_to_edn_string_with(other, sym.types()),    // everything else → encoder
```

Both halves are load-bearing and **each has already caused a shipped defect**:

- **Drop the String arm** → `(join "-" ["a" "b"])` renders `"\"a\"-\"b\""` and all 26 live call
  sites silently corrupt, because the encoder quotes strings.
- **Drop the registry argument** → records render `{:field-0 1}` instead of `{:x 1}`. That is
  literally the 296/279.2 fix whose comment sits above the line, and
  `[[feedback_a_totality_claim_is_only_as_good_as_its_sampling]]` is the twelve hours it cost.

So this stone does **not** hand-roll the match a second time inside `eval_string_join`. It mints
**one door** — `render_str_total(v, types)` — and makes `eval_str` its first caller and
`eval_string_join` its second. Two hand-rolls of a two-arm rule where the second arm is invisible at
the call site is exactly the shape `[[feedback_a_totality_claim_is_only_as_good_as_its_sampling]]`
was written about; `edn_shim.rs:3490` already states the discipline for the registry argument —
*"a caller with no registry passes `None` EXPLICITLY, in the open, where the next reader can ask why."*

## Two things measured while grounding this, neither of them in scope

1. **`render_unquoted`'s doc comment lies.** `string_ops.rs:510` says it is *"the `:wat::core::str`
   semantics"*. It is the **pre-279.2 partial** renderer — five arms, raises on the sixth — and `str`
   has not used it since `25d9d015`. Its one remaining caller is `interpolate` (`string_ops.rs:602`).
   One-line fix, in a file this stone already opens: in scope, as truth-in-comment.
2. **`interpolate` is still PARTIAL while `str` is TOTAL** — same file, one call site apart. So
   `(str <record>)` renders and `(interpolate "{x}" :x <record>)` raises. **Out of this stone's
   scope and NOT filed as a silent deferral:** widening it changes what the `format` macro does with
   a non-scalar, and the builder ruled `str` total — nobody ruled `format` total. It is a question
   for the builder, surfaced, not a task I may rule on.

## No rete mirror — measured

`grep -rn 'string::join' src/rete/` → **empty**. Chain-E's *"every string op has a paired
`:wat::rete::core::string::*` row"* does not hold for `join`; there is nothing to keep in sync, and
`vocabulary.rs`'s admission assert cannot fire on this change.

## The purity gate is settled precedent, not a new question

`join` is on `is_pure_total` (`macros/eval.rs:469`) — it **must** be, since `defrecord`'s macro body
calls it at expand time. `:wat::core::str` is on the same list (`eval.rs:632`) and **already** routes
through `value_to_edn_string_with`. Routing `join` the same way follows a verb already admitted on
that exact basis, and it makes `join` *more* total, never less: it deletes a raise.

## ⛔ THE CENSUS WAS WRONG — 19 was never the number

The stone above and the superseded brief both say **19** call sites with a per-file breakdown. Both
are wrong. Measured this session, command included so the number is reproducible:

```
grep -rn 'wat::core::string::join' --include=*.wat . | grep -v '^./target'   → 28
```

| where | n | |
|---|---|---|
| `wat/` | **13** | `core.wat` 5 · `bracket.wat` 3 · `Record.wat` 2 · `lint.wat` 1 · `service.wat` 1 · `string.wat` 1 |
| `tests/` | **7** | `collection/sort.wat` 4 · `comms/wat_pipe.wat` · `kernel/wat_string_ops.wat` · `macros/probe_arc249_4…` |
| `wat-scripts/` | **6** | `fixes/` 4 · `probes/arc-170/probe-m1-argcount` · `scratch-pad/probe-279.3` |
| `wat-tests/` | **1** | `service-cache-lru.wat` |
| `docs/` | **1** | `arc/2026/05/130-…/complected-2026-05-02/test.wat` — historical, not loaded |

**27 live, 26 of them pre-existing.** The old breakdown was wrong per-file too (`core.wat` ×6 → 5;
`string.wat` ×2 → 1). Fifth census error of the campaign, same mechanism every time: a count quoted
without the command that produced it. `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

**Everything the exemplar proved still holds and still decides the contract**: `str` is total, so `T`
needs no bound; a `String` element renders **bare**, so `(join "-" ["a" "b"])` → `"a-b"`, Ruby's
semantics, off the disk rather than by ruling.

## What this correction does NOT change

- **`join'` is not minted.** There is one `join`. The `insert-all` / `insert-all'` pattern is not used
  here; there is no wat surface to delegate from.
- **No `mapv`, no lambda, no arc-255 workaround.** The rendering happens in Rust, so the
  intrinsic-as-a-value asymmetry never arises in this stone. `255/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md`
  stands on its own — it was found here but does not depend on this stone.
- **The gate rows are unchanged.** Rows 1 and 2 are still the contract, and row 2 is still
  load-bearing.
- **The 255 cost is accepted and stated**: `join` remains one of the 535 dispatch arms to carve. That
  is real, small, and cheaper than a broken bootstrap. Trading a working substrate for a symbolic
  reduction in a future migration would be a manufactured prerequisite.

## What the rider did right, recorded because it is the reason this is correct

It followed the brief exactly, hit the wall, **did not revert**, **did not chase**, left the tree as a
live reproduction, and traced the cause to named source lines in three files plus the load order.
It also explicitly declined to fix `Record.wat` on its own authority — *"that is a design decision
outside this stone's authorized moves, not mine to make unilaterally."* Correct on every count. The
STOP is what produced the fact that settles a question I had oscillated on three times with bad
reasons on both sides.
