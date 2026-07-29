# BRIEF — Stone 1: `:wat::edn::read-json` + `ReadJsonOutcome`

**Design:** `DESIGN-wat-mcp.md`. This is the ONLY substrate gap in `wat --mcp`; everything above it
is composition. Do not build any MCP protocol here — this stone is one verb and one enum.

## THE WORK

`wat_edn::from_json_string(s) -> JsonResult<OwnedValue>` is fully implemented and has **zero
consumers in `src/`**. The write half of the JSON bridge is wired (`:wat::edn::write-json`); the
read half never was. Wire it as `:wat::edn::read-json`, returning a **matchable outcome** rather
than raising.

Why an outcome and not a raise: this verb's input arrives from a REMOTE, UNTRUSTED harness over
stdio. A raise would let one malformed byte kill the session — the exact failure `read-string` was
converted to fix (`types.rs:952`: *"an arrow key at the REPL sends ESC (0x1B) … the raise unwound
THROUGH the loop and killed the session"*).

## ROOMS — read in this order

1. **`crates/wat-edn/src/json.rs:225`** — `from_json_string`, the function you are exposing.
   Also `:53` `JsonError` (its error type) and `:100` `JsonResult`.
2. **`src/edn_shim.rs:162`** — `eval_edn_read`. Your impl is this with a different parser at the
   front: parse → `edn_to_value(&edn, sym.types().map(|a| a.as_ref()))`. **Do NOT copy its error
   handling** — it raises (`:180`), which is the pre-LAW shape this stone must not repeat.
3. **`src/edn_shim.rs:455`** — `eval_read_string`. THIS is the shape to copy: same arg-checking,
   same `TrackedValue` + `RuntimeBuilt` provenance, and a total return.
4. **`src/edn_shim.rs:409` / `:424`** — `read_outcome_forms` / `read_outcome_malformed`, the
   variant constructors. Write the `ReadJsonOutcome` twins beside them.
5. **`src/types.rs:952`** — `ReadOutcome`'s registration + the comment explaining WHY the cause is
   the structural `:wat::core::Error` and not a parser-specific enum. Mirror both.
6. **`src/runtime.rs:5307`** — where `:wat::edn::write-json` is dispatched. Register beside it.
   NB `read-string` is routed as a *producer* (`runtime.rs:4351`) — follow whichever the
   `TrackedValue` return requires; `:5311`'s comment explains the producer split.
7. **`src/check.rs:17046`** — where `write-json`'s TYPE is registered. Yours goes here.
8. **`src/to_edn.rs:361`** — `impl WatError for FlatMessage`. **You will need this** (see sketch).

## IMPLEMENTATION SKETCH

```rust
// edn_shim.rs — beside eval_read_string
pub fn eval_edn_read_json(
    args: &[WatAST], list_span: &Span, env: &Environment, sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, RuntimeError> {
    const OP: &str = ":wat::edn::read-json";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = /* String or TypeMismatch — copy eval_read_string:463-472 verbatim */;

    let value = match wat_edn::from_json_string(&s) {
        Ok(owned) => match edn_to_value(&owned, sym.types().map(|a| a.as_ref())) {
            Ok(v)  => read_json_outcome_value(v),
            Err(e) => read_json_outcome_malformed(&e.to_string(), sym),
        },
        Err(e) => read_json_outcome_malformed(&e.to_string(), sym),
    };
    Ok(TrackedValue::new(value, Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() }))
}
```

**The cause.** `read_outcome_malformed` builds its `Error` from `ParseError::error_edn()` because
`ParseError` impls `WatError` (`parser.rs:15`). **`JsonError` CANNOT** — it lives in the `wat-edn`
crate and the trait lives in `src/to_edn.rs`. Use **`FlatMessage`** (`to_edn.rs:361`), the existing
adapter that lifts a message into the `Error` floor, then reuse
`read_outcome_malformed`'s decode tail (`edn_shim.rs:424-435`) unchanged.

```clojure
;; types.rs — mirror ReadOutcome's registration, and carry over its "why Error, not a
;; JSON-specific enum" reasoning: serde's variants in every caller's exhaustive match would be
;; arms nobody branches on. Discrimination lives in the navigable causes tree.
(:wat::core::defenum :wat::edn::ReadJsonOutcome :wat::enum::Pure
  :Value     [value <- :wat::core::Value]
  :Malformed [cause <- :wat::core::Error])
```

Purity: **Pure**, and for `ReadOutcome`'s stated reason — the payload holds no fd and no peer, and
`:wat::core::Error` is Record-natured. Marking it Impure would bar it from pure aggregates and the
wire for nothing.

## BLAST RADIUS

`src/edn_shim.rs`, `src/types.rs`, `src/runtime.rs`, `src/check.rs`, plus the gate. **No new
crate deps** — `serde_json` is already `wat-edn`'s. **Do not touch `wat_edn`** — `from_json_string`
is complete; you are consuming it. **Do not touch `:wat::edn::read`** — its raise is known,
documented debt with its own blast radius, and is explicitly out of this stone.

## RED GATE — write it FIRST, watch it fail, then build

A `.wat` gate (loader-gated under `wat-scripts/scratch-pad/`, or a `tests/` probe — your call,
state which). Three assertions, and **the third is the load-bearing one**:

1. **decodes** — `(:wat::edn::read-json "{\"edn\":\"42\"}")` → `::Value`.
2. **CRUX-1, the thing this gate exists to resolve** — a **nested field is readable from wat**.
   Reach into the decoded value and pull `"edn"` back out as a String. `edn::read` returns typed
   via the registry, `read-foreign` returns dynamic `ForeignRecord` for unknown tags
   (`value/value.rs:225`); a bare JSON object matches neither obviously. **Whatever it decodes to,
   REPORT THE SHAPE** — the MCP loop cannot be briefed until this is known.
3. **a malformed line leaves the caller ALIVE** — `(:wat::edn::read-json "{not json")` →
   `::Malformed`, **and then evaluate a form afterwards and assert its result**. Absence of a crash
   is not evidence; the surviving evaluation is.

Then RED-PROVE it: make the `::Malformed` arm raise instead, confirm assertion 3 goes red, revert.
A gate that cannot fire proves nothing (R59).

## STOP TRIGGERS — rejection criteria. Surface and halt; do not improvise.

- **STOP-1:** the decoded JSON object is NOT field-addressable from wat by any existing accessor.
  Report what it decoded to (the `Value` variant) and STOP. Do NOT invent an accessor, and do NOT
  reach for `read-foreign` semantics — that is a design decision for the orchestrator.
- **STOP-2:** `FlatMessage` does not fit the `Error`-floor construction. Report the mismatch; do
  NOT hand-roll an `Error` shape or stringify the cause into a message-only value.
- **STOP-3:** you find yourself editing `crates/wat-edn/` or `:wat::edn::read`. Both are out of
  scope; stop and report why you believe it is needed.

## RULES

- Weigh nothing yourself beyond the gate + `cargo build --release`; the orchestrator runs the
  floor and commits. **Do not commit, do not push.**
- `--check` is NOT a complete red arbiter — an unknown callee defers to a runtime
  `UnknownFunction` (demonstrated this session: `--check` returned 0 on a nonexistent verb). Prove
  the verb exists by RUNNING the gate.
- Minimum diff. No reformatting, nothing adjacent.

## REPORT BACK

1. The gate's output for all three assertions, verbatim.
2. **The decoded shape from assertion 2** — this resolves CRUX-1 and unblocks Stone 2.
3. The red-proof: what you broke, the failure it produced, and confirmation you reverted.
4. Any STOP.
Be honest about anything you were unsure of rather than presenting a guess as a finding.
