# Arc 291 — Strike 3b: `stop → resp` decouple (the out-locus mirror of `:init`)

**Status: ✅ SHIPPED (`4962e925`, weighed pure — SET-diff ∅).** Closes the DESIGN §3 decouple. `:init` builds State from EDN args **in**-locus;
`:stop` projects the final State to a serializable `resp` **out**-locus. Symmetric; mirror of strike 2.

## The contract (pinned — Option A, builder-confirmed)

- defservice grows a **`:stop` opt** — `(fn [s <- :State] -> :Resp <body>)` projecting the final State to
  the stop return value. **Single-arg, graceful-only.** No `reason` parameter: the normal-vs-crash
  distinction is **structural** (a graceful stop yields `LineageUp::Final(resp)`; a crash closes the channel
  → `:Closed`/`:Lost`, the already-built arc-272 crash-surfacing) — never a flag. "Deliver state on crash" is
  an incoherent contract (a crash may have corrupted State / killed the locus); the crash signal is the
  *close*, not a value. So `:stop` runs ONLY on the graceful path, where the service is healthy.
- **Default (no `:stop`): identity** — `(fn [s <- :State] -> :State s)`, so `Resp = State` and
  `LineageUp::Final(state)` is unchanged. Preserves "return value IS final state" for EDN states
  (counter/locus-parity stay green untouched).
- **The decouple bites on process/remote** (the peer already does thread=crossbeam-values /
  process=EDN-frames, fully wired — 3b does NOT manufacture EDN). A **non-EDN State** (arc-290 `LruCache`)
  can't cross a process wire; `:stop` lets the **State stay in-locus** and an EDN `resp` ride out. For thread
  the value rides regardless; the contract is uniform (project always; default = the State itself).

## The mechanism (grounded — mirror of `:init` at `service.wat:129-151`)

`:stop` fn-node structure `(fn [params] -> :RetTy body)` → `ast->children = [fn,params,->,:RetTy,body]`
(same as `init-fn-ch`). So:
- `stop-fn-node` = `(if (opts-map has "stop") (get …) \`(fn [~s-sym <- ~state-ty] -> ~state-ty ~s-sym))` (default identity).
- `stop-fn-ch` = `ast->children`; `stop-params-vec` = `(first (drop ch 1))`; **`resp-ty` = `(first (drop ch 3))`** (index 3 = `:RetTy`); `stop-body` = `(first (drop ch 4))`.
- `stop-project-name` = `:<fqdn>::stop-project` (distinct from the `<fqdn>/stop` METHOD).
- `stop-project-def` = `\`(defn ~stop-project-name ~stop-params-vec -> ~resp-ty ~stop-body)` — emitted at BOTH
  tier sites (process `service-forms-def` + thread top-level `do`), exactly like `init-def`.

Then thread `resp-ty` through the three sites the State return currently flows:
1. **`LineageUp` defenum** (`service.wat:~256`): `:Final [state <- ~state-ty]` → `:Final [resp <- ~resp-ty]`.
2. **serve `Admin::Stop` arm** (`~532`): `(send' self (~lineage-final-kw state))` →
   `(send' self (~lineage-final-kw (~stop-project-name state)))` (project in-locus before sending).
3. **`<fqdn>/stop` method** (`~669-678`): `-> ~state-ty` → `-> ~resp-ty`; the match arm
   `((~lineage-final-kw state) state)` → `((~lineage-final-kw resp) resp)` (binder rename; returns resp-ty).
- **known-opts** (`:78`): add `"stop" true`; extend the unknown-option error message `… :record-parent :init`
  → `… :record-parent :init :stop`.

## RED probe (committed, verify-first)
`wat-tests/service-stop-resp.wat` — a counter with `:stop (fn [s <- :State] -> :i64 (State/count s))`
projecting State → a **distinct** return type (`:i64`, not `:State`). `(<svc>/stop h)` returns `7` (an i64),
both tiers. **RED at HEAD:** `:stop` is an unknown trailing option → `macro-error "unknown trailing option
:stop"`. GREEN once `:stop` is supported and stop returns the projected resp.

## Blast radius
`wat/service.wat` only (+ the RED probe un-ignore). **Pure wat — ZERO Rust edits** (the peer/wire already
carries thread=value/process=EDN; `resp-ty` is just another type threaded through the macro). Do NOT touch
`wat/spawn.wat` (launch is resp-agnostic), any `src/*.rs`, or any other arc's files.

## STOP triggers
1. STOP if `resp-ty = (first (drop stop-fn-ch 3))` doesn't yield the fn's declared return type (the `:init`
   pattern says it should — index 3) — report the actual `ast->children` shape.
2. STOP-back-compat: `service-locus-parity.wat` + `service-init-parity.wat` + `service-admin-facet.wat` MUST
   stay green (default identity = `Resp=State`, so `Final(state)`/stop-returns-State is unchanged for them).
3. STOP if making `:stop` work needs a `spawn.wat`/Rust change — the design says pure-wat-macro; surface the gap.

## Expectations (scorecard)
| what | command | expected |
|---|---|---|
| RED probe green, both tiers | `cargo test -p wat --test test stop_resp` | 2 passed (after un-ignore) |
| back-compat (default identity) | `cargo test -p wat --test test counter_on` | 4 passed |
| owner-only stop still green | `cargo test -p wat --test test admin_stop` | 2 passed |
| pure wat | `git diff --name-only` | only `wat/service.wat` + `wat-tests/service-stop-resp.wat` |
| no new regressions | orchestrator: `cargo test -p wat --no-fail-fast`, SET-diff vs HEAD | ∅ (the deporder flap aside) |

Runtime: 20–35 min (pure-wat macro mirror of strike 2). Trap-door: the `resp-ty` index-3 extraction (STOP-1)
— everything else is transcription from the `:init` bindings + the existing Final/stop-method sites.
