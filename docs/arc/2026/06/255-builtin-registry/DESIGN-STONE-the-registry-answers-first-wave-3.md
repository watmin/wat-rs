# DESIGN — STONE: wave 3 — the last five guards become registrations

> **Builder, 2026-08-31:** *"the purity measurements rete is doing… they will be satisfied by the
> registry when we're done… not there yet."* → *"we continue."*
>
> Waves 1–2 moved 28 stranded facts into the registry and deleted 16 guards. Nine hand verdicts
> remain that are not `rete_op_for`. **Pre-flight splits them 5/4.**

## ⛔ THE FAMILY SPLITS — four of the nine are NOT VERBS

| head | dispatch | verdict |
|---|---|---|
| `aggregate-new` · `kwargs-construct` | `runtime.rs:5533/5538` → named fns | ✅ homeable |
| `write-forms` · `with-children` | `runtime.rs:5371/5376` → `edn::render::eval_*` | ✅ homeable |
| `macro-error` | `runtime.rs:5390`, inline body → `Err(MacroAbort)` | ✅ homeable |
| **`verify::string` · `file-path` · `http-path` · `s3-path`** | inside **`resolve_verify_payload`** (`runtime.rs:24503`), matched on a `locator_ast` | ⛔ **NOT VERBS** |

★ **The `verify::` four are locator tags inside a payload construct, not callable heads.** Zero
corpus calls (`grep` over the whole `.wat` corpus, comments stripped: 0). Two of them
(`http-path`/`s3-path`) are `loader.rs:123`'s *"reserved but not"* implemented and unconditionally
raise — `purity.rs:463` already says so. **Homing a locator tag as an intrinsic would register a verb
that does not exist**, which is the opposite of what this campaign is for.

⚠ Their guard therefore STAYS, and it raises a question this stone does not answer: **why is
`intrinsic_meta` asked about four names that are never call heads?** Recorded, not resolved.

## THE ONE CONTRACT DECISION — pinned

**A guard retires by becoming a REGISTRATION, never by being deleted.** Where a head is not a verb,
the guard stays and the anomaly is recorded — retiring it would delete a verdict with nowhere to go.

## The five, and what each needs

```
aggregate-new     eval_aggregate_new          thin delegate
kwargs-construct  eval_kwargs_construct       thin delegate
write-forms       edn::render::eval_write_forms    thin delegate (cross-module)
with-children     edn::render::eval_with_children  thin delegate (cross-module)
macro-error       INLINE body                 the only one that is not already a named fn
```

⚠ **`macro-error` is the interesting ruling.** Its body always returns `Err(MacroAbort)` — it never
produces a value. Is a macro-abort SIGNAL a raise (`Partial`), or is it the propagation shape the
`try` verbs were ruled `Total` on? `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`
is the rule; **the rider measures which side of it this falls on and cites the line.** The two `try`
verbs are the precedent for a signal that is not a raise; `Result/expect` is the precedent for one
that is.

## ★ THE PREDICTION — uneven, measured, falsifiable in both directions

```
write-forms · with-children        register_builtins: YES  ->  NO debt row
aggregate-new · kwargs-construct · macro-error   NO       ->  a row each
FROZEN_CHECKER_DEBT   68 -> 71
KNOWN_UNREVIEWED      34 -> 34    (none of the five is on that ledger — measured, 0 rows each)
```

⚠ **The ratchet does NOT move**, and that is the prediction most likely to be wrong in a way that
looks like success. If `KNOWN_UNREVIEWED` shrinks, something was on it that my measurement said was
not — a finding, not a bonus.

## Out of scope = REJECTED (not deferred)

- **The `verify::` four.** Not verbs. Their guard stays; the anomaly is recorded above.
- **`rete_op_for` / the 74 rete-surface verbs.** ⛔ **And wave 2's DESIGN mis-framed this** — see
  `[[NOTE-rete-ops-is-a-population-missing-from-the-registry-not-an-authority-over-it]]`. It is not an
  authority overruling the registry; it is 74 verbs the registry has never heard of (`rete_name`
  registered **0/74**), across four `OpClass`es that differ in whether a registration would even
  carry a scheme. A homing campaign with its own design, not a wave of this one.
- **`accessor_meta` / `constructor_meta`.** Arc 293.W owns that ruling.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **home the five; the `verify::` four stay with a NOTE** | YES | YES | YES | YES | ✅ **ADMITTED** |
| home all nine | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| delete the `verify::` guard as dead | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| home the five AND start the 74 | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| rule `macro-error` `Partial` by family resemblance | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **all-nine Honest? NO** — four are locator tags reached through `resolve_verify_payload`, never as
  call heads. Registering them would assert verbs that do not exist.
- **delete-as-dead Honest? NO** — "no corpus call" is not "unreachable". The measurement shows they
  are not CALL HEADS; it does not show `intrinsic_meta` is never asked. Deleting a verdict on a
  question I have not asked is the failure this campaign keeps finding.
- **five-plus-74 Simple? NO** — a red could not be attributed.
- **`macro-error`-by-resemblance Honest? NO** — `try` and `expect` share a family and got OPPOSITE
  verdicts. The body decides.

## Acceptance

| what | command | expected |
|---|---|---|
| the five are registered | `lookup_entry` each | `Some` |
| each `@Totality` is its own | the five declarations | measured, cited per verb |
| ★ `macro-error`'s ruling is derived | its `@Totality` | cites the RULING and the line, not a sibling |
| the five guards are gone | `intrinsic_meta` | five fewer `OpMeta` literals |
| ★ the `verify::` guard is UNTOUCHED | `purity.rs` verify block | byte-identical |
| the uneven prediction | `FROZEN_CHECKER_DEBT` | 68 → **71**, and only those three |
| ★ the ratchet does NOT move | `KNOWN_UNREVIEWED` | **34**, unchanged |
| no purity/determinism guesswork | each declaration | derived from the body, cited |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5110/5110, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
