;; wat/query.wat — Arc 278 stone S4: the :wat::query backend-agnostic storage CONTRACT, on the
;; SERVICES-AS-SURFACES operation model.
;;
;; Ratified in docs/arc/2026/06/278-rules-engine/DESIGN-store-contract.md (S0), migrated to the
;; operation model at S4 per arc 293 Path B (`823b20ac`): `Store` is now a `:nature :wat::kernel::Peer'`
;; surface — a DIALED PEER of a `:satisfies Store` service IS a Store, intrinsically (no wrapper
;; struct, no `extend-type`). The narrow waist is still DynamoDB's (pk, sk, data) + named-GSI
;; (ipk, isk) shape: all keys are EDN-form STRINGS the consumer serializes/hydrates; `data` is
;; opaque EDN the backend never inspects (§ data model). `wat.query` (the rete-as-datalog filter)
;; reasons over decoded records in working memory; the backend only ever hands it opaque pages.
;;
;; ─── the operation model (S4) ──────────────────────────────────────────────────────────────────
;; Every fallible/successful op returns a per-op OUTCOME ENUM named `Store::<Op>Response`
;; (`:Success` first, then that op's own error variants — never a bare success type, never a
;; generic `(Result :- [T Error])`). The error channel is an errors-as-record model on the RECOVERY axis
;; (the caller's forced branch: retry / surface / abort) — `Transient` / `Constraint` / `Fatal`,
;; each carrying a `reason <- Reason` (an OPEN surface — any pure record satisfies it; `Fault
;; [message <- String]` is the concrete default a backend with nothing more structured reaches for).
;;
;; Only outward refs: `:wat::core::*` (String/i64/keyword/nil/Vector/Option/HashMap/Struct) +
;; `:wat::enum::Pure` + `:wat::kernel::Peer'`. Loads after `wat/core.wat` (defrecord/defenum/
;; defsurface + those primitives) and `wat/service.wat` (the `Peer'` nature + `:satisfies`
;; machinery); placed near the rete sources — this is the query engine's vocabulary.

;; ─── the write input ─────────────────────────────────────────────────────────────────────────
(:wat::core::defrecord :wat::query::IndexKey                 ;; a named GSI's own projected keys
  [ipk <- :wat::core::String
   isk <- :wat::core::String])

(:wat::core::defrecord :wat::query::StoredRow                ;; one record to `put`
  [pk         <- :wat::core::String                          ;; EDN-form key string; consumer serializes <-> hydrates
   sk         <- :wat::core::String
   data       <- :wat::core::String                          ;; the record's tagged EDN, opaque to the backend
   index-keys <- (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey])]) ;; index-name -> (ipk,isk)

(:wat::core::defrecord :wat::query::Key                      ;; the (pk, sk) a `delete` names — StoredRow without data/index-keys
  [pk <- :wat::core::String
   sk <- :wat::core::String])

;; ─── the read results — what scan / scan-index hand back ───────────────────────────────────
(:wat::core::defrecord :wat::query::Row
  [pk   <- :wat::core::String
   sk   <- :wat::core::String
   data <- :wat::core::String])

(:wat::core::defrecord :wat::query::IndexRow                 ;; the 4-keyed index row
  [pk   <- :wat::core::String                                ;; the base keys
   sk   <- :wat::core::String
   ipk  <- :wat::core::String                                ;; the GSI's own keys
   isk  <- :wat::core::String
   data <- :wat::core::String])

;; ─── the pages — results + the keyset resume cursor (vocabulary; the operation model's own
;; `Store::ScanResponse::Success`/`ScanIndexResponse::Success` carry rows+cursor directly rather
;; than nesting one of these — kept as the shared shape for consumers that want to box a page) ────
(:wat::core::defrecord :wat::query::Page
  [rows        <- (:wat::core::Vector :- [:wat::query::Row])
   next-cursor <- (:wat::core::Option :- [:wat::core::String])])

(:wat::core::defrecord :wat::query::IndexPage
  [rows        <- (:wat::core::Vector :- [:wat::query::IndexRow])
   next-cursor <- (:wat::core::Option :- [:wat::core::String])])

;; ─── schema declarations (ensure-schema input) ──────────────────────────────────────────────
(:wat::core::defrecord :wat::query::TableSchema
  [pk <- :wat::core::String
   sk <- :wat::core::String])

(:wat::core::defrecord :wat::query::IndexSchema
  [name <- :wat::core::String                                ;; the GSI's name — S2's secondary-complete-tables
                                                              ;; model makes this the table name (`index_<name>`)
   pk  <- :wat::core::String
   sk  <- :wat::core::String
   ipk <- :wat::core::String
   isk <- :wat::core::String])

