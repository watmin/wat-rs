# DESIGN — STONE meter-2: the completeness gate's DISPATCH half sees the whole file

> Found 2026-08-30 chasing why `:wat::core::Some` denies. **`meter-1` fixed the REGISTRATION half of
> this gate. The DISPATCH half has the identical disease**, and nobody looked because meter-1's
> finding read as "the gate hole", singular. It was one of two.

## The defect

`dispatch_verbs` (`src/rete/purity.rs`) builds the dispatched population by scanning text between
**two named anchors**:

```rust
for anchor in ["fn dispatch_keyword_head_value(", "fn dispatch_substrate_impl("]
```

Everything dispatched anywhere else is invisible. Measured across `src/runtime.rs` — 104 literal
`":wat::…" =>` arms:

| function | arms | scanned? |
|---|---:|---|
| `dispatch_keyword_head_value` | 74 | ✅ |
| `register_runtime_defs_form` | 8 | ⛔ |
| `eval_tail` | 8 | ⛔ |
| `step_list` | 5 | ⛔ |
| **`dispatch_keyword_head`** | 4 | ⛔ — **one word off the anchored name** |
| a `pub fn` | 3 | ⛔ |
| `resolve_verify_payload` | 2 | ⛔ |

**And a second ARM SHAPE the scan does not know at all** — a keyword guard, used in `fn eval_list`:

```rust
WatAST::Keyword(k, _) if k == ":wat::core::Some" => { … }
```

Eight of those, **every one unregistered**: `:wat::core::Some` · `:wat::core::Ok` ·
`:wat::core::Err` · `:None` · `:undefined` · `:wat::core::def` · `:wat::core::defalias` ·
`:wat::core::fn`.

★ **This is why the campaign never saw them.** `Some` is dispatched, unregistered, absent from
`KNOWN_UNREVIEWED`, and absent from `WORKLIST-the-44-unhomed.md` — and the floor is green, because
the ratchet whose rule is *"a verb NOT in this list ⇒ RED. A new dispatch verb needs a ruling,
always"* cannot see it. The 44 was never the population.

## THE ONE CONTRACT DECISION — pinned

**The scan's population is defined by SHAPE, not by the name of the enclosing function.** Any
dispatch arm keyed on a wat FQDN counts, wherever it lives. Anchoring on function names is what
produced both halves of this defect; replacing two anchors with three does not fix it, it reloads it.

## What ships

1. `dispatch_verbs`' literal-arm scan reads the **whole file**, not the span between two anchors.
2. It also recognises the **keyword-guard shape** (`WatAST::Keyword(k, _) if k == "…"`).
3. Every newly-visible verb is **disposed**, one at a time — see below.

## Disposing the screams — meter-1's precedent, and its lesson

`meter-1` predicted *"~25 verbs will scream"* and measured **ELEVEN**. ⚠ **The ~38 implied by the
table above is a PREDICTION from a text scan, not a measurement.** The rider reports what actually
screams; a mismatch is data, not a problem.

Each scream gets **one** of two dispositions, and the choice is per-verb:

- **RULE IT** — classify in the registry where the answer is plain from the implementation.
- **A NAMED `KNOWN_UNREVIEWED` ROW** where the ruling is genuinely open. This is the list's stated
  last resort and meter-1 used exactly it for its eleven.

⛔ **This is NOT the laundering the gate warns about.** These verbs have been dispatched all along;
the rows record **pre-existing debt that just became visible**, not new debt waved through. The
distinction is real and must be written into each row's reason — a row that does not say *why* its
ruling is open is the laundering.

## Out of scope = REJECTED (not deferred)

- **Homing `Some`/`Ok`/`Err`** and unblocking the accessor — the next stone. Making them **visible**
  is this one. Ruling a verb the meter cannot see is building on sand.
- **`def`/`defalias`/`fn`** — the declaration door, a known open item with its own shape. Expect them
  to scream; dispose them with a `KNOWN_UNREVIEWED` row naming the declaration-door question.
- **Widening the gate to `src/` beyond `runtime.rs`** — measure first; if arms live in other files
  the rider reports it rather than silently growing the scope.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **meter-2** scan by SHAPE, whole file | YES | YES | YES | YES | ✅ **ADMITTED** |
| add the six missing fn names as anchors | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| home `Some`/`Ok`/`Err` first, fix the meter later | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **more-anchors Honest? NO** — it fixes today's six and silently reloads the same gun for the
  seventh. The defect is *anchoring on names*, not *which names*. `dispatch_keyword_head` sitting one
  word from `dispatch_keyword_head_value` is the proof of how quiet the next miss would be.
- **home-first Obvious? NO** — a reader cannot tell why those three verbs and not others.
  **Honest? NO** — it treats the symptom the blind spot produced while the blind spot keeps
  producing, and every wave's numbers still come from a meter known to under-report.

## Acceptance

| what | command | expected |
|---|---|---|
| the guard shape is seen | `dispatch_verbs` output contains `:wat::core::Some` | present |
| the other fns are seen | output contains an arm from `eval_tail` / `step_list` | present |
| every scream is disposed | the completeness gate | green, with each new row carrying a REASON |
| no ruling is guessed | each newly-RULED verb | cites the implementation line it was ruled from |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
