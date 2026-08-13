# NOTE — `resolve::walk` skips the FIRST TWO ARMS of every `match`

**PROVEN by run 2026-08-12. NOT fixed — task #90.** Filed with its reproduction inline because the
probe MUST FAIL to be meaningful, so it cannot live under `wat-scripts/` (the
`every_wat_scripts_file_loads` gate would go red), and a `.wat.bad` with no paired `.rs` is an inert
file nobody runs.

## The defect

`src/resolve/walk.rs:158` iterates a `match` form's arms with `items.iter().skip(4)`, on the doc's
assumption that `items[2..=3]` is a `-> :T` return ascription. **That ascription was annihilated**
(arc 258.5 / R54, commit `1212c9ae`). `src/resolve/normalize.rs::normalize_match` was migrated for
it and walks `items[2..]`; `walk.rs` was not.

So the call-head resolution pass never walks the bodies of a `match`'s **first two arms**.

## The run

Two **identical** `:nosuchns::vanished` calls, differing only in which arm they sit in:

```clojure
(:wat::core::defenum :probe::E :wat::enum::Pure
  :A
  :B
  :C
  :D)

(:wat::core::defn :probe::first-arm [e <- :probe::E] -> :wat::core::i64
  (:wat::core::match e
    ((:probe::E::A) (:nosuchns::vanished 1))     ;; line 18 — NOT reported
    ((:probe::E::B) 2)
    ((:probe::E::C) 3)
    ((:probe::E::D) 4)))

(:wat::core::defn :probe::third-arm [e <- :probe::E] -> :wat::core::i64
  (:wat::core::match e
    ((:probe::E::A) 1)
    ((:probe::E::B) 2)
    ((:probe::E::C) (:nosuchns::vanished 3))     ;; line 28 — reported
    ((:probe::E::D) 4)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
```

`target/release/wat --check <file>` → `WAT_EXIT=1`:

```
#wat.resolve/UnresolvedReferences {:message "1 unresolved reference" … :unresolved
  [#wat.resolve/UnresolvedReference {:path ":nosuchns::vanished"
     :context "call head — not a builtin, not a registered function"
     :span … :line 28 …}]}
```

**One** reference reported, at **line 28**. The identical call at line 18 is invisible.

The non-vacuity control is the construction itself: same head, same shape, same file, one reported
and one not. The only variable is arm position.

## Blast radius

**817** `:wat::core::match ` forms across `wat/` + `wat-tests/` + `wat-scripts/` (comment lines
excluded). Every one has two arms whose call heads are never resolve-validated.

The failure mode is a **degradation, not a silence**: an unresolvable head in arm 1 or 2 defers from
a located resolve-time `UnresolvedReference` to a runtime `UnknownFunction`
(`reference_check_is_not_a_complete_red_arbiter`).

## Why it matters beyond the bug

`src/resolve/boundary.rs` exists to stop `walk` and `normalize` drifting on the boundary-head set,
and its own doc records the payoff: both passes match on `Boundary` **exhaustively**. But it also
says *"The traversal itself stays in each pass"* — and **the traversals drifted anyway**, on the very
variant the door names. Unifying the *classification* did not prevent drift; the door held the label
and let the walk rot.

That is load-bearing evidence for the boundary work: a boundary must carry the **traversal shape**,
not just the label, or a future retirement updates one consumer and misses the other — which is
exactly what R54's `-> :T` annihilation did here.

## The fix, and how NOT to do it

**Do not hand-sync the two arms.** That is the convention rung of the ladder and it is precisely what
just failed. The fix belongs inside a boundary stone where both passes derive their traversal from
one declared shape.

**Gate:** the probe above must report **TWO** unresolved references, at both line 18 and line 28.

## Cross-references

- `src/resolve/walk.rs:158` — the stale `skip(4)`; `:149-150` the doc that explains the assumption
- `src/resolve/normalize.rs::normalize_match` — the migrated twin, walking `items[2..]`
- `1212c9ae` — R54's `-> :T` annihilation, which updated one consumer and not the other
- Task **#90**
