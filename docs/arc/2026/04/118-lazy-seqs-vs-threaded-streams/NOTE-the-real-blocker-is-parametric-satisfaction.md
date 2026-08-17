# ⛔ NOTE — the real `Seqable` blocker is PARAMETRIC SATISFACTION, and it is none of the three on record

**Measured 2026-08-17, by probe. This note CORRECTS its own sibling** —
`NOTE-the-blockers-were-stale-seqable-is-spellable.md` and commit `0548f4f9`, which claimed *"the
FULL chain-D Seqable design type-checks TODAY."* **That claim was declaration-only and I stated it
as though the design worked.**

## The two probes, and the exact line between them

| probe | surface | call sites | result |
|---|---|---|---|
| `probe-seqable-is-spellable-today.wat` | `:sq::Seqable` — **non-parametric** | yes, 2 builtins | ✅ **RUNS**, prints `"3,4"` |
| `probe-seqable-parametric-all-four.wat` | `:sq::Seqable<T>` — **parametric** | **none** — declarations only | ✅ `--check` exit 0 |
| the same + four call sites | `:sq::Seqable<T>` | yes, 4 builtins | ⛔ **RED, 4 errors** |

Verbatim, all four identical in shape:

```
#wat.check/TypeMismatch {
  :message ":sq::count-of: parameter #1 expects :sq::Seqable<?454>;
            got :wat::core::Vector<wat::core::i64>"
  :callee ":sq::count-of"  :param "#1"
  :expected ":sq::Seqable<?454>"  :got ":wat::core::Vector<wat::core::i64>"
  :remedies []
}
```

`?454` is a fresh unification variable — the surface's type parameter never binds.

## ★ What this isolates

**A concrete builtin does not unify against a PARAMETRIC surface parameter.** The delta between the
two probes is *parametricity alone*:

- surfaces — **fine** (the non-parametric probe runs)
- `extend-type` on builtins — **fine** (four of them declare cleanly)
- builtins satisfying a surface — **fine** (the non-parametric probe dispatches by runtime type)
- a `defn` whose param type is a surface — **fine** (non-parametric)
- **a `defn` whose param type is `Surface<T>`, called with `Builtin<Concrete>` — BROKEN**

So the three blockers at `src/collection/infer.rs:638` are still stale — they name the wrong
things, and its sibling note stands on that. But **`Seqable` is not free**, and the thing standing
in the way was never written down anywhere.

## Why this does not sink the stone — and may not even block `join`

Chain-D's own text says the payoff of stone C is that **`T` is unconstrained**:

> *"`T` is **unconstrained**, and that is the payoff of C: with a total `str` there is nothing left
> to constrain it by. Ruby's `join` needs no bound for the same reason."*

`join` renders every element through the now-total `str`. **It never needs to know `T`.** So the
first consumer may not need a parametric surface at all — and the non-parametric form is *proven to
run end to end*. That is a real fork, and it is the builder's:

- **(a) non-parametric `Seqable`** — proven working today; enough for `join`; loses element typing
  for consumers that DO need `T` (`map`'s `fn(T)->U` is the obvious one).
- **(b) fix parametric satisfaction in the checker** — the honest general fix, unblocks `map` /
  `filter` / the seven `-stream` twins as well, and is a genuine type-system stone of unknown size.

## ⚠ Still unmeasured, and unrelated to the above

- **Per-element dispatch cost.** `join`/`map`/`filter` walk every element; a surface dispatch per
  element is a real perf question and no probe here speaks to it.
- **Can a Rust intrinsic's `TypeScheme` name a wat-defined surface?** `join` must stay an intrinsic
  (`docs/NOTE-the-stdlib-bootstrap-cycle-intrinsics-break.md`). Measured: **zero** existing schemes
  in `check.rs` name any wat-defined type (checked `Dialable`, `TypedCapability`, `Cache` — all 0),
  so there is no precedent to lean on. Unprobed; it decides whether `join` itself can take the
  surface, or whether the first consumer must be a wat-defined verb.

## The lesson, and it is the same one twice in one hour

A `--check` that passes on a file which **never invokes the thing** proves the declarations are
well-formed and nothing else. `[[feedback_a_green_test_can_prove_nothing]]` — *"my EXIT=0 defined a
macro and never called it."* Same shape, same day: I declared a generic fn over a surface, never
called it, read exit 0, and reported the design worked.

The correction cost one probe. The claim had already reached a commit message, a probe header, and
the builder.
