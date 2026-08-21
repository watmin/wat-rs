# EXPECTATIONS — arc 109: the param-spec consumption wall

Written BEFORE the strike, against `b1488d312`.

| # | what | expected |
|---|---|---|
| 1 | `(defrecord :user::R<T> [])` | **REJECTED**, naming `T` |
| 2 | `(defrecord :user::R<T> [x <- :wat::core::i64])` | **REJECTED**, naming `T` |
| 3 | `(defrecord :user::R<T,U> [x <- T])` | **REJECTED**, naming `U` |
| 4 | `(defenum :user::E<T> :wat::enum::Pure :A [f <- :wat::core::i64])` | **REJECTED**, naming `T` |
| 5 | ★★ **nested consumption is LEGAL** | `[x <- :wat::core::Vector<T>]` → clean |
| 6 | ★ direct consumption still legal | `[x <- T]` → clean |
| 7 | ★ monomorphic declarations untouched | no param-spec ⇒ no check |
| 8 | ★ `defn` with an unused `<T>` stays LEGAL | out of scope by ruling |
| 9 | floor | **0 FAIL** — the corpus scan predicts zero violations |
| 10 | clippy | 0 |

**Row 5 is the row that can fail silently-ish and the reason STOP-1 exists.** A wall that only
inspects each member's HEAD passes rows 1–4 and 6, and rejects row 5 — the exact form this arc is
introducing. Every other row would be green.

**Row 9 is a prediction, not a certainty.** The scan behind it is a regex that cannot tell code from
string literals; its one apparent hit was source text inside a migrator fixture. If the floor finds
a real violation, that is a finding about my instrument — report it, do not fix the declaration.

## Independent prediction

**15–25 minutes.** The walker exists and already handles nesting; the door exists and has three call
sites. The work is reaching each `TypeDef` variant's member types.

## Trap-doors

1. ★ **A second walker.** Writing a fresh recursion instead of reusing `collect_free_type_vars`
   reintroduces the nesting bug this stone is safe from. Verify by reading the diff, not the floor —
   a hand-rolled walker that happens to handle `Parametric` would pass every row while duplicating
   machinery the substrate already has (the fifth hand-roll of a question 251.8a collapsed into one
   door).
2. **A variant whose members are unreachable.** Six variants; `Surface` and `Union` are the least
   exercised and the most likely to be forgotten. Rows 1–4 touch only `Aggregate` and `Enum`.
3. **`Var(_)` in the walk.** `TypeExpr::Var` is synthetic and never produced by parsing source; it
   is correctly a no-op in the existing walker. A new check must not start treating it as a
   consumption.
4. **The empty case.** `type_params` empty ⇒ no check. A loop that runs anyway is harmless but a
   branch that errors on empty is not.

## Mode B

Any of: nested consumption rejected · a second walker written · `defn` signatures touched · a corpus
declaration edited to satisfy the wall · cargo run by the rider.