;; ─── the error vocabulary — recovery-axis records over an OPEN Reason surface ───────────────────
;; `Reason` has zero features: any pure record satisfies it ambiently (an OPEN Record surface)
;; — no `extend-type`/`derive` needed.
(:wat::core::defsurface :wat::query::Reason :nature :wat::core::Record :features [])

(:wat::core::defrecord :wat::query::Transient  [reason <- :wat::query::Reason]) ;; retry — momentarily unavailable
(:wat::core::defrecord :wat::query::Constraint [reason <- :wat::query::Reason]) ;; surface — schema/uniqueness violation
(:wat::core::defrecord :wat::query::Fatal      [reason <- :wat::query::Reason]) ;; abort — unrecoverable

;; a concrete default `Reason` satisfier for a backend with nothing more structured to say.
(:wat::core::defrecord :wat::query::Fault [message <- :wat::core::String])

;; ─── the Sieve filter spec (arc 278 Stone 2 — the sift Predicate delivery) ───────────────────
;; DESIGN-sift-server-side-filter.md: server-side log/metric filtering — the client submits a
;; pure filter spec, the server runs it over a page, only survivors cross the wire back. `Sieve`
;; is general (rete-over-a-page, not telemetry-specific) — a union of filter forms. THIS STONE
;; defines `:Predicate` only; `:All` (the pass-all fast path) and `:Rules` (the full rete form)
;; are later stones (RULING order: Predicate -> Rules -> All+annihilate query-*).
;;
;; `pred` is the VERBATIM `::`-source of a user `(fn [log] -> :bool …)`, printed by
;; `ast->source` (never hand-typed — see `sieve-pred` below). A `WatAST` field can't cross a
;; process wire (the general wire-decode crashes on bare symbols); a `String` of `::`-source,
;; rebuilt with `read-string` on the far side, is loci-agnostic (thread == process).
(:wat::core::defenum :wat::query::Sieve :wat::enum::Pure
  :Predicate [pred <- :wat::core::String])

