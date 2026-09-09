# DESIGN — find the 934 milliseconds

**A measurement stone.** `wat-scripts/queue/sqs.wat`, and possibly nothing else. The perf phase's
own rule 4 — *every perf stone names the new dominant term* — turned on a stone that broke it.

## WHY — the retry landed correct and cost 934 ms

`853704d24` is right: the queue survives `:Transient`, every other outcome dies by name, floor
green. It also made the arc's headline number **worse**, in a phase whose whole purpose is to
make it better.

A/B control, same quiet box, minutes apart, `sqs.wat` swapped in place and nothing else touched:

```
without the retry   publish 23629 23777 23723    median 23723
with the retry      publish 24506 24751 24657    median 24657
```

★ **+934 ms, entirely in `publish`.** `drain` is unchanged at ~200 ms, `setup` and `stop` unmoved.
8000 sends ⇒ **~117 µs per send**, which is **82 % of a whole 143 µs thread round trip** — I/O
scale, not syntax scale.

⚠ The SCORE reported +822 ms against **23672**, the baseline from *before* the two rollback
stones. Against the number immediately before this stone (23819, my run) it is +1065 ms. **A perf
delta measured against a stale baseline understates itself twice over** — that is its own lesson
and it belongs in the tracker's rules.

## WHAT IS ALREADY ELIMINATED — do not re-test these

### The closures are NOT the cost — measured twice

The visible suspect: `nap` and `once-put` are bound in a `let` on **every** send and ack, before
the outcome is known, and the happy path never calls them.

```
probe-what-a-closure-costs.wat                    two closures ~1.6 µs/iteration
probe-what-a-closure-costs-in-a-wide-scope.wat    ~1.8–2.3 µs with 16 live bindings captured
```

★★ Capture is **by reference, not environment-copy** — a wide capturing scope costs barely more
than a captureless one. 16000 ops × ~2 µs = **~32 ms of 934. 3.4 %.**

⚠ The first probe was itself flawed — captureless closures in a two-binding scope, which cannot
see a copy-cost even if one existed. The wide probe exists because that flaw was caught **before**
the number was used. Both are committed; the shape is reusable.

### The other named arms are NOT the cost — by measurement, not by reading

`drain` is unchanged. `take`'s re-put and scan-index arms, and `ack`'s whole retry, run on the
drain path. **They cost nothing measurable.** The same `_`→named-arms edit therefore costs nothing
in itself, which also clears `depth`/`total`'s arms of the *shape* charge.

## ⛔ WHAT REMAINS — and it is one structural change

The send arm's body **moved out of the `Success` match arm into a `let` sequence**:

```wat
;; before                              ;; after
(match sresp                           (let [nap … once-put …
  ((Success) <the whole send body>)          _ok (match sresp ((Success) nil) …)
  (_ (assertion-failed! …)))                 s' … <the whole send body>]
                                          …)
```

Everything else in the send arm is unchanged. **The cost is in that restructure and it is not the
closures.** That is a narrowing, not an answer, and this stone exists to finish it.

## ⛔ THE ONE CONTRACT DECISION

**Keep the retry. Recover the milliseconds.** The retry closes a real fatal flaw and does not get
traded away for speed — correctness first is the standing order. If the two cannot both be had,
say so with the measurement rather than choosing quietly.

## ★ THE OUTCOME THAT WOULD BE WORTH MORE THAN THE STONE

If ~117 µs per operation turns out to be what it costs to evaluate a large body in a `let`
sequence rather than inside a `match` arm, **that is a general fact about the wat interpreter**,
not a fact about the queue. It would apply to every service arm in the tree and would be the
largest single lever this perf phase has found.

⚠ So the honest failure mode here is finding a local workaround and shipping it without naming
the general fact. **Name the mechanism, then decide what to do about it.**

## OUT OF SCOPE — REJECTED

- **The retry's lying `Lost` arm.** `once-put`'s `Lost`/`Closed`/`TimedOut` all assert *"transient,
  exhausted after 3"*, and outside the retry the same `Lost` **redials and recovers**. A real
  defect, recorded at `853704d24`, and **its own stone** — it changes behaviour, this one must not.
- **Per-entry outcomes, the topic batch, `setup`/`stop`.** Later items on the ordered plan.
