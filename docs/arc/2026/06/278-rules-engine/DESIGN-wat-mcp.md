# DESIGN — `wat --mcp`: `wat --repl` with a JSON codec

> **The ask (builder, 2026-07-29):** *"build `wat --mcp` who is a json rpc for a harness with a
> single tool … eval"*, plus a `reset` that wipes the session fresh. **Full MCP: the handshake,
> `tools/list`, `tools/call`.**
>
> **The shape, stated first because everything below is detail:** this is `wat --repl` (`568cdf82`)
> with a JSON codec at each end. The loop, the state threading, and the evaluation semantics are
> unchanged and already proven. The new work is a codec and one missing verb.

## IT IS THE REPL LOOP

`wat/repl.wat`, shipped and gated:

```clojure
(:repl::turn defs)        → (match (read-frame)          Frame | Eof | Stopped)
(:repl::eval-and-loop …)  → (match (read-string text)    Forms | Malformed)
(:repl::eval-form …)      → (match (eval-with-defs! form defs)
                               Declared    → (turn (conj defs form))   ;; the ONLY growth
                               Evaluated v → println, (turn defs)
                               CheckFailed → println, (turn defs)      ;; session SURVIVES
                               Raised      → println, (turn defs))     ;; session SURVIVES
```

`--mcp` changes exactly two things:

| | `--repl` | `--mcp` |
|---|---|---|
| read | `read-frame` → text → `read-string` | `read-frame` → text → **decode JSON envelope** → dispatch method → `arguments.edn` → `read-string` |
| write | `println v` | **wrap the same value** in `{"edn":…}` inside a JSON-RPC response |
| `reset` | — | `(turn (:wat::core::Vector :wat::WatAST))` — `conj` inverted |

The four `FormOutcome` arms, the `defs` threading, and the survive-a-bad-line property are
**identical**. `wat --repl`'s gates already prove them.

## THE WIRE — JSON envelope, OPAQUE EDN payload

```json
{"edn":"#some.edn/Thing {:whatever 42}"}
{"edn":"(wat.core/defn user/some-fn [n <- i64] (wat.core/+ 0 n))"}
```

EDN text carried as a JSON string; JSON never represents a wat value. This is arc 278 Stone B's
ruling applied again — `Log.message` is an opaque EDN-text String so the carrier "never decodes"
(`feedback_sink_is_opaque_store_consumer_decodes`). It buys losslessness: keywords, rationals,
bigints and tagged records all survive, because the JSON layer only ever sees a string. No
`#rational`/`#bigint` sentinel games, no `write-json-natural` flattening, no round-trip debt.

## SESSION STATE

`defs` threads through the tail call, exactly as `:repl::turn` does. State lives for the process's
lifetime and dies with it.

**Ruled (builder):** stdio only; **no persistence across processes** — "spin the proc up, write to
it, it dies when the harness dies"; many instances run at once, one per harness. Concurrency is the
OS's — N harnesses ⇒ N processes ⇒ N isolated sessions.

