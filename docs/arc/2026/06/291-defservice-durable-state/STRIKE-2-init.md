# Arc 291 — Strike 2: the `:init` keystone (build the soul in-locus)

**Goal.** Make the RED probe `wat-tests/service-init-parity.wat` go GREEN on both tiers by adding an
`:init` callback to `defservice`: `start` takes an EDN seed, and the service builds its `State` **in the
locus** (thread: in the spawned thread; process: child-side after `recv'`ing the EDN seed). This unblocks
arc 290 (non-serializable cache state hosted in-locus) and is the keystone of the 291 prophecy.

**One contract decision (pinned).** `init` is **always emitted** and **always run in-locus by `launch`**,
unifying the default and the `:init` cases:
- **no `:init`** (back-compat (i)): emit `init = identity over State`; `start [locus state0 <- :State]`
  UNCHANGED; the locus applies identity. `service-locus-parity.wat` stays green untouched.
- **`:init (fn [arg <- :T] -> :State body)`**: emit that fn; `start [locus arg <- :T]` (the init param
  becomes start's 2nd param); the locus applies it to the shipped EDN value to build State.
The wire carries the **ship value** (a `State` for default — EDN for the counter; the EDN `arg` for
`:init`); `init` runs on it in-locus. **`init` is single-arg** (one value crosses the wire; multi-field
init → pass a record). State of a resource service (LruCache) is built child-side, never shipped.

## The mechanism (grounded; this is the algorithm)

`launch` already dispatches per-tier and is the only place tier differs. Today it takes a pre-built
`state0` and hands it to `serve`. The change: `launch` takes a **ship value + the init fn by name** and
runs `(init ship)` in-locus before `serve` — exactly how `serve` itself is passed by name and `apply`'d.

## Rooms — read in order (exact file:line; do NOT hunt)

1. **`wat-tests/service-init-parity.wat`** (whole file) — THE WORKED REFERENCE + the acceptance test.
   It is `service-locus-parity.wat` + one `:init` clause. Your kill = remove the two
   `(:wat::test::ignore …)` lines and make both deftests pass.
2. **`wat/service.wat:74-76`** — `known-opts` (currently only `"record-parent"`). Add `"init" true`.
3. **`wat/service.wat:87-113`** — the opts-map fold + per-option `get`. After it, add an `init` extraction
   (mirror `state-parent` at `:109-114`): `init-fn-node = (HashMap/get opts-map "init")` if present, else
   the default identity fn node.
4. **`wat/service.wat:119-120`** — `state-ty` mint (`:<fqdn>::State`). You'll reuse it.
5. **`wat/service.wat:629-633`** — `launch-head-kw` builds `wat::spawn::Locus/launch<Op,Reply>`. Change to
   `…/launch<Op,Reply,State>` (add the State type-arg; see STOP-1).
6. **`wat/service.wat:654-666`** — `child-main-form`. Today: self-peer `~addr-ty ~state-ty` (line 658),
   `cm-st (recv' self)` (661), `apply serve … cm-st`. Change: self-peer R becomes the **ship type**;
   `recv'` the ship value; build `st` via `(apply -> :<State> :<fqdn>::init <ship> [])`; apply serve with
   that `st`. The init fn name is emitted into service-forms (step 8), so the child can call it by name.
7. **`wat/service.wat:674-684`** — `service-forms-def` (the forms shipped to the process child). Add the
   emitted `:<fqdn>::init` defn to this `forms` block (so the child can call it), alongside `~child-main-form`.
8. **`wat/service.wat:686-693`** — `start-params` + `start-body`. `start-params` becomes
   `[locus <- :wat::spawn::Locus  <init-param>]` (the init fn's single param). `start-body` passes the
   ship value + init-by-name: `(launch<Op,Reply,State> locus <init-param-ref> :<fqdn>::init-keyword
   serve-keyword (service-forms))`.
9. **`wat/service.wat:717-728`** — the final `do`. Emit the `:<fqdn>::init` defn here too (top-level, so
   the THREAD tier's `launch` can `apply` it by name — serve lives in the parent universe for thread).
10. **`wat/spawn.wat:207-211`** — the `Locus/launch` protocol decl. Change signature to
    `launch<S,R,St,Sh> [self  ship <- :Sh  init <- :wat::core::keyword  serve <- :wat::core::keyword
    service-forms <- :wat::core::Vector<wat::WatAST>] -> :wat::spawn::Launched<S,R>` (was
    `[self state0 <- :St serve service-forms]`).
11. **`wat/spawn.wat:220-230`** — ThreadOpts `launch` impl. The serve closure must build state in-thread:
    replace the captured `state0` arg to `apply serve` with `(:wat::core::apply -> :St init ship [])`.
12. **`wat/spawn.wat:240-250`** — ProcessOpts `launch` impl. Ship the EDN value: `send' svc ship` (was
    `send' svc state0`). The child builds State via init (step 6).

## Implementation sketch (fill it; don't reinvent the shape)

```clojure
;; service.wat — init fn node (after opts-map, near :109):
;;   default identity uses symbol-node hygiene for the `s` binder (like the existing _/r binders).
init-fn-node (if (HashMap/contains-key? opts-map "init")
               (HashMap/get opts-map "init")
               `(:wat::core::fn [~s-sym <- ~state-ty] -> ~state-ty ~s-sym))   ;; identity default
init-name    (keyword/from-string "{fqdn-str}::init")
init-def     `(:wat::core::defn ~init-name ~(fn-params init-fn-node) -> ~state-ty ~(fn-body init-fn-node))
init-param   ;; the single [name <- :T] binder triple from init-fn-node's params (becomes start's 2nd param)

;; start (service.wat:686):
start-params `[locus <- :wat::spawn::Locus  ~@init-param]
start-body   `(:wat::core::let
                [~lr-sym (~launch-head-kw locus ~ship-ref
                           (:wat::core::keyword/from-string ~init-name-str)
                           (:wat::core::keyword/from-string ~serve-name-str)
                           (~service-forms-kw))]
                (~handle-name (Launched/handle ~lr-sym) (Launched/address ~lr-sym)))

;; spawn.wat thread impl (220):
(launch [self ship init serve service-forms]
  (let [b  (listener' self :S :R)
        sp (spawn-program' self
             (fn [self-peer <- Peer'<R,S>] -> nil
               (apply -> nil serve self-peer (Bound/listener b)
                 (Vector Peer'<R,S>) (apply -> :St init ship []) [])))]
    (Launched/new sp (Bound/address b))))

;; spawn.wat process impl (240): … _ (send' svc ship) …   ;; ship the EDN value, child runs init
```

## Blast radius (bound it)
`wat/service.wat` + `wat/spawn.wat` + `wat-tests/service-init-parity.wat` (un-ignore). **Pure wat — expect
ZERO Rust edits.** No new types. Do NOT touch any other `wat/*.wat`, any `src/*.rs`, or any other arc's files.

## STOP triggers (halt + report the gap; do NOT improvise a workaround)
1. **STOP-1 (the type-param):** if `(apply -> :St init ship)` in the thread `launch` impl cannot bind `St`
   (the new State type-param) — e.g. the protocol won't accept 4 type-params, or `St` won't resolve from
   the explicit `launch<Op,Reply,State>` call — STOP and report the exact checker error. Do not fall back
   to building State in the caller (that breaks process in-locus).
2. **STOP-2 (back-compat):** if making the no-`:init` default work forces an edit to
   `service-locus-parity.wat`, STOP — the default must keep that file green untouched.
3. **STOP-3 (single-arg):** the contract is one ship value on the wire. If you find yourself needing
   `init` to take multiple wire args, STOP and report.
4. You are a LEAF. Do NOT spawn subagents. If the change exceeds these rooms, STOP and report what extra
   the foundation needs.

## Expectations (scorecard — written before the strike)

| what | command | expected |
|---|---|---|
| the probe goes green, both tiers | `cargo test --test test seeded` | `2 passed; 0 failed` (after un-ignore) |
| back-compat: locus-parity stays green | `cargo test --test test counter_on` | green (unchanged) |
| pure wat — no Rust touched | `git diff --name-only` | only `wat/service.wat`, `wat/spawn.wat`, `wat-tests/service-init-parity.wat` |
| no new workspace regressions | `cargo test --test test 2>&1 \| tail -3` | failing-test SET ⊆ HEAD's (the ~known floor; weighed by the orchestrator) |

Runtime prediction: 15–30 min (a pure-wat macro + protocol change). Trap-door: the `apply -> :St`
type-param binding (STOP-1) is the one genuinely novel spot; everything else is transcription from the
worked reference + the existing `serve`-by-name pattern.
