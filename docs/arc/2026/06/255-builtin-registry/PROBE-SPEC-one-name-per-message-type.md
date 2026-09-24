# PROBE SPEC — can generated code name a message type without the per-op alias?

**For:** `DESIGN-one-name-per-message-type.md`. **Run before the design is drawn as a stone.**
Per `examinare`: *a ten-line probe that attempts exactly the assumption, failing on exactly the gap.*

## The assumption under test

> Generated service code can annotate an operation's message type **without** `<S>::<op>/Request`,
> using only the required name — either by omitting the annotation, or by a form resolved after
> registration — **and still get the non-uniform argument mapping right.**

## The discriminating case — it must be non-uniform

A uniform mapping proves nothing (the required name plus the surface's params would already work). Use the
measured non-uniform shape, `wat/cache.wat:177`:

```wat
(:wat::core::defsurface :probe::C :- [K V] :nature :wat::kernel::Peer
  :messages [ … (defenum :probe::C::GetResponse … :Hit [value <- :V] …) ;; takes [V] only
              … (defrecord :probe::C::PutRequest [k <- :K  v <- :V]) ]    ;; takes [K V]
  :features [(get [self <- (:probe::C :- [K V]) req <- …] -> (:probe::C::GetResponse :- [V]) …)
             (put [self <- (:probe::C :- [K V]) req <- (:probe::C::PutRequest :- [K V])] -> … )])
```

## Rows — write each by hand, exactly as `defservice` would emit it at `wat/service.wat:2045`

| # | client-method return annotation | purpose | expected |
|---|---|---|---|
| 0 | `(:probe::C::get/Response :- [K V])` — the alias, today's form | **control — must pass** | rc=0 |
| 1 | *omitted* | candidate 1 — does wat accept it, and infer `(GetResponse :- [V])`? | **the question** |
| 2 | `(:probe::C::GetResponse :- [V])` — required name, correct args | the target shape — resolves, but the macro cannot compute `[V]` at step 4 | rc=0 (proves the target is well-formed) |
| 3 | `(:probe::C::GetResponse :- [K V])` — required name, all params | ⛔ **negative control** — wrong arity must be REFUSED | rc=1 |
| 4 | row 1's form, where the body returns a `PutResponse` | ⛔ **negative control** — if row 1 passes, it must not pass *vacuously* | rc=1 |

## What each outcome means

- **Row 1 passes and row 4 refuses** → candidate 1 is viable: draw the stone as *omit the annotation*.
- **Row 1 refuses because an annotation is mandatory there** → candidate 1 is dead; probe candidate 2
  (a type-level projection) next, as its own spec.
- **Row 1 passes and row 4 also passes** → the inference is unsound or absent; row 1's pass means nothing.
- **Row 0 fails** → the probe is wrong, not the design. Fix the probe.
- **Row 3 passes** → the required name accepts an arity it does not declare — a separate defect; report it.

## Discipline

- ⭐ **Prove the probe can say both words:** row 0 must be rc=0 and row 3 rc=1 *before* reading rows 1, 2, 4.
- Strip ANSI (`sed 's/\x1b\[[0-9;]*m//g'`) on any live output you match against.
- `--check` for type questions; **run** anything whose claim is about runtime behaviour.
- Scratch `.wat` goes in `wat-scripts/scratch-pad/` only if it must load clean; negative rows live in the
  session scratchpad or as `.wat.bad` fixtures.