;; sieve-pred — the organic-UX capture macro. The user writes a REAL `(fn [log] -> :bool …)`;
;; this macro (modeled on `defrule`, wat/rete.wat:1971) captures the form, `ast->source`s it into
;; a String, and expands to a `Sieve::Predicate` — the user never types a string themselves.
(:wat::core::defmacro :wat::query::sieve-pred
  [fn-form <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let [src (:wat::core::ast->source fn-form)]
    `(:wat::query::Sieve::Predicate ~src)))

;; ─── sift-rules-defsvc — arc 278 task #6: the Rules form (the chaos engine's inference tier) ────
;; DESIGN-sift-server-side-filter.md / BRIEF-STONE-sift-rules.md. The user hands `:defs` (their
;; `defrecord`s) + `:rules` (their `defrule`s); this macro emits an ADHOC `:satisfies` service
;; (surface + defservice) whose `sift-rules` op reads a page of Logs from a held Journal peer and
;; fires the user's rules PER LOG (one seed per fire — alpha-only structural, RENASCOR NON RETRACTO
;; at the record grain), flat-mapping the DERIVED facts (not the seeds) into one
;; `(PersistentVector :- [wat::core::Value])` reply — the level-up from the Predicate's SELECT to INFER
;; (one log can yield MANY deductions; the returned count can exceed the page size).
;;
;; Canonical kwargs order assumed (defrule precedent, wat/rete.wat:1971): `:name :defs :rules`.
;;
;; ── the two macro-time extraction problems (both probed clean, scratchpad/probe-rule-lits.wat) ──
;;   (1) Rule VALUES, not defns: `:rules` are literal `(defrule name :when […] :then …)` forms. A
;;       spliced top-level defn would NOT cross a process fork (only the satisfied surface's
;;       :messages + :peers surfaces' :messages + this defservice's own internals ship —
;;       tests/services/probe_arc278_sift_arena.wat's `:cons::sift-loop` note). So each rule form
;;       is decomposed (name/`:when`/`:then`, mirroring defrule's own body) into a
;;       `(:wat::rete::make-rule …)` call LITERAL — no defn, no cross-fork gap.
;;   (2) Deduction extraction: `Session/facts` does NOT carry derived facts post-fire (proven false
;;       — probed: pre == post even though the rules demonstrably fired); QueryNode lookup DOES.
;;       This macro walks every rule's `:then` fact-forms (arc 278 Stone A: `:then` is a vector of
;;       bare facts, no `insert` wrapper), collects the UNIQUE derived type names, and emits one
;;       `make-query` + `(:wat::rete::query fired q)` per unique type, flat-mapped via
;;       `:wat::core::concat`. No type-keyword `query` mouth — that path is gone.
;;
;; Everything (both extractions above, the Journal page read, the class-guarded foreign decode, the
;; fire/collect fold) is INLINED directly into the `:impls` op body — never a sibling top-level
;; defn — for the same cross-fork reason as (1).
(:wat::core::defmacro :wat::query::sift-rules-defsvc
  [& clauses <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::WatAST
  (:wat::core::let
    [name-node  (:wat::core::Option/expect (:wat::core::get clauses 1) "sift-rules-defsvc: missing :name")
     defs-node  (:wat::core::Option/expect (:wat::core::get clauses 3) "sift-rules-defsvc: missing :defs")
     rules-node (:wat::core::Option/expect (:wat::core::get clauses 5) "sift-rules-defsvc: missing :rules")

     ;; strip-leading-colon inline (defrule's own idiom — can't call a user-defn from a
     ;; program-body macro).
     raw-name   (:wat::core::ast-name name-node)
     name-str   (:wat::core::if (:wat::core::= (:wat::string::subs raw-name 0 1) ":")
                  (:wat::string::subs raw-name 1 (:wat::string::length raw-name))
                  raw-name)
     svc-str    (:wat::string::interpolate "{name-str}'" :name-str name-str)

     surface-kw  (:wat::core::keyword-node (:wat::string::interpolate ":{name-str}" :name-str name-str))
     svc-kw      (:wat::core::keyword-node (:wat::string::interpolate ":{svc-str}" :svc-str svc-str))
     req-kw      (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesRequest")))
     resp-kw     (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesResponse")))
     resp-ded-kw (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesResponse::Deductions")))
     resp-fat-kw (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesResponse::Fatal")))
     resp-rtl-kw (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesResponse::RequestTooLarge")))
     resp-rm-kw  (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesResponse::RequestMalformed")))
     sift-reply-kw (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::Reply::SiftRules")))
     surface-reply-kw (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::Reply")))
     svc-op-kw (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat svc-str "::Op")))
     req-ns-kw   (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesRequest/namespace")))
     req-lo-kw   (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesRequest/time-lo")))
     req-hi-kw   (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesRequest/time-hi")))
     req-lim-kw  (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesRequest/limit")))
     req-cur-kw  (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat name-str "::SiftRulesRequest/cursor")))

     record-ty-kw (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat svc-str "::Record")))
     state-ty-kw  (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat svc-str "::State")))
     state-journal-kw  (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat svc-str "::State/journal")))
     state-template-kw (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat svc-str "::State/template")))
     state-durable-kw  (:wat::core::keyword-node (:wat::string::concat ":" (:wat::string::concat svc-str "::State/durable")))

     defs-children  (:wat::core::ast->children defs-node)
     rules-children (:wat::core::ast->children rules-node)

     ;; ── (1) rule-lits: (Vector :- [WatAST]) of `(make-rule name-str (quote when-vec) (quote then-vec))`
     ;; call literals — one per :rules form. Mirrors defrule's own body (rete.wat:2150) exactly,
     ;; minus the defn wrapper (see the doc comment above). Arc 278 Stone A: `:then` is now a
     ;; VECTOR (child[5] of rch), quoted as-is — symmetric with when-vec; no more splicing.
     rule-lits
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) rf <- :wat::WatAST]
           -> (:wat::core::Vector :- [:wat::WatAST])
           (:wat::core::let
             [rch        (:wat::core::ast->children rf)
              rname      (:wat::core::Option/expect (:wat::core::get rch 1) "sift-rules-defsvc: rule missing name")
              raw-rname  (:wat::core::ast-name rname)
              rname-str  (:wat::core::if (:wat::core::= (:wat::string::subs raw-rname 0 1) ":")
                           (:wat::string::subs raw-rname 1 (:wat::string::length raw-rname))
                           raw-rname)
              when-vec   (:wat::core::Option/expect (:wat::core::get rch 3) "sift-rules-defsvc: rule missing :when")
              then-vec   (:wat::core::Option/expect (:wat::core::get rch 5) "sift-rules-defsvc: rule missing :then")
              rule-lit   `(:wat::rete::make-rule ~rname-str (:wat::core::quote ~when-vec) (:wat::core::quote ~then-vec))]
             (:wat::core::conj acc rule-lit)))
         (:wat::core::Vector :- [:wat::WatAST])
         rules-children)

     ;; ── (2) derived-type-strs: UNIQUE type names across every rule's `:then` fact-forms — the
     ;; set of types this Rules-form ever DERIVES (query'd back per type; the proven fallback,
     ;; since Session/facts doesn't carry them). Arc 278 Stone A: each `:then` member IS the
     ;; fact-form directly (no more `(insert (:Type …))` wrapper to unwrap).
     derived-type-strs
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) rf <- :wat::WatAST]
           -> (:wat::core::Vector :- [:wat::core::String])
           (:wat::core::let
             [rch (:wat::core::ast->children rf)
              then-vec   (:wat::core::Option/expect (:wat::core::get rch 5) "sift-rules-defsvc: rule missing :then")
              then-forms (:wat::core::ast->children then-vec)]
             (:wat::core::foldl
               (:wat::core::fn [acc2 <- (:wat::core::Vector :- [:wat::core::String]) tf <- :wat::WatAST]
                 -> (:wat::core::Vector :- [:wat::core::String])
                 (:wat::core::let
                   ;; tf IS the fact-form directly (arc 278 Stone A dropped the insert wrapper).
                   [cch  (:wat::core::ast->children tf)
                    tkw  (:wat::core::Option/expect (:wat::core::get cch 0)
                           "sift-rules-defsvc: :then fact-form missing a type")
                    traw (:wat::core::ast-name tkw)
                    tstr (:wat::core::if (:wat::core::= (:wat::string::subs traw 0 1) ":")
                           (:wat::string::subs traw 1 (:wat::string::length traw))
                           traw)]
                   (:wat::core::if (:wat::vec::contains? acc2 tstr) acc2 (:wat::core::conj acc2 tstr))))
               acc
               then-forms)))
         (:wat::core::Vector :- [:wat::core::String])
         rules-children)

     ;; ── (2b) fired-upon-type-strs: UNIQUE type names appearing as the HEAD of any rule's
     ;; `:when` condition (mirrors (2)'s ctor-head walk, but over when-vec conditions directly —
     ;; every `:when` clause is `(:Type (?bind <- :field) …tests…)`, per make-rule/compile-rule's
     ;; own "form::matches?-shaped clauses" contract). This is the CASCADED-UPON set.
     fired-upon-type-strs
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) rf <- :wat::WatAST]
           -> (:wat::core::Vector :- [:wat::core::String])
           (:wat::core::let
             [rch      (:wat::core::ast->children rf)
              when-vec (:wat::core::Option/expect (:wat::core::get rch 3)
                         "sift-rules-defsvc: rule missing :when")
              conds    (:wat::core::ast->children when-vec)]
             (:wat::core::foldl
               (:wat::core::fn [acc2 <- (:wat::core::Vector :- [:wat::core::String]) cf <- :wat::WatAST]
                 -> (:wat::core::Vector :- [:wat::core::String])
                 (:wat::core::let
                   [cch  (:wat::core::ast->children cf)
                    ckw  (:wat::core::Option/expect (:wat::core::get cch 0)
                           "sift-rules-defsvc: :when condition missing a type")
                    craw (:wat::core::ast-name ckw)
                    cstr (:wat::core::if (:wat::core::= (:wat::string::subs craw 0 1) ":")
                           (:wat::string::subs craw 1 (:wat::string::length craw))
                           craw)]
                   (:wat::core::if (:wat::vec::contains? acc2 cstr) acc2 (:wat::core::conj acc2 cstr))))
               acc
               conds)))
         (:wat::core::Vector :- [:wat::core::String])
         rules-children)

     ;; ── (2c) deduction-type-strs: derived − fired-upon — the TERMINAL types (derived and no
     ;; rule ever matches on them). Lemma types (derived ∩ fired-upon) cascade internally
     ;; (fire-to-fixpoint, already proven) but are deliberately EXCLUDED here — never returned.
     deduction-type-strs
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) tstr <- :wat::core::String]
           -> (:wat::core::Vector :- [:wat::core::String])
           (:wat::core::if (:wat::vec::contains? fired-upon-type-strs tstr)
             acc
             (:wat::core::conj acc tstr)))
         (:wat::core::Vector :- [:wat::core::String])
         derived-type-strs)

     fired-sym (:wat::core::symbol-node "fired")
     fact-sym  (:wat::core::symbol-node "?fact")
     pmap-sym  (:wat::core::symbol-node "p")
     ;; query-lits / query-calls: one make-query + `(:wat::rete::query fired q)` per unique
     ;; DEDUCTION (derived − fired-upon) type. make-query (not defquery) so the Query value
     ;; is a literal in the :init / op body — same cross-fork reason as make-rule. Lemma
     ;; types are NOT queried back — they stay internal to the fired session.
     query-lits
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) tstr <- :wat::core::String]
           -> (:wat::core::Vector :- [:wat::WatAST])
           (:wat::core::let
             [tkw  (:wat::core::keyword-node
                      (:wat::string::interpolate ":{tstr}" :tstr tstr))
              cond `(~fact-sym <- ~tkw)]
             (:wat::core::conj acc
               `(:wat::rete::make-query ~tstr
                  (:wat::core::quote [])
                  (:wat::core::quote [~cond])))))
         (:wat::core::Vector :- [:wat::WatAST])
         deduction-type-strs)
     query-calls
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) lit <- :wat::WatAST]
           -> (:wat::core::Vector :- [:wat::WatAST])
           (:wat::core::conj acc
             `(:wat::core::into (:wat::core::PersistentVector)
                (:wat::core::map
                  (:wat::core::fn [~pmap-sym <- :wat::core::PersistentMap] -> :wat::core::Value
                    (:wat::core::Option/expect
                      (:wat::map::get ~pmap-sym "?fact")
                      "sift-rules: ?fact"))
                  (:wat::rete::query ~fired-sym ~lit)))))
         (:wat::core::Vector :- [:wat::WatAST])
         query-lits)

     ;; concat-chain: `:wat::core::concat` is BINARY (Vector/concat's arity, not variadic) —
     ;; `(:wat::core::concat ~@query-calls)` only type-checks when there are exactly 2 deduction
     ;; types (the #6 gate's Hot/Warn happened to be 2 — masking this). A richer graph (3+
     ;; deduction types) needs a LEFT-FOLDED chain of binary concats, built here at expand time.
     ;; Zero deduction types (theoretically possible) folds to an empty PersistentVector literal.
     concat-chain
       (:wat::core::if (:wat::core::= (:wat::core::length query-calls) 0)
         `(:wat::core::PersistentVector)
         (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::WatAST qc <- :wat::WatAST] -> :wat::WatAST
             `(:wat::core::concat ~acc ~qc))
           (:wat::core::first query-calls)
           (:wat::core::rest query-calls)))

     ;; def-type-strs: colon-free type names of the user's `:defs` — the class-guard vocabulary
     ;; (fail-closed: a Log whose decoded message class isn't among these → ::Fatal, never a crash
     ;; or a silent skip — no-hidden-failures).
     def-type-strs
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) df <- :wat::WatAST]
           -> (:wat::core::Vector :- [:wat::core::String])
           (:wat::core::let
             [dch  (:wat::core::ast->children df)
              dn   (:wat::core::Option/expect (:wat::core::get dch 1) "sift-rules-defsvc: def missing a name")
              draw (:wat::core::ast-name dn)
              dstr (:wat::core::if (:wat::core::= (:wat::string::subs draw 0 1) ":")
                     (:wat::string::subs draw 1 (:wat::string::length draw))
                     draw)]
             (:wat::core::conj acc dstr)))
         (:wat::core::Vector :- [:wat::core::String])
         defs-children)

     ;; synthetic binder symbols — every literal `let`/`fn` inside the templates below must use
     ;; one of these (Unquote nodes), never a bare Symbol, per the ProgramBodyIntroducesName gate
     ;; (expand.rs:875) — a bare Symbol at a let-binder/fn-param slot directly inside a quasiquote
     ;; reads as accidental capture, not intentional hygiene.
     record-sym (:wat::core::symbol-node "record")
     jaddr-sym  (:wat::core::symbol-node "journal-addr")
     ok-sym     (:wat::core::symbol-node "ok")
     log-sym    (:wat::core::symbol-node "log")
     acc-sym    (:wat::core::symbol-node "acc")
     stop-s-sym (:wat::core::symbol-node "s")
     rel-sym    (:wat::core::symbol-node "rel")
     payload-sym (:wat::core::symbol-node "payload")
     cause-sym   (:wat::core::symbol-node "cause")]
    `(:wat::core::do
       (:wat::core::defsurface ~surface-kw :nature :wat::kernel::Peer
         :messages
         [~@defs-children
          (:wat::core::defrecord ~req-kw
            [namespace <- :wat::core::String
             time-lo   <- :wat::core::i64
             time-hi   <- :wat::core::i64
             limit     <- :wat::core::i64
             cursor    <- (:wat::core::Option :- [:wat::core::String])])
          (:wat::core::defenum ~resp-kw :wat::enum::Pure
            :Deductions [items  <- (:wat::core::PersistentVector :- [:wat::core::Value])
                         cursor <- (:wat::core::Option :- [:wat::core::String])]
            :Fatal      [err   <- :wat::query::Fault]
            :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
            :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
         :features
         [(sift-rules [self <- ~surface-kw req <- ~req-kw] -> ~resp-kw :max-request-bytes 524288)])
       (:wat::service::defservice ~svc-kw
         :satisfies ~surface-kw
         :durable   []
         :ephemeral [journal  <- (:wat::kernel::Peer :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply])
                     template <- :wat::rete::Session]
         :peers     [:wat::telemetry::Journal]
         ;; :init compiles ~@:rules into a Session TEMPLATE (WM empty) held in :ephemeral state —
         ;; the arena's `journal` peer field is the precedent for a derived-at-init, never-mutated
         ;; resource living there (mem.wat's `rows` is the :durable precedent for a plain held
         ;; value; a Session is closer to "a resource" than "mutated data").
         :init (:wat::core::fn
                 [~record-sym <- ~record-ty-kw
                  ~jaddr-sym  <- (:wat::kernel::Address :- [:wat::telemetry::Journal::Op :wat::telemetry::Journal::Reply])]
                 -> ~state-ty-kw
                 ;; arc 278 the connect'-outcome wall — the generated :init dial faces all
                 ;; four arms; ::Connected → the journal Peer'; failure arms →
                 ;; assertion-failed! (fatal, preserving the pre-wall raise-unwind: a sift
                 ;; service whose journal dial fails at :init cannot start).
                 (~state-ty-kw
                   :durable  ~record-sym
                   :journal  (:wat::core::match (:wat::kernel::connect ~jaddr-sym)
                               ((:wat::kernel::ConnectOutcome::Connected p) p)
                               ((:wat::kernel::ConnectOutcome::Refused c)
                                 (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                               ((:wat::kernel::ConnectOutcome::Rejected c)
                                 (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
                               ((:wat::kernel::ConnectOutcome::Failed c)
                                 (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
                   :template (:wat::rete::compile-all
                               (:wat::core::PersistentVector ~@rule-lits)
                               (:wat::core::PersistentVector ~@query-lits))))
         ;; Hangup drops the intern lease `compile-all` took (`DESIGN-STONE-intern-eviction`).
         :stop (:wat::core::fn [~stop-s-sym <- ~state-ty-kw] -> ~record-ty-kw
                 (:wat::core::let [~rel-sym (:wat::rete::release-session (~state-template-kw ~stop-s-sym))]
                   (~state-durable-kw ~stop-s-sym)))
         :impls
         ;; arc 278 ctx-is-mandatory — `[s ctx req]`, not `[s req]`: EVERY public op arm receives an
         ;; `:wat::service::Invocation` (BRIEF-ctx-is-mandatory.md). This template was missed by the
         ;; migration's file list because THIS FILE NEVER SAYS "defservice" at its consumers — a
         ;; caller writes `(:wat::query::sift-rules-defsvc …)`, so `grep -rl 'defservice'` and a
         ;; top-level `defservice`-form census BOTH skipped every consumer of this macro. The new
         ;; arity wall is what surfaced it, by name, at load. One template, every consumer fixed.
         [(sift-rules [s ctx req]
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (~sift-reply-kw (:wat::core::match
                (:wat::telemetry::Journal/query-logs (~state-journal-kw s)
                  (:wat::telemetry::Journal::QueryLogsRequest
                    :namespace (~req-ns-kw req)
                    :time-lo   (~req-lo-kw req)
                    :time-hi   (~req-hi-kw req)
                    :limit     (~req-lim-kw req)
                    :cursor    (~req-cur-kw req)))
                ;; the Journal peer client-method now returns (RecvOutcome :- [QueryLogsResponse]) — a lost/closed
                ;; Journal backend must NOT kill this shared sift service (client-triggerable-DoS forbidden):
                ;; map to our own ::Fatal response value and KEEP SERVING (mirrors telemetry/journal.wat).
                ((:wat::kernel::RecvOutcome::Message sresp)
                  (:wat::core::match sresp
                    ((:wat::telemetry::Journal::QueryLogsResponse::Success logs next-cur)
                      (:wat::core::if
                        (:wat::core::foldl
                          (:wat::core::fn [~ok-sym <- :wat::core::bool ~log-sym <- :wat::telemetry::Log]
                            -> :wat::core::bool
                            (:wat::core::if ~ok-sym
                              (:wat::core::match
                                (:wat::edn::read-foreign (:wat::telemetry::Log/message ~log-sym))
                                ((:wat::edn::ReadForeignOutcome::Value ~payload-sym)
                                  (:wat::vec::contains?
                                    (:wat::core::Vector :- [:wat::core::String] ~@def-type-strs)
                                    (:wat::core::type ~payload-sym)))
                                ((:wat::edn::ReadForeignOutcome::Malformed ~cause-sym)
                                  false))
                              false))
                          true
                          logs)
                        (~resp-ded-kw
                          (:wat::core::foldl
                            (:wat::core::fn [~acc-sym <- (:wat::core::PersistentVector :- [:wat::core::Value])
                                             ~log-sym <- :wat::telemetry::Log]
                              -> (:wat::core::PersistentVector :- [:wat::core::Value])
                              (:wat::core::concat ~acc-sym
                                (:wat::core::let
                                  [~fired-sym (:wat::rete::fire-rules
                                                (:wat::rete::insert (~state-template-kw s)
                                                  (:wat::edn::read (:wat::telemetry::Log/message ~log-sym))))]
                                  ~concat-chain)))
                            (:wat::core::PersistentVector)
                            logs)
                          next-cur)
                        (~resp-fat-kw
                          (:wat::query::Fault :message "sift-rules: a Log message type is not among :defs"))))
                    ;; propagate the budget signal EXPLICITLY — never lump RequestTooLarge into Fatal (ruling A).
                    ((:wat::telemetry::Journal::QueryLogsResponse::RequestTooLarge bytes cap)
                      (~resp-rtl-kw bytes cap))
                    ;; …and the SHAPE signal identically (arc 278 Stone 2). The codemod could not
                    ;; decide this arm structurally — the RequestTooLarge arm above propagates through
                    ;; an UNQUOTED head (`~resp-rtl-kw`, a macro-built keyword), not a literal one, so
                    ;; it fell to the terminal `assertion-failed!` default. Asserting here would be
                    ;; wrong and dangerous: a shape refusal from the journal peer would kill THIS
                    ;; service for every client — the exact DoS this stone closes, one tier up.
                    ((:wat::telemetry::Journal::QueryLogsResponse::RequestMalformed mpath mexpected mgot)
                      (~resp-rm-kw mpath mexpected mgot))
                    (_ (~resp-fat-kw (:wat::query::Fault :message "sift-rules: journal query-logs failed")))))
                ((:wat::kernel::RecvOutcome::Lost cause)
                  (~resp-fat-kw (:wat::query::Fault :message (:wat::kernel::LociDiedError/message cause))))
                ;; arc 278 #73 — a stop reached this call, not a close. Same Fatal shape (the sift
                ;; cannot complete either way) with the TRUE reason: the journal peer was alive.
                ;; This arm is macro-generated, so it reports at the `sift-rules-defsvc` CALL SITE,
                ;; never here — which is why it was missed on the first stdlib pass and found by a
                ;; rider hitting STOP-1 in tests/services.
                (:wat::kernel::RecvOutcome::Stopped
                  (~resp-fat-kw (:wat::query::Fault :message "query.wat: stop requested mid-sift — the journal peer was ALIVE")))
                (:wat::kernel::RecvOutcome::Closed
                  (~resp-fat-kw (:wat::query::Fault :message "query.wat: journal peer closed"))) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
              (:wat::core::Vector :- [(:wat::service::Directed :- [~surface-reply-kw])])
              (:wat::core::Vector :- [(:wat::service::Alarm :- [~svc-op-kw])])))]))))

;; ─── the contract — the Store surface, on the operation model ──────────────────────────────────
;; :nature :wat::kernel::Peer' — a satisfier is a `:satisfies Store` defservice; a dialed
;; `(Peer' :- [Store::Op Store::Reply])` IS a Store INTRINSICALLY (arc 293 Path B) — no wrapper struct,
;; no extend-type. `ReadStore` (the S0 read-only narrowing) is DELETED here: no live consumer, and
;; its only satisfiers were the wrapper structs this stone removes; reintroduce as a Store-peer
;; read-only narrowing when a real read-only consumer needs it.
;;
;; ─── the surface OWNS its protocol (arc 278 S4c) ──────────────────────────────────────────────
;; The per-op request/response records live in the surface's `:messages` block — convention-named
;; `Store::<Op>Request`/`Store::<Op>Response` (the `defservice :satisfies` macro synthesizes
;; req-ty/resp-ty from these exact names — wat/service.wat:1046-1051). Owning them here means a
;; `:satisfies Store` service ships the protocol across a process fork via the surface's
;; surface-forms carrier (else the forked child never receives them → StartupError). The SHARED
;; domain vocabulary they are built from (StoredRow/Row/IndexRow/IndexKey/Page/IndexPage/
;; TableSchema/IndexSchema) + the error records (Reason/Transient/Constraint/Fatal/Fault) stay
;; top-level: they cross via stdlib, are not per-op messages.
(:wat::core::defsurface :wat::query::Store :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat::query::Store::EnsureSchemaRequest
     [table   <- :wat::query::TableSchema
      indexes <- (:wat::core::Vector :- [:wat::query::IndexSchema])])

   (:wat::core::defenum :wat::query::Store::EnsureSchemaResponse :wat::enum::Pure
     :Success        []
     :Constraint     [err <- :wat::query::Constraint]
     :Fatal          [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::query::Store::PutRequest
     [rows <- (:wat::core::Vector :- [:wat::query::StoredRow])])

   (:wat::core::defenum :wat::query::Store::PutResponse :wat::enum::Pure
     :Success        []
     :Constraint     [err <- :wat::query::Constraint]
     :Transient      [err <- :wat::query::Transient]
     :Fatal          [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::query::Store::DeleteRequest
     [keys <- (:wat::core::Vector :- [:wat::query::Key])])

   (:wat::core::defenum :wat::query::Store::DeleteResponse :wat::enum::Pure
     :Success        []
     :Constraint     [err <- :wat::query::Constraint]
     :Transient      [err <- :wat::query::Transient]
     :Fatal          [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::query::Store::ScanRequest         ;; a base-table page request
     [pk     <- :wat::core::String
      sk-lo  <- :wat::core::String
      sk-hi  <- :wat::core::String
      limit  <- :wat::core::i64
      cursor <- (:wat::core::Option :- [:wat::core::String])])        ;; None = first page; Some sk = resume after (keyset)

   (:wat::core::defenum :wat::query::Store::ScanResponse :wat::enum::Pure
     :Success   [rows   <- (:wat::core::Vector :- [:wat::query::Row])
                 cursor <- (:wat::core::Option :- [:wat::core::String])]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::query::Store::ScanIndexRequest    ;; a GSI page request
     [index  <- :wat::core::String
      ipk    <- :wat::core::String
      isk-lo <- :wat::core::String
      isk-hi <- :wat::core::String
      limit  <- :wat::core::i64
      cursor <- (:wat::core::Option :- [:wat::core::String])])

   (:wat::core::defenum :wat::query::Store::ScanIndexResponse :wat::enum::Pure
     :Success   [rows   <- (:wat::core::Vector :- [:wat::query::IndexRow])
                 cursor <- (:wat::core::Option :- [:wat::core::String])]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::query::Store::CountIndexRequest   ;; a GSI count — no rows
     [index  <- :wat::core::String
      ipk    <- :wat::core::String
      isk-lo <- :wat::core::String
      isk-hi <- :wat::core::String])

   (:wat::core::defenum :wat::query::Store::CountIndexResponse :wat::enum::Pure
     :Ok        [n <- :wat::core::i64]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [;; idempotently establish the store for (pk,sk,data) + the declared GSIs. Called once at
   ;; consumer init.
   (ensure-schema [self <- :wat::query::Store  req <- :wat::query::Store::EnsureSchemaRequest]
     -> :wat::query::Store::EnsureSchemaResponse :max-request-bytes 524288)

   ;; write a batch ATOMICALLY (one transaction). Each row carries its opaque data + the
   ;; (ipk,isk) it projects to for each declared GSI (supplied by the consumer's write path —
   ;; the backend cannot read `data`).
   ;;
   ;; REPLACE-BY-(pk,sk) — DynamoDB PutItem. An incoming row whose (pk,sk) already
   ;; exists completely replaces the old item; a table cannot hold two items with
   ;; one primary key. The old item's GSI projections go with it; the new item's
   ;; `index-keys` are what remain. Within a batch, later rows win. sqlite
   ;; implements this as DELETE+clear-index-projections+INSERT; mem drops the
   ;; matching StoredRow (projections are derived from surviving rows).
   (put [self <- :wat::query::Store  req <- :wat::query::Store::PutRequest]
     -> :wat::query::Store::PutResponse :max-request-bytes 10485760)

   ;; remove a batch of (pk, sk) keys ATOMICALLY (one transaction). Mirrors `put`:
   ;; batch-shaped, one txn, same max-request-bytes. GSI projections, if any, are
   ;; addressed by (pk, sk) — sqlite's `clear-index-projections` already deletes
   ;; that way; mem derives index rows from the StoredRow, so dropping the row
   ;; drops the projection. A Key is sufficient; no read-before-delete.
   (delete [self <- :wat::query::Store  req <- :wat::query::Store::DeleteRequest]
     -> :wat::query::Store::DeleteResponse :max-request-bytes 10485760)

   ;; a PAGE on the base key: pk fixed, sk in a prefix/range, ordered ASC, after `cursor`.
   (scan [self <- :wat::query::Store  req <- :wat::query::Store::ScanRequest]
     -> :wat::query::Store::ScanResponse :max-request-bytes 524288)

   ;; a PAGE on a named GSI: ipk fixed, isk in a prefix/range, ordered ASC, after `cursor`.
   (scan-index [self <- :wat::query::Store  req <- :wat::query::Store::ScanIndexRequest]
     -> :wat::query::Store::ScanIndexResponse :max-request-bytes 524288)

   ;; a COUNT on a named GSI: ipk fixed, isk in a prefix/range. Returns n, never rows.
   (count-index [self <- :wat::query::Store  req <- :wat::query::Store::CountIndexRequest]
     -> :wat::query::Store::CountIndexResponse :max-request-bytes 524288)])
