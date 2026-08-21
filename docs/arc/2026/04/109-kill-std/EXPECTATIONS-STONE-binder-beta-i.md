# EXPECTATIONS — arc 109, binder strike β-i

Written BEFORE the strike, against `c5ac5174c`.

| # | what | expected |
|---|---|---|
| 1 | `(:wat::core::defrecord :user::Box :- [T] [item <- T])` | expands and checks |
| 2 | ★★ **`T` is a VARIABLE** | `(:user::Box :item 42)` → `#user/Box {:item 42}` **AND** `:item "hi"` → `#user/Box {:item "hi"}` |
| 3 | ★ `(:wat::core::defrecord :user::Box<T> [item <- T])` still works | `#user/Box {:item 42}` — the additive control |
| 4 | `:wat::holon::defrecord`, both spellings | both work |
| 5 | the 4 parametric records in `wat/` still load | `Entry<K,V>` · `Cache::GetRequest<K>` · `Cache::PutRequest<K,V>` · `Alarm<O>` |
| 6 | a malformed `defrecord` still diagnoses clearly | a named missing-field-vector message, not a panic and not silence |
| 7 | floor | **0 FAIL** |
| 8 | clippy `-D warnings` | 0 |

Row 2 is the row that can pass hollowly, for the same reason it was in α: a binder that expands but
whose params never reach `type_params` leaves `T` a CONCRETE type, and the symptom
(`"expects :T; got …"`) reads like an ordinary type error while row 1 stays green. **Two different
value types through the same field, or it is not proven.**

## Independent prediction

**15–25 minutes.** The shape is copied from a sibling in the same repo that already works; the
novelty is only the emission splice.

## Trap-doors

1. **The kwargs companion.** `defrecord` mints a companion macro from the field vector. If the
   binder is accidentally swept into the field walk, the companion bakes a bogus field name and the
   failure surfaces far from the cause — at a `kwargs-construct` call site, not at the declaration.
2. **`~@binder` on an empty vector.** The non-binder path splices nothing; if that emits a stray nil
   or an empty node into the `recordtype` form, every existing `defrecord` breaks at once. Row 3 and
   the floor both cover it, but the FIRST symptom will look like an arity error from
   `parse_aggregate`.
3. **`:wat::holon::defrecord` forgotten.** It is a near-copy 99 lines below the base macro and has
   ZERO parametric call sites today, so nothing in the corpus would go red if it were skipped. Row 4
   is the only thing that catches it.
4. **Losing the arity diagnostic.** Removing the fixed-arity signature removes a real error message.
   Row 6 exists so it is replaced, not just deleted.

## Mode B

Any of: a non-binder `defrecord` expands differently · a `.wat` file other than `wat/Record.wat`
edited · a parametric call site migrated · the body counts `args` · cargo run by the rider.
