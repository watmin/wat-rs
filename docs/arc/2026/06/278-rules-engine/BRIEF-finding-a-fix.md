# BRIEF — Finding A FIX: the self-scheduling `-tick` op ref drops its keyword colon

> **Tier:** sonnet shadowdancer. **Arc:** 278 item-c — the payoff (the `VNDE ORTVM` tail). **HEAD:** `4543ef7a`.
> This is the last stone: a one-character root fix + un-`#[ignore]` the RED gate + weigh it green.

## The root (grounded — do not re-diagnose)

The self-scheduling serve loop dies with `RuntimeError: UnboundSymbol "probe::ticker'::Op::-Tick"`. The
`defservice` macro resolves a handler body's `:op :-tick` keyword to its `<service>::Op` variant ctor via a
source round-trip (`ast->source → split/join → read-string`, `wat/service.wat:896-910`), and the
replacement string comes from `internal-op-repl-strs` (`wat/service.wat:849-868`):

```
"(" + service-op-str + "::" + variant-pascal + ")"   →   "(probe::ticker'::Op::-Tick)"
```

`service-op-str` is `"probe::ticker'::Op"` (no leading colon — `fqdn-str = keyword/to-string(fqdn)` at
`:90` strips it). So the head has **no leading colon** → `read-string` parses `probe::ticker'::Op::-Tick`
as a **bare symbol** → eval → `UnboundSymbol`. A variant constructor must be a **keyword** form
`(:probe::ticker'::Op::-Tick)`. The def side is correct (`:798` builds the variant kw with `:`; `:811`
defines `:probe::ticker'::Op` with the `:-Tick` variant; comment `:833` shows the intended
`(:<fqdn>::Op::-Tick)` *with* the colon). The ref side dropped the `:`.

## The fix (one character)

In `wat/service.wat`, `internal-op-repl-strs` (~`:862`), change the leading `"("` to `"(:"`:

```clojure
;; before:
(:wat::core::conj acc
  (:wat::core::string::concat "("
    (:wat::core::string::concat service-op-str
      (:wat::core::string::concat "::"
        (:wat::core::string::concat variant-pascal ")")))))
;; after — prepend the keyword colon so the ref is a variant CONSTRUCTOR, not a bare symbol:
(:wat::core::conj acc
  (:wat::core::string::concat "(:"
    (:wat::core::string::concat service-op-str
      (:wat::core::string::concat "::"
        (:wat::core::string::concat variant-pascal ")")))))
```

Confirm this is the ONLY internal-op ref-construction site (grep `internal-op-repl` — the round-trip at
`:896-910` consumes it; there is no second builder). Do NOT touch the def side (`service-op-variant-items`,
`:785-811`) — it's correct.

## Un-`#[ignore]` the RED gate

In `tests/services/probe_arc278_self_scheduling.rs`, remove the `#[ignore = "..."]` on BOTH tests
(`self_tick_fires_rearms_and_reactor_serves_thread`, `..._process`). They must now pass (the `-tick` arms,
fires, re-arms to target=3; `poll` replies 3). Leave their assertions unchanged (return `i64` == 3).

## RED-gate STOP (the fix might reveal a next layer)

The colon fix makes the `-tick` ref resolve. If ticking now proceeds but a **different** error surfaces —
STOP and report it, do NOT chase it in this strike:

- **STOP-1:** `(:probe::ticker'::Op::-Tick)` is *still* unbound/unknown even with the colon → the
  leading-dash variant isn't registered as a nullary constructor (a deeper enum-binding gap); report the new
  error verbatim.
- **STOP-2:** ticking works but a `poll'` reactor-class / idx-shift error now appears (the SCOUT's original
  suspects a/b, only reachable once ticking runs) → report it; that's the next strike.
- **STOP-3:** thread passes but process-tier fails with a tier-specific error → report both; do not fix the
  process path here.

## Verify (weigh by your own re-run)

1. `./target/release/wat --check tests/services/probe_arc278_self_scheduling.wat` clean.
2. Both self_scheduling tests (no longer ignored), thread AND process, run + PASS:
   `cargo nextest run --release self_tick_fires_rearms_and_reactor_serves 2>&1 | tee /tmp/claude-scout/findingA_fix.log`
   — both `passed` (count reached target 3).
3. **Whole release floor:** `cargo nextest run --release 2>&1` — READ THE SUMMARY yourself. Was 4207/0 with
   2 ignored; now the 2 formerly-ignored pass → **4209 tests run, 4209 passed, 0 failed** (skipped drops by
   2). Run twice; both 0-failed.

## Deliverable

The one-char fix in `wat/service.wat` + both tests un-ignored and passing + floor green. Report: (1) confirm
the fix + that it's the only ref site; (2) both self_scheduling tests PASS (paste the Summary); (3) two full
floor Summaries (both 0-failed, 4209 run); (4) `git diff --stat`. Do NOT commit — leave it for the
orchestrator to weigh.

## Blast radius

`wat/service.wat` (one char) + `tests/services/probe_arc278_self_scheduling.rs` (remove 2 `#[ignore]`s). No
`src/`, no other files. If a STOP fires, nothing else. Scratch logs → `/tmp/claude-scout/`.
