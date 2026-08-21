# SCORE — arc 109 γ-i, flight 1: the rider was right and the BRIEF was wrong

**Mode B — the stone did not ship, and the defect was upstream.** The rider fired STOP-2 and STOP-3
and shipped nothing complete. Both STOPs were correct. Both were caused by an acceptance row I wrote.

Every row below was re-run by the orchestrator against the disk. Nothing is credited from the report.

## The scorecard, independently re-run

| # | row | result |
|---|---|---|
| 1 | `defn` takes the binder | ✅ |
| 2a-2e | the non-vacuity gate (see below) | ✅ **all five** |
| ~~2~~ | ~~let-bound, applied at two types~~ | ⛔ — **the row was wrong**, see below |
| 3 | both spellings → contradiction error | ⛔ silently checks |
| 4 | 251.7 does not regress | ✅ |
| 5 | HOF control undisturbed | ✅ |
| 6 | parametric kwargs `defn` | ⛔ *"triple is incomplete"* |
| 7 | variadic `defn` | ✅ |
| 8 | `def` untouched · `check.rs` diff EMPTY | ✅ |

## ⛔ THE BRIEF'S DEFECT — row 2 fired the neighbouring mechanism

Row 2 demanded ONE let-bound value apply at TWO different types. That is **let-polymorphism**, not
"an anonymous fn can declare its type params." The rider's analysis was exactly right and I verified
all three of its citations verbatim:

```rust
check.rs:2065   WatAST::Symbol(…) => match locals.get(…).cloned() { Some(ty) => ok(ty), …   // clone, no re-instantiation
check.rs:11757  let rhs_ty = infer(rhs, …)                                                   // one infer, a TYPE not a scheme
check.rs:15977  func.name.as_ref()?;   // "Fns (name = None) … aren't statically typed here" // the scheme door, shut by design
```

So row 2 genuinely required `check.rs`, STOP-2 fired correctly — **and the stone it killed was
already delivered.** The builder settled it by writing the two forms out:

```clojure
(wat.core/fn [a :- i64 x :- i64] :- i64 …)      ;; implicit :- []   ✅ checks
(wat.core/fn :- [X] [a :- X b :- X] :- X …)     ;; explicit :- [X]  ✅ checks
```

★ **My justification for the row — *"one instantiation proves nothing; a rigid `:T` passes a single
application"* — was a correct concern behind a wrong vehicle.** A rigid `:T` and a missing
let-generalization fail that row IDENTICALLY, so it could never report which one it had found. The
question *"is `X` a real variable?"* is answered by whether `X` unifies ACROSS POSITIONS, and that
needs no let-polymorphism at all:

```
(f 1 2)              ✅ checks
(f 1 "s")            ⛔ REJECTS   ← X unifies across positions: not a wildcard
(f "p" "q")          ✅ checks    ← X is not pinned by the first use: not rigid
(takes-str (f 1 2))  ⛔ REJECTS   ← the return is tied to X
```

`[[feedback_a_gate_must_fire_the_mechanism_the_way_production_fires_it]]`

## Two more places the BRIEF was wrong, both caught by the rider

1. **`wat/core.wat` needed no edit for FORWARDING.** I invented that sketch item. Macroexpanded
   against the pre-flight binary: `(defn :user::f :- [T] …)` → `(def :user/f (fn :- [T] …))` — the
   binder already lands inside the `fn`. ★ But the rider ALSO wrote *"it may still be needed for the
   `name-tp`-derived `Kwargs<T,U>` naming, which I did not test."* **It is.** Row 6 fails in binder
   spelling and passes in angle spelling with a byte-identical argspec. The rider's hedge was more
   accurate than my deletion of the item.
2. **The silent-accept is far wider than reported.** The rider found a `:-` binder silently accepted
   on an anonymous fn. Measured on the RELEASE binary from before the flight, with a control:

   ```
   first slot of (fn ??? [x <- :i64] -> :i64 x), then applied to a String:
     (nothing)  ✅ rejected   ← control: the instrument works
     :- [T]     ⛔ ACCEPTED       :foo  ⛔ ACCEPTED       42  ⛔ ACCEPTED       "s"  ⛔ ACCEPTED
   ```

   **Any** stray token makes the whole fn unconstrained and every call to it check vacuously.
   Pre-existing on `main`, reachable by a typo, silent. Filed as its own stone.

## What the rider did well

Read `check.rs` exhaustively and returned a mechanism, not an opinion — three citations, all of which
survived independent verification. Honoured STOP-1 (never touched the `[WatAST; 3]` wall). Reported
row 3 as *"silently unions — my stopgap does not raise a diagnostic"* rather than claiming it. Flagged
its own untested hedge on `core.wat`, which turned out to be the thing my brief got wrong. Refused to
claim the stdlib half was verified. And when the harness blocked its revert, it said so plainly
instead of narrating a clean tree.

## Disposition

Flight 1's five modified files are KEPT as flight 2's base — every row they deliver is verified green
above. Flight 2 closes rows 3 and 6. G3 stands; `check.rs` is empty and the blast radius holds for
the actual feature. **~14 min, well under the 35-60 prediction — because the rider stopped the moment
it had proof, which is the behaviour the STOP triggers exist to buy.**
