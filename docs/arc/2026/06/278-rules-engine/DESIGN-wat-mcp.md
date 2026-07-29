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
(:wat::edn::read-json <json-string>) -> :wat::edn::ReadJsonOutcome
```

Structurally the twin of `eval_edn_read` (`edn_shim.rs:162`) — parse, then
`edn_to_value(&edn, sym.types())`. The only difference is the parser at the front.

### It returns an OUTCOME, not a raise

A malformed line arrives from a remote, untrusted harness. A raise would let one bad byte kill the
server — the exact failure `read-string` was converted to fix (`types.rs:952`: *"an arrow key at
the REPL sends ESC (0x1B) … the raise unwound THROUGH the loop and killed the session"*).

```clojure
(:wat::core::defenum :wat::edn::ReadJsonOutcome :wat::enum::Pure
  :Value     [value <- :wat::core::Value]
  :Malformed [cause <- :wat::core::Error])
```

Mirrors `:wat::core::ReadOutcome`, including why the cause is the structural `:wat::core::Error`
rather than a JSON-specific enum: lifting serde's variants into every caller's exhaustive match
hands them arms nobody branches on. Discrimination lives in the navigable causes tree.

**⚠ ADJACENT DEBT, named not fixed:** `:wat::edn::read` still RAISES (`edn_shim.rs:180`) — the
pre-LAW shape. `read-string` was converted; `edn::read` was not. Converting it is its own strike
with its own blast radius. This doc only refuses to add a second raiser.

**CRUX-1 (open, resolved by Stone 1's gate):** the `:Value` payload type. `edn::read` returns typed
via the registry; `edn::read-foreign` returns dynamic `ForeignRecord`/`ForeignVariant` for unknown
tags. An MCP envelope is plain JSON (objects, strings, ints), so the typed path should suffice —
but **whether a bare JSON object is field-addressable from wat must be PROVEN before Stone 2 is
briefed.** Do not brief a protocol on an unproven decode.

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

## STONES

Each: RED gate first, then the strike, weighed by the orchestrator's own `--release` re-run.

**Stone 1 — `:wat::edn::read-json` + `ReadJsonOutcome`.** Registration mirrors `write-json`; impl
mirrors `eval_edn_read`; outcome mirrors `ReadOutcome`.
 - RED gate: a well-formed JSON object decodes AND a nested field is readable from wat (this is
   what resolves CRUX-1); a MALFORMED line returns `::Malformed` and **the caller survives** —
   proven by evaluating a form afterwards, not by the absence of a crash.

**Stone 2 — `wat/mcp.wat`.** A stdlib MODULE exposing `(:mcp::serve defs)`, no `:user::main` — the
same split `wat/repl.wat` uses, for the same reason (a stdlib file declaring `:user::main` hands
one to every wat program).
 - RED gate: a scripted transcript — `initialize` → `tools/list` → `tools/call eval` ×2 →
   `tools/call reset` → `tools/call eval` — compared as **structured goldens**, never `.contains`.
   Must prove a definition from call N is visible at call N+1, and **invisible after `reset`**.

**Stone 3 — `Mode::Mcp` + shim.** Copies `Mode::Repl` (`argv.rs`, `mod.rs`): zero positionals, a
one-form entry calling `:mcp::serve`.
 - RED gate: `wat --mcp` end-to-end over a real pipe; `--mcp <path>` is EX_USAGE 64.

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
