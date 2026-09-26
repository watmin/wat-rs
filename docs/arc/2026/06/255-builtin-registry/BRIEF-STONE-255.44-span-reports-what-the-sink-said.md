# BRIEF — STONE 255.44: the span reports what the sink said (`LogResponse` mirrors `CloseResponse`)

**Drawn 2026-09-26 against `main` @ `cbaac2bbb`.** Floor 6137/6137, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 198. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first

`WEIGH-STONE-255.43-telemetry-faces-its-outcomes.md`. Your own STOP-1 there was correct.

## Ruled (builder, 2026-09-26): L1, four YES

`wat/telemetry/span.wat` ~:98 binds `_w` to the journal's `write-logs` `RecvOutcome` and replies
`Span::LogResponse.Ok` whether or not the write succeeded. **`Span::LogResponse` (`wat/telemetry.wat` ~:257)
mirrors its sibling `Span::CloseResponse` (~:263)**, which already passes the sink outcome through. It gains
the sink's failure arms:

```
(:wat::core::defenum :wat::telemetry::Span::LogResponse :wat::enum::Pure
  :Ok               []
  :Constraint       [err <- :wat::query::Constraint]
  :Transient        [err <- :wat::query::Transient]
  :Fatal            [err <- :wat::query::Fatal]
  :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
  :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
```

Rejected: L2 (keep replying `Ok`: Honest N), and L3 (map a failure onto `RequestMalformed`: a store failure is
not a malformed request).

## The work

1. **Extend the `defenum` as above.** Keep `CloseResponse`'s field names and types exactly, so the two
   siblings read the same.
2. **`span.wat`: face the write.** Replace `_w` with a `match` over the `RecvOutcome` and its
   `Journal::WriteLogsResponse`, and reply with the honest arm:
   - `Success` → `Ok`;
   - `Constraint`/`Transient`/`Fatal` → the same-named `LogResponse` arm carrying the journal's `err`;
   - the journal's own wire breach (its `RequestTooLarge`/`RequestMalformed`), and the `RecvOutcome` non-message
     arms (the journal is gone or stopped): **measure what `CloseResponse`'s producer already does** for these
     same arms, and **do exactly that**, so the two siblings cannot disagree. If `CloseResponse`'s producer has
     no precedent for one of them, STOP and report it.
3. **Consumers of `Span::LogResponse`:** census them. Each exhaustive `match` gains the three arms. Read each
   body: a consumer that treated `Ok` as "logged" must now face a failure honestly. Report every site and what
   it does now.
4. **Rows:** a span whose journal replies `Fatal`/`Constraint`/`Transient` (a `Store` satisfier that fails the
   write, like 255.43's probe for `ensure-schema`) → the caller receives that `LogResponse` arm with the
   `err`. **Pre-stone it received `Ok`.** Prove both words on the pre-stone build. Plus the success twin.

## STOP triggers

1. `CloseResponse`'s producer has no precedent for one of the arms in step 2 → STOP, report it.
2. A consumer's body cannot honestly face a failure (it has no way to report one upward) → report the site
   and the shape; do not invent a new variant there.
3. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `LogResponse` | mirrors `CloseResponse`'s failure arms |
| `_w` in `span.wat` | gone, the write matched |
| the failure rows | the caller sees `Fatal`/`Constraint`/`Transient` (pre-stone: `Ok`) |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 198 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger.
- Tests assert exact values, never loose `contains`/`starts_with`, never a count of source text.
- Line-pinned `.edn` goldens that move: recapture them with `UPDATE_EDN=1`, and show a `:line`-only diff.
- ⭐ **A STOP trigger means STOP.**
- **Declare things where they are logical; a new file is fine** (the builder's standing ruling).
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.44-span-reports-what-the-sink-said.md` beside this brief.
