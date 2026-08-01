# DESIGN-STONE — a persistent structure built in a loop is a TRANSIENT; `x = x.push_back(v)` is the copy-per-element lie

> **Origin (2026-08-01).** Chasing the production pass, `out:production` measured **28.53 ms of a
> ~106 ms fire** — the single biggest item, larger than hash-join. The builder pushed on the seam's
> disposition of it: *"we mentioned prod:out is not deletable.. but not deletable ..... is != not
> optimizable?"* He was right, and the seam agreed with him in its own words (*"Optimising the
> materialisation is open; deleting it is not"*) — the apparatus had let **not deletable** stand in
> for **not worth looking at**, which is R24 `NON MVRVS SED VITIVM` at the level of a disposition
> rather than a number.

## The defect, read out of the library source (not asserted)

`rpds-1.2.1`, `src/vector/mod.rs`:

```rust
#[must_use]
pub fn push_back(&self, v: T) -> Vector<T, P> {
    let mut new_vector = self.clone();   // ← every node's refcount 1 -> 2
    new_vector.push_back_mut(v);
    new_vector
}

pub fn push_back_mut(&mut self, v: T) { … self.assoc(length, v) }

fn assoc(&mut self, index: usize, v: T) {
    SharedPointer::make_mut(&mut self.root).assoc(…)   // copies ONLY when shared
}
```

`make_mut` copies a node **only if its refcount is > 1**. So the two forms are not "slower vs
faster" — they are categorically different:

| form | refcount at `make_mut` | cost per element |
|---|---|---|
| `pv = pv.push_back(v)` | **2** (the `clone()` inside `push_back` made it so) | a full root→leaf **path copy**, then the old version is dropped unread |
| `pv.push_back_mut(v)` | **1** | in-place write; allocation only when a node genuinely fills |

**The `.clone()` inside the copying API is itself what forces the copy.** Building a fresh structure
nobody else holds, every one of those copies is waste — 39,999 intermediate vectors built and
discarded to produce one.

`HashTrieMap` has the identical pair (`insert` / `insert_mut`, `remove` / `remove_mut`), and
`FromIterator` for both already routes through `extend` → the `_mut` form.

**This is R8 in the output path.** R8 named the three points on the axis — immutable-by-copying
(`reduce { merge }`), immutable-by-structural-sharing, and **mutate-a-transient-then-freeze
(`each_with_object`)** — and called the third one *"the native kernel."* rpds's `*_mut` family **is**
that transient API. The kernel was running the Ruby anti-pattern R8 was written about.

## ★ THE ONE CONTRACT DECISION — the detector is SOUND, so the wall is exact

The anti-pattern has a syntactic signature that needs no case-by-case judgement:

```rust
x = x.<copying-method>(…)      // the SAME identifier on both sides
```

**A self-reassignment proves the old version is dead.** If the previous value were still needed it
could not have been overwritten — so the copy the form forces can never be observed by anyone. There
is no such thing as a legitimate instance of this shape. That makes the detector sound by
construction: no false positives to triage, no exemption class to argue.

The legitimate copying use binds a *different* name and does **not** match:

```rust
pm = pm.insert(k, PV(pv.push_back(fact.clone())))
//   ^^ outer: self-reassignment, a DEFECT       ^^ inner: `pv` is borrowed and still live — CORRECT
```

`kernel.rs:1558` is exactly that line, and it is the trap in this sweep: the outer `insert` converts,
the inner `push_back` must not. A transform that rewrites by method name rather than by the
LHS-equals-receiver shape will corrupt it.

## Measured — the golden exemplar, proven by hand before anything is armed

`hashmap_to_pm` (`kernel.rs:153`) is the whole of `out:production`: `out:alpha` and `out:beta` both
read 0.001 ms because those memories are cleared before freeze. Four lines changed to `push_back_mut`
/ `insert_mut`; fanout `[100 x 20]`, 40,000 derived facts, 3 runs per state:

```
                  baseline                       after                    delta
out:production    29.643 / 23.882 / 32.075       4.445 / 1.898 / 7.055    28.53 -> 4.47 ms  (-84.3%)
THE FIRE         104.534 / 102.617 / 110.128    90.846 / 84.429 / 82.178 105.76 -> 85.82 ms (-18.8%)
```

Ranges disjoint on both. Predicted 5–8 ms before the run; measured 4.47 — the mechanism accounts for
the number.

## Blast radius — 35 sites, 6 files (the sound detector's count, not the loose grep's)

```
src/rete/kernel.rs        18      src/edn_shim.rs            7  (3 in #[cfg(test)] bodies)
src/collection/eval.rs     5      src/rete/matcher.rs        3
src/collection/transform.rs 1     src/rete/collect.rs        1
```

Two are on measured hot paths beyond the exemplar:
- **`kernel.rs:905`** — `extend_token`'s bindings fold, inside `hj:catchup:probe` (**18.8 ms**,
  40,000 calls x 2 bindings). Note the first `insert_mut` there still copies once and *must*: the
  binding trie is cloned from a live `tok`, so its refcount is legitimately 2. `make_mut` handles
  that correctly — one copy instead of one-per-key.
- **`kernel.rs:244` / `:303` / `:406`** — the token/element materialisers.

## The stones

1. **The exemplar** — `hashmap_to_pm`. Measured above. **DONE.**
2. **The sweep** — the remaining 34 self-reassignment sites → the `_mut` twin. Mechanical, and
   rider-shaped, with two hard rules in the brief: convert **only** where the LHS identifier equals
   the receiver identifier, and **never** rewrite a nested call (`:1558`). Edit-only riders, no
   cargo; the orchestrator weighs centrally (FM 18).
3. **The wall** — `tests/lint/no_rpds_rebuild_loop.rs`, armed at zero, beside `no_rc_use` /
   `no_inlined_wat` / `no_loose_string_assert`. Same architecture: `collect_rs` walk, FAIL-list the
   offenders, floor-assert the file count so it cannot pass vacuously
   (`feedback_a_gate_that_discovers_beats_one_that_lists`), file-scoped
   `// rune:lint(no-rpds-rebuild-loop)` for a justified exemption — of which there should be
   **none**, since the shape is unconditionally wrong.

Prove red before green: the sweep must be able to go back to RED if reverted.

## Out of scope = REJECTED (affirmative cuts)

- **Changing any public shape.** The values produced are byte-identical persistent structures; this
  is a build-strategy change only, invisible above the function.
- **Replacing rpds, or adding a transient type of our own.** The `*_mut` family already is one.
- **`FromIterator`-ing the sites into `collect()`.** Tempting and often equivalent, but it changes
  more lines than it needs to and obscures the one-token diff the lint is going to police.
- **The `wm.beta` duplicate-storage question** (every join result cloned into a map read at three
  sites and cleared before freeze). Real, measured at `hj:catchup:emit` 7.2 ms, and **its own stone**
  — it is a *semantics* question (who reads beta, and when), not this one's build-strategy question.
  Tracked, not smuggled in.
