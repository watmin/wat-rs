# DESIGN — the blanket dies, and it dies in THREE

## The census, re-derived 2026-09-09 on a quiescent tree

```
143 of 845 files refuse · 35 distinct names
```

Identical to the arc's own 2026-09-07 census, two days and seven stones later. **The number held** —
the first settled figure this week that did not expire.

## ⛔ The 35 are not one problem. They are three, and each has a different fix.

### ① The rete vocabulary — 11 names, ~950 occurrences, 87% of the volume

`:wat::rete::i64::{+,-,*,/,rem,mod}` · `:wat::rete::f64::{*,/}` · `:wat::rete::vector::get` ·
`:wat::rete::string::subs` · `:wat::rete::holon::cosine` · `:wat::rete::core::PersistentVector/first`

They live inside `(:wat::rete::where …)` bodies, which `Boundary::MakeRule` deliberately treats as
**code** while the rest of `make-rule` is data.

★ **The arc already measured this and wrote the epitaph** —
`NOTE-rete-ops-is-a-population-missing-from-the-registry-not-an-authority-over-it.md`:

```
RETE_OPS rows ......................... 74   (Alias 35 · Fallback 20 · Redispatch 10 · Form 9)
rows carrying an OpMeta literal ....... 74
rete_name REGISTERED as an intrinsic ... 0 / 74
```

> *"A rete row is a DIFFERENT VERB from its core twin, usually a TOTALIZED one.
> `:wat::rete::i64::/` is TOTAL — `(… 1 0 :undefined -1)` returns `-1` — where core `/` is PARTIAL."*

**Disposition: REGISTER the vocabulary.** These are real verbs with real metadata that were never
rows. The table already carries everything a row needs.

### ② Type arguments walked as call heads — 4 names

`:wat::type::{Tuple,i64,String,Vector}`, from `(:wat::core::HashSet :- [:wat::type::Infer] …)` and
kin. `:wat::type::Infer` is a **live** type-position marker (`src/types.rs:55`).

`resolve/walk.rs` already knows this hazard — its own comment says a type in this position is
*"reported as `call head — not a builtin, not a registered function`, naming the type as if it were a
missing function"*, and it handles `is_binder_marker` for one spelling.

**Disposition: a BOUNDARY fix, not a registration.** A `:- [...]` type-argument list is DATA. Names
in it are types, and registering type names as callables would be the wrong cure for the right
symptom.

### ③ Real verbs that were never registered — 4 names

`:wat::eval-ast!` (337) · `:wat::eval-with-defs!` (6) · `:wat::string::=` (2) ·
`:wat::core::stream->pvec` (5). Each has live dispatch arms in `src/runtime.rs` and zero registry
rows — arc 255's original "68 scheme-only verbs" class, nearly exhausted.

**Disposition: REGISTER.**

## The order, and why

```
① rete vocabulary     87% of the volume, evidence already written, no design left to do
② the :- boundary     4 names, and registering them would be actively WRONG
③ the last four       smallest, and independent of both
then                  DELETE THE BLANKET — `resolve` asks `registry().lookup_entry`
then                  the DOT FLIP unblocks
```

⚠ **Do not delete the blanket first.** Measured on clean main, 2026-09-09:

```
(:wat::core::Option.Some {:value 7})   check=0   →   #wat.core/Option.None {}
```

The spelling the migration moves TOWARD is still silently mis-read as a keyword-accessor miss. The
blanket is what lets that head through. Flip the notation before the blanket dies and every
partially-migrated head becomes a silent `None` instead of an error.

## What must NOT happen

⛔ **Registering names to make the census green.** ① and ③ are registrations *because the verbs are
real and the evidence exists*, not because a number needs to move. ② is the proof that
disposition-by-count is wrong: four of the thirty-five must NOT be registered, and only reading them
tells you which.