Nothing is redefined per call: a definition made in one `tools/call` is live for every later one
(`wat --repl`'s `definitions_persist_across_turns` is the standing proof of that mechanism).

## WHAT IS ALREADY BUILT — measured, not assumed

| need | mechanism | evidence |
|---|---|---|
| read a JSON line off stdin | `:wat::kernel::read-frame` → `Frame text` | **MEASURED** (`probe-mcp-wire.wat`) — a JSON line comes back INTACT. Non-obvious: the frame scanner is EDN-AWARE (`repl.wat`: "continues only while the prefix is INCOMPLETE EDN"), and a JSON object opens `{` exactly as an EDN map does. |
| write `{"edn":…}` | `:wat::edn::write-json` on a String-keyed HashMap | **MEASURED** same probe — BARE keys (`{"edn":…}`, not `{":edn":…}`) |
| eval EDN text against a session | `:wat::eval-with-defs!` → `FormOutcome` | shipped arc 170 |
| the loop, as a LIBRARY | `(:repl::turn defs)` — `wat/repl.wat` is a stdlib module | shipped `568cdf82` |
| a CLI mode with its own arity contract | `Mode::Repl` | the precedent to copy |

## THE ONE SUBSTRATE GAP — `:wat::edn::read-json`

`wat_edn::from_json_string(s) -> JsonResult<OwnedValue>` is **fully implemented**
(`crates/wat-edn/src/json.rs:225`) and has **ZERO consumers in `src/`**. The write half was wired
(`runtime.rs:5307`); the read half never was. Everything else is composition.

```
(:wat::edn::read-json <json-string>) -> :wat::edn::ReadJsonOutcome<T>
```

Structurally the twin of `eval_edn_read` (`edn_shim.rs:162`) — parse, then
`edn_to_value(&edn, sym.types())`. The only difference is the parser at the front.

### It returns an OUTCOME, not a raise

A malformed line arrives from a remote, untrusted harness. A raise would let one bad byte kill the
server — the exact failure `read-string` was converted to fix (`types.rs:952`: *"an arrow key at
the REPL sends ESC (0x1B) … the raise unwound THROUGH the loop and killed the session"*).

```clojure
(:wat::core::defenum :wat::edn::ReadJsonOutcome<T> :wat::enum::Pure
  :Value     [value <- :T]
  :Malformed [cause <- :wat::core::Error])
```

Mirrors `:wat::core::ReadOutcome`, including why the cause is the structural `:wat::core::Error`
rather than a JSON-specific enum: lifting serde's variants into every caller's exhaustive match
hands them arms nobody branches on. Discrimination lives in the navigable causes tree.

**⚠ ADJACENT DEBT, named not fixed:** `:wat::edn::read` still RAISES (`edn_shim.rs:180`) — the
pre-LAW shape. `read-string` was converted; `edn::read` was not. Converting it is its own strike
with its own blast radius. This doc only refuses to add a second raiser.

**CRUX-1 — RESOLVED by Stone 1's gate (`b9d48a65`).** A JSON object decodes to a String-keyed
`HashMap` (wat-edn mints a keyword only when a JSON string opens with `:`, and MCP envelope keys
never do), and it IS field-addressable: `(:wat::core::HashMap/get m "edn")` → `"42"`, measured.

**PARAMETRIC, and the first draft of this doc was not — that error is instructive.** It specified
`:Value [value <- :wat::core::Value]`, the universal top, where UP is free and DOWN is CHECKED (R7).
The payload could be produced and never consumed; `HashMap/get` correctly refused an opaque `Value`
receiver, and the rider building it hit that wall and STOPPED. The fix was the idiom three lines
away in the same file: `edn::read` and `read-foreign` both declare a fresh type var so "the caller's
binding unifies with whatever dynamic value shape lands". The Rust side needed no change — only the
DECLARED type was wrong. **A type in a design is a claim and owes the same evidence as any other.**

## THE MCP SURFACE

Newline-delimited JSON-RPC 2.0 on stdio. Three methods.

- **`initialize`** → `protocolVersion`, `capabilities: {tools:{}}`, `serverInfo`
- **`tools/list`** → `eval` (`{"edn": "<string>"}`) and `reset` (`{}`)
- **`tools/call`** → dispatch on `params.name`, unwrap `params.arguments`

### `eval`'s result — the FormOutcome IS the answer

| arm | MCP result | payload |
|---|---|---|
| `Declared` | ok | the definition set grew; no value to show |
| `Evaluated v` | ok | `{"edn": "<v as EDN>"}` |
| `CheckFailed cause` | `isError: true` | `{"edn": "<Error tree>"}` — navigable to a real span |
| `Raised cause` | `isError: true` | same |

`CheckFailed`/`Raised` are **not** transport errors: they are successful tool calls reporting a
failed evaluation, and **the session survives them**. That is the whole reason a REPL's failures
must be values (R53), and `a_bad_line_does_not_end_the_session` is the existing gate for the
mechanism.

A JSON-RPC **error response** is reserved for envelope faults: malformed JSON, unknown method,
unknown tool, missing `arguments.edn`.

## STONES — ⚠ THE PLAN BELOW WAS SUPERSEDED BY THE BUILD. Read this block first.

**SHIPPED. `wat --mcp` works end to end**, gated by `tests/cli/wat_mcp.rs` (5 tests), floor
4180/4180 by the orchestrator's own `--release` re-run. What actually landed is SMALLER than the
plan, and the difference is the point:

**Stone 1 — `:wat::edn::read-json` + `ReadJsonOutcome`. DONE (`b9d48a65`), as designed.**

**Stone 2a — `value_to_json_natural` learns `Nature::Record`. NOT BUILT, and NOT NEEDED.** The
measurement is real and stands (`wat-scripts/scratch-pad/probe-json-natural-record.wat`: a struct
emits bare-keyed JSON, a record falls to the `#tag`/`body` sentinel) — it is a genuine substrate
gap and `write-json-natural` has ZERO callers, which is why it sat unobserved. But it is **not on
this path**, and thinking it was cost most of a session.

> **THE RULING THAT KILLED IT (builder, 2026-07-29):** *"edn string in, edn string out … the wat
> side evals edn."* The payload is an EDN **string**. It arrives as a string, it leaves as a
> string, and **no wat value is ever converted to JSON.** A record crossing the wire is EDN text —
> `#some.ns/Rec {:field "val"}` — sitting inside a JSON string slot as characters. There is no
> record-to-JSON problem because records never become JSON.
>
> The envelope is a CONSTANT with two holes (the echoed `id`, and `result.content[0].text`). It is
> a template, not a structure to build. Stone 2a existed because a prior self assumed wat would
> *assemble* the reply from wat aggregates — and it cannot anyway: `HashMap/assoc` is homogeneous
> (measured, `probe-mcp-reply-emit.wat`), so a heterogeneous envelope is unbuildable as wat data.

**Stone 2b — `wat/mcp.wat`. NOT BUILT, and should not be.** There is no wat file. The loop is
`src/distribution/mcp.rs`.

> **WHY, and it is the substrate's own doctrine rather than convenience:** JSON is not EDN, and
> wat's stdin/stdout are strict-EDN data channels by construction (R51, typed-Unix). A wat
> `println` EDN-*encodes* what it is handed — printing a JSON frame delivers
> `"{\"jsonrpc\":\"2.0\"…}"`, an escaped string literal, not a JSON object (MEASURED). That is the
> channel correctly refusing a foreign format. The bridge therefore belongs at the transport,
> beside argv and the frame reader.
>
> The SEMANTICS are not duplicated: `runtime::eval_form_against_defs` (factored out of
> `:wat::eval-with-defs!`) is called by both the wat verb and the MCP loop, so `--repl` and `--mcp`
> cannot drift on classification, on which arm grows the definition set, or on what a failure is.

**Stone 3 — `Mode::Mcp`. DONE**, mirroring `Mode::Repl`: zero positionals, `--mcp <path>` is
EX_USAGE 64.

### The gate, and why it is not vacuous
`definitions_persist_across_turns` · `reset_empties_the_session` (asserts the call works BEFORE and
fails AFTER, so a no-op reset cannot pass) · `a_failed_evaluation_is_not_fatal` ·
`the_payload_is_edn_not_json` (structural `.edn` golden) · `mcp_rejects_a_positional`.
**PROVEN by a deliberate break** (R59 `NISI FRANGAS, NIHIL PROBAS`): cutting the one line
`session.defs.push(form)` turned 3 of the 5 RED, including the load-bearing one.

### ⚠ A DEFECT FOUND BY THE GATE — the session render loses record field names
A record returned from a session comes back `#usr/Point {:field-0 3 :field-1 4}` — the declared
`:x`/`:y` are gone. MEASURED as a SESSION-path defect, not a record-path one, and **inherited, not
introduced**:

```
ordinary program :  #usr/Point {:x 3 :y 4}       ← correct
session (--repl) :  #usr/Point {:field-0 3 :field-1 4}
```

`field-N` is the fallback when the type is absent from the `TypeEnv` (`edn_shim.rs:3635`). The
value is produced inside the per-turn frozen world — which knows the type — and rendered later
against a symbol table that never saw the `defrecord`. So it is a **shipped defect in `wat --repl`**
(arc 170's closure condition) that `--mcp` inherits faithfully. It matters more here: a human can
shrug at `:field-0`; an LLM has been handed a shape it cannot write back. The fix belongs in the
session path so both modes get it, and it touches `eval_form_against_defs`'s contract — the
builder's call, tracked, not taken unilaterally. The golden
(`tests/cli/wat_mcp__record.edn`) captures the current behaviour deliberately, so the day it is
fixed the gate goes red and the correction is explicit rather than silent.

### Still standing on a claim, not a run
`PROTOCOL_VERSION` and the `result` envelope shape come from the MCP specification and have **not**
been measured against a live harness. Pointing a real client at it is what settles them.

## WHAT MAKES THE GATES NON-VACUOUS

An MCP gate is trivially faked — assert exit 0 and you have proven a binary starts (R59: 4105/4105
green over a protocol that had never executed). The load-bearing pair, both red-provable by cutting
one line:

- **a definition from an earlier `tools/call` is visible in a later one** — kill the `Declared`
  arm's `conj` and it must go red, exactly as it does for `wat --repl`;
- **`reset` makes it invisible again** — the same mechanism inverted, so a no-op `reset` cannot pass.

## OUT OF SCOPE (affirmatively cut)

- HTTP/SSE transport — stdio only.
- Session persistence across processes — explicitly ruled out; the process IS the session.
- A `defservice` for the session — the loop is proven, shipped, and gated; a service would add
  ceremony for state that dies with its process and is never contended.
- Converting `:wat::edn::read` to an outcome — adjacent debt, named above.
- Structural JSON↔wat value mapping — the payload is opaque EDN text BY DESIGN; re-opening it
  re-introduces every lossy-mapping problem the opaque carrier avoids.
