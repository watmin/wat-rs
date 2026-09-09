# DESIGN — the instrument reports what happened

**`wat-scripts/fanout/circuit.wat` only.** No system code. Two counters lie, and both lies have
already misled this arc.

## WHY — `stop` is 73 % bookkeeping

Measured by instrumenting the phase, `stop = 6613 ms`:

```
calls+ticks       45 ms
topic-ticks        2 ms
sum-disrupts    1262 ms   ← 105 ms per worker x 12
seen               3 ms
collect-stop    3509 ms   ← 8000 Outcome records shipped to the parent
teardown        1790 ms   ← 26 process reaps, 69 ms each   ← the ONLY system cost
```

★★ **`stop` is 27 % teardown and 73 % the benchmark measuring itself.** And this is the third time
today harness cost has been read as system cost — the driver's 1 ms poll loop counted inside
`publish`, my pinned-delay runs mis-reading `asleep`, and now this.

## WHY — `full-retries` undercounts

`publish-until-accepted!*` increments `retries` **only** in the `Accepted 0` branch. The partial
branch recurses with `retries` unchanged, so **every partial resend is invisible**.

★ That cost a wrong conclusion: I computed "5.7 ms per call" from `retries + 200`, called it
per-call overhead, and built a model on it. The real call count was higher and the quotient
meaningless — it was attributing *blocked* time to calls that were merely waiting.

## ⛔ THE ONE CONTRACT DECISION — a phase measures one thing

`stop` measures **teardown**. A new `collect` phase carries the bookkeeping. Both are reported;
neither is hidden. The sum still accounts for the wall.

★★ Not "make the bookkeeping faster" — **it is measurement, and measurement belongs outside the
thing being measured.** Whether `collect-stop` should page (it is an unbounded response, and
`Worker::disrupts` returns an accumulated `points` String) is a real question and **not this
stone**: first make the number honest, then decide if it matters.

## AND `publish-attempts` COUNTS EVERY ROUND TRIP

`full-retries` keeps its meaning — bounces. A new counter reports **total topic calls**, so a
per-call figure can be computed without the error I made.

## ⚠ THE PINNING HAZARD, RECORDED

`asleep` sums the **drawn** delay and the shipped code awaits that same value, so it is correct as
shipped. It is wrong only when an experiment pins the delay by hand and the accumulator keeps
adding the draw. **That is a property of the technique, not a defect** — but it produced a bad
number for me today, so it goes in the file where the next person pinning a delay will read it.

★ Along with the sharper version of the same lesson: **pin BOTH delay sites or neither.** The
parent helper and the Publisher child's inlined copy are separate; patching one silently measures
the other, which gave me a false refutation of a SCORE earlier today.

## OUT OF SCOPE — REJECTED

- **Paging `collect-stop` / bounding `Worker::disrupts`.** Real — they are unbounded responses of
  exactly the kind `:max-page` exists for — but the question of whether 4.8 s of bookkeeping is
  worth optimising should be asked *after* it stops being counted as system time.
- **The 1.8 s teardown.** 69 ms per process reap is the real system cost in `stop` and is the
  sibling of the cold-boot `setup` the builder has parked. Its own stone.
- **Any system code.** This stone changes the benchmark and nothing else.
