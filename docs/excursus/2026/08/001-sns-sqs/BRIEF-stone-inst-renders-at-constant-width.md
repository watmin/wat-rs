# BRIEF — excursus 001 stone INST: `#inst` renders at constant nanosecond width

**One token in the substrate. The value is that it makes a silent-data-loss class impossible.**

## The work, in one paragraph

`crates/wat-edn/src/writer.rs:227` renders an instant with `chrono::SecondsFormat::AutoSi` —
*"the shortest representation that is a multiple of 3 digits"*. So `1.200000000s` prints
`".200Z"` while `1.200000100s` prints `".200000100Z"`, and `'Z'` (0x5A) sorts **after** `'0'`
(0x30): **the earlier instant compares greater.** Every range `scan` over a timestamp sort key
depends on lexicographic order matching chronological order, and it does not hold. Change
`AutoSi` to `Nanos` (always 9 digits), decide the JSON sibling, and land the probe that pins the
property so it cannot silently stop being true again.

## Why this is worth a substrate change rather than a workaround

`:wat::query::Store/scan` orders by the `sk` **string**. `:wat::telemetry::journal` puts a
timestamp there — and hand-pads it to 9 digits in `:wat::telemetry::time-sk`, precisely because
the renderer trims. **That hand-padding is a local workaround for a global defect**, and the
one place that reasonably used the native renderer instead would get a silently truncated key.

`src/intrinsic/time.rs:70` already documents `(:wat::time::now)` as
`#inst "2026-08-13T12:00:00.000000000Z"` — **nine digits, trailing zeros included.** The
documentation already promises the property; the renderer is what is out of line.

Nine digits is RFC-3339 legal (`time-secfrac = "." 1*DIGIT`, unbounded). **Measured against
Clojure 1.12.4 on this host**, not recalled:

| reader | result |
|---|---|
| `read-string` (default) | `java.util.Date` — `.getTime` 1200ms, **nanos dropped** |
| `clojure.edn/read-string` | same — **lossy** |
| `clojure.instant/read-instant-timestamp` | `java.sql.Timestamp` — `.getNanos` 200000100, **exact** |

The truncation is `java.util.Date`'s physical limit, not a parse failure, and it already
swallows any sub-millisecond instant we emit at any width. Clojure *reads* nanosecond `#inst`
fine, which is the direction that matters.

## The gate — RED at HEAD, measured

`docs/excursus/2026/08/001-sns-sqs/PROBE-inst-lexicographic-order-is-not-chronological.wat`

At HEAD, driving its comparisons directly gives:

```
9-digit-boundary=false  whole-second=false  6-digit=false  3-digit=false
same-width-control=true    widths=32/38/28
```

**Four boundary rows red, the control green, and three different widths.** The rows sit on
AutoSi's 0/3/6/9-digit switches — chosen from the RULE (where does the renderer change width?),
not from the single failure that was found.

**The stone is done when all six deftests pass with no edit to the probe.**

## Read in order

1. **`crates/wat-edn/src/writer.rs:227`** — `to_rfc3339_opts(SecondsFormat::AutoSi, true)`.
   The defect, and the whole fix.
2. **`crates/wat-edn/src/json.rs:170`** — the same call in the JSON writer. **A decision, not a
   copy-paste:** EDN's `#inst` is a sort key in this system; JSON's is an interchange value.
   Make them consistent unless you can say why not — and if you keep them different, say so in
   the SCORE.
3. **`src/intrinsic/time.rs:70`** and **`:208` (`to-iso8601`)** — `to-iso8601` takes an explicit
   digit count and uses a **different** path (`SecondsFormat::Secs` at `:222`). It is NOT
   affected, which is why the golden at `wat-tests/time.wat:69–74` does not churn. Confirm that
   rather than assuming it.
4. **`wat/telemetry/journal.wat`, `time-sk`** — the hand-padding this makes redundant. **Do not
   delete it in this stone** (see out-of-scope).

## Implementation sketch

```rust
// crates/wat-edn/src/writer.rs:227
- out.push_str(&dt.to_rfc3339_opts(SecondsFormat::AutoSi, true));
+ out.push_str(&dt.to_rfc3339_opts(SecondsFormat::Nanos,  true));
```

Then promote the probe into `wat-tests/` so the property is on the floor permanently.

## Blast radius

- `crates/wat-edn/src/writer.rs` — one line
- `crates/wat-edn/src/json.rs` — one line, or a stated decision not to
- `wat-tests/<promoted probe>.wat` — new
- this arc's SCORE

**Measured golden churn: zero.** There are 11 ISO-8601 timestamps in `tests/` + `wat-tests/`;
ten are already 9-digit, and the one that is not (`wat-tests/time.wat:69`) goes through
`to-iso8601` with an explicit digit count — a different code path. `AutoSi` appears in exactly
two places, both listed above.

## STOP triggers

1. **If any golden churns** — the census above said zero. If one does, **STOP and report which**;
   a wrong blast-radius estimate means the census instrument was wrong, and that matters more
   than the golden.
2. **If the floor reds anywhere outside the two `wat-edn` lines and the new test** — STOP,
   capture the arm whole, do NOT re-run.
3. **If you find yourself editing the probe** — STOP. It is the gate.
4. **If `Nanos` turns out not to be constant-width** for some input (a pre-1970 instant, a
   leap second, a far-future year) — STOP and report the input. The property is
   *constant width*, and a counterexample is worth more than a green floor.

## Out of scope, affirmatively cut

- **Deleting `time-sk`.** It becomes redundant, and removing it is a telemetry change with its
  own callers. Not this stone; it is named in the SCORE as owed.
- **`journal`'s `SortKey`.** The key-collision bug
  (`NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md`) is downstream of this and
  is drawn separately.
- **A `:wat::gen::` property test.** This would be a natural generative property, but wat-gen
  lives on `grok-rete` and is not on this branch. The boundary table is the honest instrument
  until the branches converge.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

Read the Summary line. Floor at HEAD on this branch is **5097 with 1 pre-existing failure**
(`probe_arc278_span_macros`, the journal key-collision bug — see the NOTE). **That red is
expected and is NOT yours.** Your stone must not add a second one.
