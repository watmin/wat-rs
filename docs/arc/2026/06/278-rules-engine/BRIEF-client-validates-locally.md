# BRIEF — the client validates locally, and a bounced request never waits for a reply

**Design: `DESIGN-STONE-the-client-validates-locally.md` — RULED, nothing open.** Read it first and do
not re-derive it. This brief carries the sites (re-grounded today; the stone's own line numbers had
drifted) and the gates.

**Floor:** `4344 tests run: 4344 passed, 262 skipped` — my own `--release` re-run at `824feb14`.

## The work, in one sentence

`defservice`'s generated client method must refuse an over-budget request **locally**, against the
limit the surface already publishes, and must **not** `recv` after a send it did not face.

## ⛔ IT IS ONE STRIKE, NOT TWO

Fixing the budget alone converts a deadlock into a silent wrong answer. Both halves land together in
the same generated method or neither does.

## Read in order

1. **`DESIGN-STONE-the-client-validates-locally.md`** — the whole design, including why the client
   checks the *surface* limit and never the deployment's.
2. **`wat/service.wat:1368`** — `op-methods`, the generator. The defect is at **`:1432-1436`**:

   ```clojure
   [~discard-sym (:wat::core::match (:wat::kernel::send c (~op-variant-kw req))
                    … three arms …)
    ~r-sym (:wat::kernel::recv c)]
   ```

   `discard-sym` is `(symbol-node "_")` and all three arms return `nil`, so the method recvs
   regardless of whether the send landed.
3. **`src/types.rs:3033-3050`** — `build_op_budget_constants`, called at `:3132` at **defsurface**
   registration. It emits one `(def :<Surface>::<OP>-MAX-REQUEST-BYTES <n>)` per serviceable op, op
   name **upper-cased** (`:probe::Cap1` + `do-op` → `:probe::Cap1::DO-OP-MAX-REQUEST-BYTES`). Field
   members are skipped. The constant is **surface-scoped**, so the client already holds it — nothing
   to thread, negotiate, or put on the wire.
4. **`src/types.rs:1081` and `:2686-2690`** — `RequestTooLarge` is MANDATED on every response enum,
   with a required shape. So the local refusal returns the same value the server would have sent, and
   the caller's match does not change.
5. **`tests/services/probe_arc278_service_max_frame_bytes.wat`** — the fixture that already declares
   both limits, and where the server-side path is asserted.

## The shape

```
op-method(c, req):
  measure the encoded req against <Surface>::<OP>-MAX-REQUEST-BYTES
  over  -> RecvOutcome::Message(<Op>Response::RequestTooLarge{bytes, cap})   ; no send, no recv
  under -> send -> ACT on the outcome -> recv only on the arm that landed
```

## ⛔ STOP-1 — the gate needs a DISCRIMINATOR or it proves nothing

A fat request refused locally and a fat request refused by the server produce the **same value** at
the caller — that is the design working (locality is invisible). So a test that only asserts
`RequestTooLarge` passes identically whether your code ran or not.

**Assert that nothing was sent.** The peer's receiver must have nothing pending after the refusal.
If you cannot construct that assertion, STOP and report — do not ship a gate that cannot tell the two
apart.

## ⛔ STOP-2 — do not replace one uniform match with another

Three arms returning `nil` is what caused the deadlock. Three arms that look different but all still
fall through to `recv` is the same defect wearing a new coat. Each arm must reach a *different*
place: only the landed arm recvs.

## ⛔ STOP-3 — the budget is a property of the WIRE

It applies wherever `send_wire` is the path. The thread tier hands a Rust value across shared memory
and **never encodes**, so there is no frame to be too large for and no byte count to measure. Do not
branch on `locus == process`; branch on whether there is a wire.

## ⛔ STOP-4 — check the surface limit, never the deployment's

`:max-request-bytes` is declared on the **surface** and both sides know it. `:max-frame-bytes` (FOO)
is the **service's**, may be stricter, and a dialer cannot predict it — `smallfoo` in the fixture
above proves a deployment can be tighter than the contract. A FOO violation stays a server-side
dismissal. Do not delete or weaken that path: the client honours the contract, FOO defends against
liars.

## ★ GROUND THIS, DO NOT ASSUME — there is a SECOND site with the same shape

`wat/service.wat:1478-1482` generates the admin `stop` method with the identical
`[_ (match (send …) …) r (recv …)]` pattern (`stop-discard-sym` / `stop-r-sym`).

The budget half plainly does not apply there — `Admin::Stop` carries no user payload. **Whether the
fall-through half applies is a question for you to answer by reading it**, not for me to assert.
Report what you find. If it is the same defect, say so and leave it — a second site is a scope call,
not a bonus.

## ★ THE DELIBERATE BREAK

Break the local check (let the over-budget request through) and confirm the new gate goes **RED** —
and confirm it reddens on the *discriminator*, not merely on the response value. Restore byte-exact,
confirm green. Report both with real output.

## Done means

- An over-budget request is refused by the client, with nothing sent, proven by the discriminator.
- A send that did not land does not lead to a `recv`.
- No new type minted; `RequestTooLarge` carries the refusal.
- The server-side FOO rejection still passes.
- One deliberate break, RED output shown, restored byte-exact.
- `cargo nextest run --release` **Summary verbatim** against `4344/4344/0/262`, any delta explained.
- `cargo clippy --release --all-targets` clean.
- Every STOP hit named; if none, say so.

## Notes that will save you time

- **`macroexpand` the generated method before theorising.** A confusing error out of a `defservice`
  form is almost never the macro's logic — it is something the macro emitted that nobody has looked
  at. Dump the expansion and read the actual emitted names. (A `defservice` cannot be
  runtime-macroexpanded; expand at the form level.)
- `target/release/wat --check <file>` is the fast (~0.2s) per-file arbiter. It is **not** complete —
  an unknown callee defers to a runtime `UnknownFunction` — so the test run is the arbiter for
  anything about a missing verb.
- Line numbers in this arc's docs drift. Every one above was re-grounded today; re-ground again if
  something does not match.

Run every verification in the FOREGROUND and block on it — your turn ends when the numbers are in
your hands, not when the command is launched. Do not commit, push, or stash.
