# DESIGN-STONE — `insert-all` is the primitive; `insert` becomes varargs over it

> **Origin (builder, 2026-07-31): "both, mirror clara — insert varargs + insert-all."** Ruled after
> the measurement below, and after reading Clara's own source: we did not choose a one-fact-at-a-time
> UX, we shipped the degenerate case of a door the reference engine has as its *primitive*.

## The finding — we built the special case and called it the API

```clojure
;; clara-rules/src/main/clojure/clara/rules.cljc:11,17
(defn insert     "Inserts one or more facts…"   [session & facts]   (eng/insert session facts))
(defn insert-all "Inserts a sequence of facts…" [session fact-seq]  (eng/insert session fact-seq))
```

**Both call the same `(eng/insert session facts)`.** In Clara a *sequence* is the base case and the
single-fact call is varargs sugar. Ours has only the one-fact form, so a caller seeding N facts writes
a `foldl` threading the session by hand — which is exactly what `insert-all` exists to prevent, and
exactly what `wat-scripts/perf/grid/accum.wat:120,128` does.

## The cost, measured (`probe-insert-cost-split.wat`, n=40,000, release, mean of 3)

```
  baseline  2223.8 ns/fact    fold + construct + read a field
  conj      1779.4 ns/fact    fold + construct + PersistentVector/conj
  insert    2806.4 ns/fact    fold + construct + :wat::rete::insert
                              insert - conj = +1027 ns
```

**~1.03 µs per fact above a bare `conj`** — the Session reconstruction the current form performs *per
fact*: 6 accessors + one for `:facts` + a conj + a 7-field constructor. At 40,000 facts that is
**~41 ms of pure rebuild**, and batching collapses N reconstructions into one.

(`conj` reading below `baseline` is real: an interpreted record-field accessor costs more than a
`conj`. `insert − conj` is therefore the conservative isolation and the one quoted.)

Context, so this is not oversold: seeding 40,000 facts costs **254 ms** total (measured: total-eval
minus fire, minus a fixed-size run). Only ~41 ms of that is ours — the rest is the harness's own
interpreted loop constructing records, which a real consumer receiving facts off a wire does not pay.
**The claim is ~41 ms, not 254 ms.**

## Why it is semantically free

`insert` performs **zero activation** (`wat/rete.wat:828-830` — working memory stays open until
`fire-rules`). So batch insert is "extend `facts` by N" instead of "extend by 1, N times". No ordering
question, no activation subtlety, no truth-maintenance surface. A rare combination: a measured win
with no correctness risk.

## ★ THE ONE CONTRACT DECISION

**`insert-all` is the primitive. `insert` keeps its existing 2-ary clause UNCHANGED and gains a
variadic clause that delegates.**

Clara routes even the single-fact call through the sequence form. **We must not**, and the reason is
the target: the chaos engine (R25) takes facts *one at a time off a wire*, so the 2-ary path is the
streaming hot path. Sending it through `insert-all` would force a one-element `PersistentVector`
allocation onto the case that matters most, to buy nothing.

```clojure
(:wat::core::defn :wat::rete::insert
  ([session <- :Session  fact <- :Record] -> :Session
    (:wat::rete::insert' session fact))                                    ;; UNCHANGED — hot path
  ([session <- :Session  fact <- :Record  & rest <- :Vector<Record>] -> :Session
    (:wat::rete::insert-all session (conj-front fact rest))))              ;; new — sugar
```

Multi-arity `defn` with a typed rest-param is proven in the corpus: `:wat::core::+`
(`wat/core.wat:68-78`) has exactly this shape.

## The dual-impl, mirroring `insert`

`insert-all` joins the trio the arc requires — the oracle is never skipped:

| existing | new |
|---|---|
| `insert-spec` (wat oracle) | `insert-all-spec` |
| `insert'` (native prime) | `insert-all'` |
| `insert` (one-line delegate) | `insert-all` |

`insert-all'` resolves `facts` **by name** through `RecordDef.field_names` — never a positional index
— exactly as `insert'` does (`DESIGN-STONE-native-insert.md`'s contract, and the same silent-wrong-slot
risk applies).

## Blast radius

`wat/rete.wat` (three new forms + one clause on `insert`), `src/rete/` (`eval_insert_all_native`),
`src/runtime.rs` (one dispatch arm beside `insert'`). **No call-site churn** — `insert`'s 2-ary
signature is untouched, so every existing caller compiles unchanged. No corpus migration, no codemod.

## The RED gate

`insert-all` does not exist → `UnknownFunction: :wat::rete::insert-all`. Red today, exactly as the
native-insert gate was.

Four assertions:

1. **Equivalence** — `insert-all(s, [f1..fN])` produces a Session structurally identical to N chained
   `insert` calls. This is the correctness claim and it is the load-bearing row.
2. **The oracle** — `insert-all-spec` == `insert-all'` on the same input.
3. **Non-vacuity** — N > 1 and the resulting `facts` length is exactly N. A no-op `insert-all`
   returning the session unchanged would pass (1) and (2) against an empty vector.
4. **The 2-ary path is untouched** — a single `insert` still routes to `insert'`, not through
   `insert-all`. Assert it by behaviour (fact count) plus a read of the emitted form.

**Measured expectation:** seeding 40,000 facts via one `insert-all` should drop ~41 ms against the
`foldl`. Say the real number; this is a subtraction of a measured per-fact cost, not a model.

## Out of scope = REJECTED (affirmative cuts)

- **Routing the 2-ary `insert` through `insert-all`.** See the contract decision — it taxes the
  streaming path, which is the target.
- **`retract-all`.** Retract has a real, unmeasured O(everything) problem
  (`PVRITAS VERVM NON CELERITATEM`); adding a batch door there without measuring first would be the
  corpus-census error in a new costume.
- **Incremental insert-and-fire (R22's T3).** Different stone. Batch is bulk-load; the chaos engine
  wants incremental. Conflating them is how the streaming gap stays hidden.
- **`wat/rete.wat`'s oracle logic beyond adding `insert-all-spec`.** The oracle is never optimized.
