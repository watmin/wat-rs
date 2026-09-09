# DESIGN — the summary counts with a set

Removes the harness's own O(N²) term **before** the n=4000 measurement it would otherwise contaminate.

## The site the tracker has carried, relocated and confirmed

`:fanout::summarize`, `circuit.wat:2092-2106`:

```wat
id-map (:wat::core::foldl … (:wat::hashmap::assoc acc (:fanout::key-of o) true) … outs)
w-map  (:wat::core::foldl … (:wat::hashmap::assoc acc (:fanout::Outcome/worker o) true) … outs)
distinct (:wat::core::count (:wat::hashmap::keys id-map))
dup      (:wat::core::- total distinct)
```

`summarize` runs **once**, but it folds over **every outcome** — 8000 at n=2000 — and
`:wat::hashmap::assoc` clones the whole map per insert (`src/collection/eval.rs:367`). `id-map` grows to
8000, so this is **O(N²)** in the harness.

★ The tracker has carried this as `circuit.wat:2075`; the line moved to `:2096` as the file grew. It is
the same defect and it is still live.

⚠ **`w-map` is not the same problem.** It keys on worker id and reaches ~8–12 entries, so its clone cost
is trivial. It is changed here for consistency, not for cost — and this DESIGN says so rather than
claiming a win it does not have.

## ★★ A set wearing a map's clothes

`id-map` is `(HashMap :- [String bool])` whose **only** use is `count (keys id-map)`. Every value is
`true`. That is a **set**, modelled as a map-to-true because no persistent set existed when it was
written.

`:wat::set::` now exists (`2fdcfefaa`), so:

```wat
id-set   (:wat::core::foldl … (:wat::set::conj acc (:fanout::key-of o)) … outs)
distinct (:wat::set::length id-set)
```

`HashTrieSetSync::insert` is `O(log n)` and shares. The `keys` allocation disappears too — `length` reads
the size directly.

This is the type's **second production caller**, after `seen-ids` (`0e026d2de`).

## ⛔ Why this must precede the n=4000 run

The next measurement asks whether the **system** is superlinear. If the **harness** carries its own
superlinear term, the answer is contaminated — the measurer inside the measurement, which is the exact
class this arc has been extirpating all session.

Rough size: `total` minus the sum of the phase timers is ~560 ms at n=2000, and `summarize` is part of
that unaccounted remainder. At n=4000 an O(N²) term is ~4× that. **Small now, and growing precisely as
the axis under study.**

## ★ How I nearly skipped this

The census gave me the six cloning sites and their enclosing functions. Four are `:user::` scenario
deftests with tiny inputs; two are in `summarize`, which is called once. From that I concluded *"not one
is per-message"* — and was about to move on. Wrong: **called once and iterating N are different things**,
and only reading the body shows which. The enclosing-function name was a proxy, and proxies have cost
this arc four stones today.

## The one contract decision

**`distinct` and `dup` keep their exact meanings and values.** `distinct` is the count of unique
`key-of` values; `dup` is `total - distinct`. This is a data-structure change with **zero** semantic
change, and rows 1–2 gate that.

## OUT OF SCOPE — REJECTED

- **The four `:user::` scenario deftests** (`:2644 :2732 :2836 :2893`). Each folds over a handful of
  outcomes; the cloning cost is unmeasurable there. Removing the verb from the file entirely would be
  tidy, and tidy is not a reason. **Named, not scheduled.**
- **`wat/`'s cloning sites** — `span.wat` 8, `service.wat` 12, `deporder.wat` 2, `core.wat` 2. Stdlib,
  the builder's call.
- **Any change to admission, the cap, the topic, or the queue.** This is the harness only.

## Files

`wat-scripts/fanout/circuit.wat` only — `:fanout::summarize`.
