# BRIEF — Stone C.2: defservice generates `Reply` + `serve` (the RPC dispatch loop)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
operate ONLY here; `git -C /home/watmin/work/holon/wat-rs`; ignore any `.claude/worktrees/` path).
The RED probe is on disk + verified RED (`tests/probe_arc209_c2_defservice_dispatch.rs` →
`UnresolvedReference :my::counter::serve`). Do NOT commit — the Inquisitor weighs. Run `cargo test`
PLAINLY (no setsid/timeout). Stale rust-analyzer diagnostics can contradict a clean build — trust the build.

## The work in one paragraph

Extend the `:wat::service::defservice` macro in **`wat/service.wat`** so its expansion goes from a bare
`defenum` (C.1, the request enum `<fqdn>::Op`) to a **`(:wat::core::do (defenum <fqdn>::Op …) (defenum
<fqdn>::Reply …) (defn <fqdn>::serve …))`**. C.1 already folds `:ops` into the `Op` variants — KEEP that.
C.2 ADDS, from the same op clauses: the **`Reply`** enum (one variant per op, carrying the op's OUTPUT
fields) and **`serve`** (the `poll'`/`ServiceEvent` dispatch loop that owns the live `:state`, decodes an
`Op`, runs the inline handler body, `send'`s the `Reply`, TCO-recurs). The `do`-wrapper now holds: both
`defenum`s splice to top level via `splice_type_decls`; `defn serve` is the non-type-decl sibling that
makes the `do` non-empty (the C.1 note's resolution — see DESIGN-STONE-C.1 § "do-wrapper").

## The op-clause shape (RPC model — surface A, inline-and-minted)

Each op clause is a List, children: `[opkw, in-argvec, ->, out-fieldvec, body]`:
- `ch[0]` `opkw` — `:Get` / `:Increment`
- `ch[1]` `in-argvec` — `[s <- :State n <- :wat::core::i64]` (FIRST triple is `s <- :State` = state-self;
  the REST are the input record fields → already the `Op` variant fields in C.1)
- `ch[2]` `->`
- `ch[3]` `out-fieldvec` — `[value <- :wat::core::i64]` (the output record fields → the `Reply` variant fields)
- `ch[4]` `body` — returns `(:wat::core::Tuple new-state <reply>)` where `<reply>` is a `<fqdn>::Reply::<Op>`
  value the body constructs. (NO `& rest` in any argvec — fixed-arity records only; STOP if `&` appears.)

## The EXACT target expansion (the counter — your template; generalize over ops)

`(defservice :my::counter :state :i64 :ops [(:Get …) (:Increment …)])` must expand to:

```clojure
(:wat::core::do
  ;; C.1 (KEEP) — request enum: variant per op = input fields (state-self dropped)
  (:wat::core::defenum :my::counter::Op
    :Get
    :Increment [n <- :wat::core::i64])

  ;; C.2 (NEW) — response enum: variant per op = the out-fieldvec verbatim
  (:wat::core::defenum :my::counter::Reply
    :Get       [value <- :wat::core::i64]
    :Increment [value <- :wat::core::i64])

  ;; C.2 (NEW) — the dispatch loop. self/l/clients param TYPES mirror probe_arc209_c0b1b's
  ;; `serve` EXACTLY, substituting I=:my::counter::Op (request), O=:my::counter::Reply (reply);
  ;; plus the `state <- <the :state type>` param threaded through every recur.
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
        ;; GENERATED: one arm per op clause. `s` bound to the live `state`; the inline body
        ;; is spliced as `result` (refs s + the variant-bound args); destructure the Tuple;
        ;; send the reply to clients[idx]; TCO-recur with new-state.
        (:wat::core::match op -> :wat::core::nil
          (:my::counter::Op::Get
            (:wat::core::let [s         state
                              result    (:wat::core::Tuple s (:my::counter::Reply::Get s))
                              new-state (:wat::core::nth result 0)
                              reply     (:wat::core::nth result 1)
                              _         (:wat::kernel::send' (:wat::core::nth clients idx) reply)]
              (:my::counter::serve self l clients new-state)))
          ((:my::counter::Op::Increment n)
            (:wat::core::let [s         state
                              result    (:wat::core::let [s' (:wat::core::i64::+ s n)]
                                          (:wat::core::Tuple s' (:my::counter::Reply::Increment s')))
                              new-state (:wat::core::nth result 0)
                              reply     (:wat::core::nth result 1)
                              _         (:wat::kernel::send' (:wat::core::nth clients idx) reply)]
              (:my::counter::serve self l clients new-state)))))
      ((:wat::kernel::ServiceEvent::Closed idx)
        (:my::counter::serve self l (:wat::std::list::remove-at clients idx) state))
      ((:wat::kernel::ServiceEvent::Lost idx _cause)
        (:my::counter::serve self l (:wat::std::list::remove-at clients idx) state)))))
```

## The generation algorithm (extend service.wat's existing `foldl` over `clauses`)

C.1's macro already binds `clauses = (ast->children ops)` and folds them into `Op` variants. Reuse that
walk; from each clause also build the `Reply` variant + the `serve` op-arm:
1. **`enum-name` / `reply-name` / `serve-name`** = `<fqdn>::Op` / `<fqdn>::Reply` / `<fqdn>::serve`
   (string::concat on `(keyword/to-string fqdn)` + `"::Op"` / `"::Reply"` / `"::serve"` →
   `keyword/from-string`, as C.1 does for `enum-name`).
2. **Reply variant** per clause = `opkw` + `ch[3]` (out-fieldvec children) — same shape as C.1's Op-variant
   build but using the out-fieldvec INSTEAD of the dropped-state in-argvec fields.
3. **op-arm** per clause: pattern = bare `<fqdn>::Op::<Op>` if no input fields, else
   `` `(~variant-kw ~@arg-name-syms) `` (arg-name-syms = the binder symbols of the in-argvec AFTER the
   `s <- :State` triple — every-3rd symbol of `(drop in-argvec-children 3)`); arm body = the `let`
   shown (s←state, `result`←spliced `ch[4]` body, `nth 0/1`, `send'`, recur).
4. **assemble** `serve` = the ServiceEvent skeleton (copy c0b1b's `serve` verbatim — Shutdown/Connection/
   Closed/Lost arms + the typed param list) with the generated `(match op …)` spliced into the `:Message`
   arm; wrap `(do <defenum Op> <defenum Reply> <defn serve>)`.

Program-body path (as C.1): macro top-level is a regular `let`; params are node-values `ast->children`
accepts; output via NESTED quasiquote. A top-level quasiquote EVALUATES the arg → breaks `ast->children`.

## Read in order (the rooms)

1. `wat/service.wat` — the C.1 macro you extend (the `foldl` over `clauses` building `variants`). KEEP the
   Op-variant logic; add Reply-variant + serve generation alongside.
2. `tests/nursery/probe_arc209_c0b1b_select_listener.rs` — the hand-rolled thread-tier `serve` + driver.
   **Copy `serve`'s ServiceEvent skeleton + the exact param TYPES** (substitute the enum names).
3. `tests/probe_arc209_c2_defservice_dispatch.rs` — the gate. Make it GREEN (returns `5`).
4. `wat/core.wat` `cond` (`:254`) + `wat/fix.wat` — the program-body defmacro + ast-native walking idioms
   (`ast->children`, `first`/`rest`/`drop`, `keyword/from-string`, nested quasiquote, `~@` splice).
5. `docs/arc/2026/05/209-defservice/DESIGN-STONE-C.2-dispatch-loop.md` — the design + the LOCKED names.

## Blast radius

`wat/service.wat` ONLY (the macro). NO Rust change. NO change to the C.2/C.1 probes. NO change to
`poll'`/`ServiceEvent`/`Peer'`/`Listener'` (all exist — thread tier is c0b1b, shipped).

## STOP triggers (halt + report, ship nothing)

1. **STOP-1:** an op argvec contains `&` (rest) — reject; ops are fixed-arity records (model variadic as a
   `Vector<T>` field). Report the offending op.
2. **STOP-2:** the program-body contract breaks (a value reaches `ast->children` instead of a node) — keep
   the macro top-level a regular `let`, output via nested quasiquote (the C.1 trap).
3. **STOP-3:** the `do`-wrapper fails check (e.g. a `defenum` doesn't top-level, or the `do` is seen empty)
   — re-read the C.1 § "do-wrapper" note; both `defenum`s splice out, `serve` keeps the `do` non-empty.
4. **STOP-4:** `(nth <tuple> <literal>)` won't type-check as a tuple projection — find the real accessor
   before improvising (check.rs:9610 says nth is polymorphic over Vec + tuple, index-addressed).

## Gate (report each exact line; do NOT commit)

```
cargo test --release -p wat --test probe_arc209_c2_defservice_dispatch        # 1 passed (returns 5)
cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum         # still GREEN (Op unchanged)
cargo test --release -p wat --lib -- --test-threads=1                          # 915 / 36 (zero new)
cargo test --release -p wat --test nursery -- --test-threads=1                 # 895 / 4 (zero new)
cargo test --release --workspace --no-run                                      # compiles
```

## Prior comparable (copy the shape)

`wat/service.wat` (the C.1 macro you extend) + `probe_arc209_c0b1b` (the `serve` you mirror) +
`wat/core.wat` `cond` (the program-body defmacro template).
