# SCORE — STONE N RELAND 1: the generated `::Op::` send

No commit. Floor left to the orchestrator. Lands on N's uncommitted tree.

## THE SITE — expanded, not grepped

`defsurface` does **not** emit the construction. `wat/service.wat:2080` already had `(~op-variant-kw {:req req})`. That is the **service-namespaced** client (`<fqdn>/<op>`). Corpus fixtures call the **surface** name (`:wat::kernel::StdOut/write`).

That path is **Path B** in `src/runtime.rs` (~3144): a synthesized send-then-recv AST. The Op construction was:

```
(:S::Op::<Variant> __req)     // positional — retired
```

Comment at 3147 even documented the old form. This is the mechanism `:S/method` actually runs through (the comment says so). `synthesize_surface_protocol` (`src/types.rs:2563`) only builds the **type** (`::Op` / `::Reply` enums); the **construction** is Path B.

STOP-1 held: no grep as diagnostic. STOP-2: `src/` change, the emitter only.

## What changed

`src/runtime.rs` Path B AST:

- Op: `(:S::Op::V {:req __req})`
- RecvOutcome::Message / Lost bodies: `{:msg …}` / `{:cause …}` (same synthesized form; positional Message would have refused as soon as Op was fixed)
- over-budget RTL: `(:Response::RequestTooLarge {:bytes n :cap cap})` inside `{:msg …}`

`src/services/verbs.rs` `read_via_stdin`: Frame field named `text`, and if that field is a HashMap `{:text s}` (map-as-payload leftover) take the string. Needed for `readln` after Path B's Message wrap; without it Frame's field was `HashMap {:text "[\"hello\"]"}`.

## Expectation: `readln` lives

**PASSED.**

```
echo '["hello"]' | ./target/release/wat /tmp/n-r1/rl.wat   # EXIT=0, printed the line
./target/release/wat /tmp/n-r1/ok.wat                      # println "ok" EXIT=0
```

The rename/wrap driver can read a worklist again.

## Codemod re-run + idempotence (STOP-4, STOP-5)

**PASSED.** Wrap over 1040 `tests/**/*.wat` (the `:probe::` population and the rest of tests). No hand-edits of probe files.

- Pass 1: EXIT=0
- Pass 2: EXIT=0, **sha256 of every tests `.wat` unchanged** (diff empty, 0 lines)

UNRESOLVED noise on non-ctor heads (`:wat::core::defenum`, `match`, `forms`, `do`, `~init-name`) — arity mismatch, not wrapped.

## Probe / clippy

- `probe_arc296_enum_map_ctor`: **5 passed**, `#[ignore]` = 0
- `cargo clippy --release --all-targets --workspace`: **0 errors**, 5 pre-existing dead-code warnings

## Floor delta per population

**Not run.** Orchestrator. Pre-reland (centrally measured): 473 failed, all `positional variant construction is retired`, split 117 StdOut::Op::Write / 31 Store::Op::EnsureSchema / 27 StdIn::Op / 38 EchoResponse::Ok / 36 probe::Outcome / 111 match-downstream. This strike removes the Path B Op send (the 117+27+31 surface-method calls). Probe wrap should take the Echo/Outcome tests. Exact post-reland counts are the floor's.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 grep as diagnostic | **held.** Path B read in `runtime.rs` |
| STOP-2 src/ change authorized | **held.** Emitter only (+ Frame reader for the map-as-payload leftover) |
| STOP-3 wall weakened | **held.** |
| STOP-4 idempotence unmeasured | **held.** Second wrap: 0 bytes changed |
| STOP-5 probe hand-edited | **held.** Codemod |

## Residue

- Frame text arriving as `HashMap {:text s}` at the rust `read_via_stdin` boundary — accepted by named-field + map unwrap. Finding about map-as-payload of `ReadFrameOutcome::Frame`, not folded into this stone.
- Wrap-script Option matches sometimes see `Value::Enum` not native `Option`; wildcards added so the tool runs. Same class as N's Option spelling split.
