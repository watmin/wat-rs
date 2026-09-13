# BRIEF — the check skips what cannot dial

**Read `DESIGN.md` beside this first.** It carries the construction proof (why the guard cannot change
behaviour), the measured cost this is recovering, and the **contract that the guard ships only if its saving
exceeds the noise band** — with reverting as a permitted outcome.

## The work, in one paragraph

The third bijection check added at `49bdf2b41` walks every `defservice`'s `:impls` tree unconditionally, even
when the service declares **no `Address`-typed field** and so cannot possibly have a resolvable dial. Guard it
on `addr-fields` being empty, then **measure the floor with and without, ≥3 runs each**, and rule on the
number: keep if the saving clears the ~6 s band, revert and record the null if it does not.

## The rooms — both in `wat/service.wat`

| where | why you are going there |
|---|---|
| `:971` | `addr-fields  (:wat::vec::concat (addr-fields-of durable-fields) (addr-fields-of ephemeral-fields))` — the list the guard tests. ⭑ Read how it is built: the collector's **only** way to add a surface is matching a connect argument against one of these fields, which is what makes the guard a proof rather than a hope. |
| `:1029–1031` | ⭑ **THE ROOM.** `impls-dialed-surfaces (collect-impls-dialed collect-impls-dialed ops (Vector String))` — wrap this in the guard. |
| `:889–922` | the two sibling checks. **Do not touch them.** They are cheap (they fold over already-computed lists); only the `:impls` tree walk is the cost. |
| `tests/services/probe_dial_declares_its_peer.rs` + its two `.wat` fixtures | the refusal and the declared-redial, as floor tests. ⭑ **Both must stay green** — they are the guard against a fast path that skips too much. |

## Implementation sketch

```
impls-dialed-surfaces  (:wat::core::if (:wat::core::empty? addr-fields)
                         (:wat::core::Vector :- [:wat::core::String])
                         (collect-impls-dialed collect-impls-dialed ops
                           (:wat::core::Vector :- [:wat::core::String])))
```

⚠ **The sketch is a convenience; the disk is the contract.** Check whether `empty?` accepts the vector type
`addr-fields` actually has (it is built with `:wat::vec::concat`, so confirm the element type and the right
emptiness predicate — `:wat::core::empty?` vs a `vec`/`length` form). **My sketches have been wrong seven times
in this campaign**, including one that would have caused the trap-door I had ranked #1. Take the shape, verify
every name.

## ⭑⭑ The measurement IS the deliverable

| what | how |
|---|---|
| **≥3 floor runs WITHOUT the guard** | current `HEAD` state, quiet box, `./scripts/floor.sh` each time |
| **≥3 floor runs WITH the guard** | same, after the change |
| the ruling | keep if the mean saving clears the **~6 s** band by a clear margin; **revert** if not |

⚠ **The box must be quiet** — this is a *timing* measurement. (This campaign learned the inversion the hard
way: a quiet box is right for timing and **hides races**; a loaded box reveals races and lies about timing.)

⭑ **Reverting is a permitted, expected result.** The +27–47 s may be dominated by services that *do* hold
Address fields — the queue, the topic, the workers. If so, say so with the numbers and take the guard out; an
unmeasurable optimisation inside the `defservice` macro is complexity with no evidence.

## Also verify

- ⛔ **The refusal still fires:** `cargo nextest run --release -E 'test(dial_declares)'` → **2 passed**.
- All three corpus controls still `--check` clean: `sqs.wat`, `circuit.wat`, `sns-fanout.wat` → exit **0**.
- `./scripts/floor.sh` green at **5241** (not 5237, not 5239 — two stones have moved it).
- ⛔ **Never a piped exit code** (a type-error run this session reported `$?` = 0 through `| head`; true exit 3).
- `cargo clippy --release --workspace --all-targets -- -D warnings` → 0; `git diff --stat -- src/` **EMPTY**.
- Happy path `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — if the guard changes any behaviour**, STOP. It is supposed to be a construction-proof no-op on
   semantics: empty `addr-fields` ⇒ no resolvable dial ⇒ empty result. If that reasoning does not hold on the
   disk, the guard is wrong and I want to know why before any measurement.
2. **STOP-2 — if the saving does not clear the band, REVERT and report.** Do not keep it "because it must help."
   The null is the ruling, and DESIGN says so in advance.
3. **STOP-3 — do not touch the two sibling checks or the check's semantics.** Fast path only.
4. **STOP-4 — do not guard on anything but `addr-fields` emptiness.** A cleverer predicate loses the
   construction proof, which is the only reason this is cheap to trust.
5. **STOP-5 — if the box cannot be made quiet**, STOP and say so rather than quoting a timing number taken
   under load. That exact mistake produced a published-and-wrong rate earlier in this campaign.

## Shape to copy

`the-dial-declares-its-peer/SCORE.md` for the cost measurement it established (and the per-test breakdown), and
`fill-is-refusal-bound/FINDING.md` for a stone whose honest result was **"measured, and it does not help"**.
