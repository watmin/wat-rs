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
