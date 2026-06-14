# BRIEF — Stone C.2: defservice generates `Reply` + `serve` (RPC dispatch loop, gen_server result)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
operate ONLY here; `git -C /home/watmin/work/holon/wat-rs`; ignore any `.claude/worktrees/` path).
The RED probe is on disk + verified RED (`tests/probe_arc209_c2_defservice_dispatch.rs` →
`UnresolvedReference :my::counter::serve`). Do NOT commit — the Inquisitor weighs. Run `cargo test`
PLAINLY (no setsid/timeout). Stale rust-analyzer diagnostics can contradict a clean build — trust it.

## The work in one paragraph

Extend the `:wat::service::defservice` macro in **`wat/service.wat`** so its expansion goes from a bare
`defenum` (C.1, the request enum `<fqdn>::Op`) to a **`(:wat::core::do (defenum <fqdn>::Op …) (defenum
<fqdn>::Reply …) (defn <fqdn>::serve …))`**. C.1 already folds `:ops` into the `Op` variants — KEEP that
walk. C.2 ADDS, from the SAME clauses: the **`Reply`** enum (variant per op = the op's OUTPUT fields)
and **`serve`** (the `poll'`/`ServiceEvent` dispatch loop that owns the live `:state`, decodes an `Op`,
runs the inline handler body — which returns a `:wat::service::Outcome` — `send'`s the reply, TCO-recurs).
**SINGLE FORMAT:** every op clause is the RPC shape `(:Op [s <- :State …in] -> [..out fields] body)`.
There is NO dual-format / format-detection — the C.1 probe has been migrated to this shape; do NOT add
a compatibility branch for the old `(:Tuple :State :T)` return (hard-cut, no shim).

## The handler contract (gen_server result — `Outcome` ALREADY EXISTS)

`:wat::service::Outcome<S,R>` is a stdlib type (already on disk, `wat/service.wat`):
`(:wat::core::defenum :wat::service::Outcome<S,R> :Reply [state <- :S reply <- :R])`. A handler is a
PURE transform: `(s <- :State, …in-args) -> :Outcome<:State, <fqdn>::Reply>`, body returns
`(:wat::service::Outcome::Reply new-state <reply>)` where `<reply>` is a `<fqdn>::Reply::<Op>` value the
body constructs. (`serve` matches the `Outcome` — when C.4 adds `:NoReply`/`:Stop`, serve's Outcome-match
grows arms; no reshape.) NO `&` rest in any argvec (fixed-arity records; STOP if `&` appears).

## Op-clause shape — children `[opkw, in-argvec, ->, out-fieldvec, body]`

- `ch[0]` `opkw` (`:Get`/`:Increment`) · `ch[1]` `in-argvec` `[s <- :State n <- :i64]` (1st triple
  `s <- :State` = state-self; rest = input fields = Op variant fields, C.1) · `ch[2]` `->` ·
  `ch[3]` `out-fieldvec` `[value <- :i64]` (= Reply variant fields, NEW) · `ch[4]` `body` (returns `Outcome::Reply`).

## The EXACT target expansion (the counter — your template; generalize over ops)

```clojure
(:wat::core::do
  ;; C.1 (KEEP) — request enum: variant per op = input fields (state-self dropped)
  (:wat::core::defenum :my::counter::Op
    :Get
    :Increment [n <- :wat::core::i64])

  ;; C.2 (NEW) — response enum: variant per op = the out-fieldvec (ch[3]) verbatim
  (:wat::core::defenum :my::counter::Reply
    :Get       [value <- :wat::core::i64]
    :Increment [value <- :wat::core::i64])

  ;; C.2 (NEW) — the dispatch loop. self/l/clients param TYPES mirror probe_arc209_c0b1b's
  ;; `serve` EXACTLY, substituting I=:my::counter::Op, O=:my::counter::Reply; + state <- <:state type>.
  (:wat::core::defn :my::counter::serve
    [self    <- :wat::kernel::Peer'<my::counter::Reply,my::counter::Op>
     l       <- :wat::kernel::Listener'<my::counter::Op,my::counter::Reply>
     clients <- :wat::core::Vector<wat::kernel::Peer'<my::counter::Reply,my::counter::Op>>
     state   <- :wat::core::i64]
    -> :wat::core::nil
    (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
      (:wat::kernel::ServiceEvent::Shutdown nil)
      ((:wat::kernel::ServiceEvent::Connection peer)
        (:my::counter::serve self l (:wat::core::conj clients peer) state))
      ((:wat::kernel::ServiceEvent::Message idx op)
        ;; GENERATED: one arm per op. Bind `s`=state, run the inline body (refs s + variant args)
        ;; → an Outcome; match it; send the reply to clients[idx]; TCO-recur with new-state.
        (:wat::core::match op -> :wat::core::nil
          (:my::counter::Op::Get
            (:wat::core::match
                (:wat::core::let [s state] (:wat::service::Outcome::Reply s (:my::counter::Reply::Get s)))
                -> :wat::core::nil
              ((:wat::service::Outcome::Reply new-state reply)
                (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx) reply)]
                  (:my::counter::serve self l clients new-state)))))
          ((:my::counter::Op::Increment n)
            (:wat::core::match
                (:wat::core::let [s state]
                  (:wat::core::let [s' (:wat::core::i64::+ s n)]
                    (:wat::service::Outcome::Reply s' (:my::counter::Reply::Increment s'))))
                -> :wat::core::nil
              ((:wat::service::Outcome::Reply new-state reply)
                (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx) reply)]
                  (:my::counter::serve self l clients new-state)))))))
      ((:wat::kernel::ServiceEvent::Closed idx)
        (:my::counter::serve self l (:wat::std::list::remove-at clients idx) state))
      ((:wat::kernel::ServiceEvent::Lost idx _cause)
        (:my::counter::serve self l (:wat::std::list::remove-at clients idx) state)))))
```

## The generation algorithm (extend service.wat's existing `foldl` over `clauses`)

C.1 binds `clauses = (ast->children ops)` and folds them into `Op` variants — REUSE. From each clause:
1. **names**: `<fqdn>::Op` (C.1), `<fqdn>::Reply`, `<fqdn>::serve` (string::concat on `(keyword/to-string fqdn)`
   + `"::Op"`/`"::Reply"`/`"::serve"` → `keyword/from-string`).
2. **Reply variant** = `opkw` + `ch[3]` (out-fieldvec children) — same build as C.1's Op variant but using `ch[3]`.
3. **serve op-arm** = pattern `<fqdn>::Op::<Op>` (bare if no input fields, else `` `(~variant-kw ~@arg-name-syms) ``
   where arg-name-syms = the binder symbols of `(drop in-argvec-children 3)`, every 3rd). Arm body = the
   nested `(match (let [s state] <ch[4] body>) ((Outcome::Reply new-state reply) (let [_ (send' (nth clients idx) reply)] (serve self l clients new-state))))`.
4. **assemble** `serve` = c0b1b's ServiceEvent skeleton (copy verbatim — Shutdown/Connection/Closed/Lost +
   the typed param list) with the generated `(match op …)` spliced into the `:Message` arm; wrap
   `(do <defenum Op> <defenum Reply> <defn serve>)`.

Program-body path (as C.1): macro top-level is a regular `let`; params are node-values; output via NESTED
quasiquote (a top-level quasiquote EVALUATES the arg → breaks `ast->children`, STOP-2).

## Read in order (the rooms)

1. `wat/service.wat` — the C.1 macro you extend + the `Outcome<S,R>` defenum (already minted near the top).
2. `tests/nursery/probe_arc209_c0b1b_select_listener.rs` — the hand-rolled `serve`; copy its ServiceEvent
   skeleton + exact param TYPES (substitute the enum names).
3. `tests/probe_arc209_c2_defservice_dispatch.rs` (the gate — returns 5) + `tests/probe_arc209_c1_defservice_op_enum.rs`
   (still GREEN; already RPC-format).
4. `wat/spawn.wat:101` (the `ServiceEvent<I,O>` + the parametric-defenum-with-typed-fields shape, the
   model for `Reply`) + `wat/core.wat` `cond` + `wat/fix.wat` (program-body defmacro + ast-native walking).

## Blast radius

`wat/service.wat` ONLY (the macro; the `Outcome` defenum is already there). NO Rust change. NO change to
the probes. NO change to `poll'`/`ServiceEvent`/`Peer'`/`Listener'`/`Outcome`.

## STOP triggers (halt + report, ship nothing)

1. an op argvec contains `&` (rest) → reject (fixed-arity records only).
2. the program-body contract breaks (a value hits `ast->children`) → top-level `let`, output via nested quasiquote.
3. the `do`-wrapper fails check → re-read DESIGN-STONE-C.1 § do-wrapper (both defenums splice out; serve keeps the do non-empty).
4. **NO dual-format / format-detection branch** — single RPC format only (the C.1 probe is migrated). If you find yourself adding "if ch[3] is a List …", STOP — that's the shim we rejected.

## Gate (report each exact line; do NOT commit)

```
cargo test --release -p wat --test probe_arc209_c2_defservice_dispatch        # 1 passed (returns 5)
cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum         # still GREEN
cargo test --release -p wat --lib -- --test-threads=1                          # 915 / 36 (zero new)
cargo test --release -p wat --test nursery -- --test-threads=1                 # 895 / 4 (zero new)
cargo test --release --workspace --no-run                                      # compiles
```

## Report back

- Each exact gate line.
- The `wat/service.wat` diff shape (Reply-variant build, serve assembly, the Outcome match in the op-arm).
- Any STOP-trigger hits.
- Confirm: touched ONLY `wat/service.wat`, single-format (no detection branch), no Rust/probe edits.
