;; wat/rete/syntax.wat — rete user syntax: query, cond, defrule, defquery.
;;
;; Loads after wat/rete.wat (Rule / Query / Session). defmacro refs are
;; order-free; query-read is a defn over Session.query-memory.
;;
;; Namespace: :wat::rete::

;; ─── query — ONE mouth: QueryNode harvest, filtered by params ───────────────

;; query-read — binding maps parked on QueryNode at fire, filtered by params.
(:wat::core::defn :wat::rete::query-read
  [session <- :wat::rete::Session
   q       <- :wat::rete::Query
   params  <- :wat::core::PersistentMap]
  -> (:wat::core::PersistentVector :- [:wat::core::PersistentMap])
  (:wat::core::let [want (:wat::rete::Query/params q)
                    got  (:wat::core::foldl
                           (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                                            k   <- :wat::core::String]
                             -> (:wat::core::PersistentVector :- [:wat::core::String])
                             (:wat::core::PersistentVector/conj acc k))
                           (:wat::core::PersistentVector)
                           (:wat::core::PersistentMap/keys params))
                    missing (:wat::rete::keys-minus want got)
                    extra   (:wat::rete::keys-minus got want)
                    _params (:wat::core::Option/expect
                              (:wat::core::if
                                (:wat::core::if (:wat::core::= (:wat::core::length missing) 0)
                                  (:wat::core::= (:wat::core::length extra) 0)
                                  false)
                                (:wat::core::Some nil)
                                :wat::core::None)
                              "query: params must match the query's :params")
                    raw (:wat::core::match
                           (:wat::core::PersistentMap/get
                             (:wat::rete::Session/query-memory session)
                             (:wat::rete::Query/name q))
                           ((:wat::core::Some pv) pv)
                           (:wat::core::None (:wat::core::PersistentVector)))]
    (:wat::core::if (:wat::core::= (:wat::core::length want) 0)
      raw
      (:wat::core::into (:wat::core::PersistentVector)
        (:wat::core::filter
          (:wat::core::fn [m <- :wat::core::PersistentMap] -> :wat::core::bool
            (:wat::core::foldl
              (:wat::core::fn [ok <- :wat::core::bool
                               k  <- :wat::core::String]
                -> :wat::core::bool
                (:wat::core::if ok
                  (:wat::core::= (:wat::core::PersistentMap/get m k)
                                 (:wat::core::PersistentMap/get params k))
                  false))
              true
              want))
          raw)))))

;; query-params-form — expand-time map builder (a MACRO: a defn head is refused
;; inside another macro by the F5 pure-combinator gate). Recurses in the
;; template so a PersistentVector of mixed keyword/value kwargs never exists.
(:wat::core::defmacro :wat::rete::query-params-form
  [acc <- :wat::WatAST
   & items <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::empty? items)
    acc
    (:wat::core::if (:wat::core::empty? (:wat::core::rest items))
      (:wat::core::macro-error "query: param kwargs must come in key/value pairs")
      (:wat::core::let [k (:wat::core::first items)
                        v (:wat::core::first (:wat::core::rest items))
                        knm (:wat::core::ast-name k)
                        kstr (:wat::core::if
                               (:wat::core::= (:wat::string::subs knm 0 1) ":")
                               (:wat::string::subs knm 1
                                 (:wat::string::length knm))
                               knm)]
        `(:wat::rete::query-params-form
           (:wat::core::PersistentMap/assoc ~acc ~kstr ~v)
           ~@(:wat::core::rest (:wat::core::rest items)))))))

;; query — ONE mouth. q is a Query. Optional Clara-shaped kwargs.
;;   (:wat::rete::query session (:wq::all-wind))
;;   (:wat::rete::query session (:wq::temps-at) :?loc "MCI")
(:wat::core::defmacro :wat::rete::query
  [session <- :wat::WatAST
   q       <- :wat::WatAST
   & rest  <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  `(:wat::rete::query-read ~session ~q
     (:wat::rete::query-params-form (:wat::core::PersistentMap) ~@rest)))

;; ─── cond — rete's OWN macro, expanding into rete's `if` ──────────────────
;;
;; BRIEF-rete-cond-is-its-own-macro.md (2026-08-05) — builder's ruling: "i think we need
;; rete's cond to just be a macro itself that expands into rete's if?". This REPLACES the
;; earlier attempt (a `freeze::env::build_env` loop that cloned core's registered `MacroDef`
;; and re-registered it under the rete name): a clone carries core's TEMPLATE, so it emitted
;; `:wat::core::if`/`:wat::core::cond` regardless of which name invoked it — a second door
;; that launders straight back through core's spelling (arc 179's `()`-vs-`nil` shape).
;;
;; This is instead its own `defmacro`, an exact copy of core's `cond` (wat/core.wat:1237)
;; with ONLY the emitted head keywords moved to the rete namespace: every backtick-quoted
;; `(:wat::core::if …)` becomes `(:wat::rete::core::if …)`, and every recursive
;; `(:wat::core::cond ~@…)` becomes `(:wat::rete::core::cond ~@…)` — including the
;; annotated-form branch's recursive call, which recurses WITHOUT emitting an `if` and is
;; therefore the spelling most easily missed by eye. The macro-error text on a
;; non-exhaustive clause list is kept byte-identical (still a located expansion error, same
;; first-class primitive). The `:else` structural comparison, the `List?` head test, and the
;; annotated-form strip are unchanged LOGIC — they call genuine core primitives
;; (empty?/List?/first/second/rest/let/=), not the emitted if/cond spelling, so they stay
;; `:wat::core::`.
;;
;; Rete `if` (`:wat::rete::core::if`, RETE_OPS `Form` row) re-dispatches at runtime to core
;; `if`'s genuine eval arm, so it fires inside a real `defrule` — proven by
;; `wat-scripts/scratch-pad/probe-rete-if-in-where.wat` (`hits=1`). Expanding into rete `if`
;; is therefore a correct target, not a hope.
;;
;; Present: `defrule` still quotes `:when`/`:then` vectors, but `expand_form` on
;; `make-rule` (`src/macros/expand.rs` `expand_make_rule` / `expand_make_rule_when`,
;; `src/resolve/boundary.rs` `Boundary::MakeRule`) expands only each `where` body's
;; code — not fact patterns, not `:then`. A `cond` written inside a `where` is legal
;; and expands to rete `if`. This macro is that expansion. Ordinary (non-quoted) rete
;; `cond` uses the same template.
(:wat::core::defmacro :wat::rete::core::cond
  [& clauses <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::empty? clauses)
    ;; empty clause list — non-exhaustive / no terminal :else. Same located diagnostic as
    ;; core's cond, byte-identical text.
    (:wat::core::macro-error "cond: non-exhaustive — needs a terminal :else arm")
    (:wat::core::if (:wat::core::List? (:wat::core::first clauses))
      ;; First clause is a List — bare form: (cond (test body) … (:else body))
      (:wat::core::let [arm  (:wat::core::first clauses)
                        head (:wat::core::first arm)]
        (:wat::core::if (:wat::core::List? head)
          ;; test arm — head is a sub-list like (= 1 2): (if head body (cond rest…)) —
          ;; rete-spelled all the way down.
          `(:wat::rete::core::if
              ~head
              ~(:wat::core::second arm)
              (:wat::rete::core::cond ~@(:wat::core::rest clauses)))
          ;; non-List head — detect :else by structural comparison with the :else keyword form.
          (:wat::core::if (:wat::core::= head (:wat::core::first `(:else)))
            ;; :else terminal arm — emit body unconditionally
            (:wat::core::second arm)
            ;; other non-List head — treat as test arm (v1 fallback for malformed input)
            `(:wat::rete::core::if
                ~head
                ~(:wat::core::second arm)
                (:wat::rete::core::cond ~@(:wat::core::rest clauses))))))
      ;; First clause is NOT a List (it is the -> symbol) — annotated form. Strip -> and :T
      ;; (first two elements) and re-expand as bare cond — the recursive spelling most easily
      ;; missed by eye, because it recurses WITHOUT emitting an if.
      `(:wat::rete::core::cond ~@(:wat::core::rest (:wat::core::rest clauses))))))

;; ─── make-rule + defrule ────────────────────────────────────────────────────

;; make-rule — runtime helper: split quoted vector nodes into PVs and build a Rule.
;; when-ast / then-ast are quoted VECTOR nodes; ast->children yields the per-element WatASTs.
;; Converts the std Vector from ast->children to a (PersistentVector :- [WatAST]) via foldl/conj.
(:wat::core::defn :wat::rete::make-rule
  [name     <- :wat::core::String
   when-ast <- :wat::WatAST
   then-ast <- :wat::WatAST]
  -> :wat::rete::Rule
  (:wat::core::let [lhs-pv (:wat::core::foldl
                               (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::WatAST])
                                                c   <- :wat::WatAST]
                                 -> (:wat::core::PersistentVector :- [:wat::WatAST])
                                 (:wat::core::PersistentVector/conj acc c))
                               (:wat::core::PersistentVector)
                               (:wat::core::ast->children when-ast))
                    rhs-pv (:wat::core::foldl
                               (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::WatAST])
                                                c   <- :wat::WatAST]
                                 -> (:wat::core::PersistentVector :- [:wat::WatAST])
                                 (:wat::core::PersistentVector/conj acc c))
                               (:wat::core::PersistentVector)
                               (:wat::core::ast->children then-ast))]
    (:wat::rete::Rule :name name :lhs lhs-pv :rhs rhs-pv)))

;; defrule — homoiconic rule macro: expand a readable rule form into a zero-arg defn
;; returning a Rule. The zero-arg fn is the reflection marker for collect-rules (stone 5b).
;;
;; Surface (arc 278 DESIGN-STONE-then-is-a-vector-of-singular-facts, Stone A):
;;   (:wat::rete::defrule :weather::cold-and-windy
;;     :when [<cond1> <cond2> …]
;;     :then [<fact1> <fact2> …])
;;
;; Both :when and :then are VECTORS. Each :then member is a single fact to insert — the
;; `:wat::rete::insert` RHS-marker wrapper is GONE (the engine is inserts-only by doctrine,
;; so naming it per entry said nothing; see eval_insert.rs's `build_insert_fact`).
;;
;; Expands to:
;;   (:wat::core::defn :weather::cold-and-windy [] -> :wat::rete::Rule
;;     (:wat::rete::make-rule "weather::cold-and-windy"
;;       (:wat::core::quote [<cond1> <cond2> …])
;;       (:wat::core::quote [<fact1> <fact2> …])))
;;
;; The macro is kept TRIVIAL: it quotes both vectors as-is — symmetric with when-vec.
;; make-rule (above) does the per-element split at runtime.
;; Assumes canonical :when then :then order (STOP if a general parse is needed).
(:wat::core::defmacro :wat::rete::defrule
  [name <- :wat::WatAST
   & rest <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let [;; name-str: ast-name returns the raw keyword text WITH leading colon;
                    ;; strip it to get the bare FQDN matching (:wat::core::type fact).
                    raw-name  (:wat::core::ast-name name)
                    ;; strip-leading-colon inline (can't call user-defn from program-body macro)
                    name-str  (:wat::core::if (:wat::core::= (:wat::string::subs raw-name 0 1) ":")
                                 (:wat::string::subs raw-name 1 (:wat::string::length raw-name))
                                 raw-name)
                    ;; rest = (:when <when-vec> :then <then-vec>); canonical order assumed.
                    when-vec  (:wat::core::Option/expect
                                 (:wat::core::get rest 1)
                                 "defrule: missing :when conditions vector")
                    then-vec  (:wat::core::Option/expect
                                 (:wat::core::get rest 3)
                                 "defrule: missing :then facts vector")]
    `(:wat::core::defn ~name [] -> :wat::rete::Rule
       (:wat::rete::make-rule ~name-str
         (:wat::core::quote ~when-vec)
         (:wat::core::quote ~then-vec)))))

;; make-query — split quoted :params / :when vectors into a Query.
(:wat::core::defn :wat::rete::make-query
  [name       <- :wat::core::String
   params-ast <- :wat::WatAST
   when-ast   <- :wat::WatAST]
  -> :wat::rete::Query
  (:wat::core::let [params-pv (:wat::core::foldl
                                 (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                                                  p   <- :wat::WatAST]
                                   -> (:wat::core::PersistentVector :- [:wat::core::String])
                                   (:wat::core::PersistentVector/conj acc (:wat::core::ast-name p)))
                                 (:wat::core::PersistentVector)
                                 (:wat::core::ast->children params-ast))
                    lhs-pv (:wat::core::foldl
                              (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::WatAST])
                                               c   <- :wat::WatAST]
                                -> (:wat::core::PersistentVector :- [:wat::WatAST])
                                (:wat::core::PersistentVector/conj acc c))
                              (:wat::core::PersistentVector)
                              (:wat::core::ast->children when-ast))]
    (:wat::rete::Query :name name :params params-pv :lhs lhs-pv)))

;; defquery — Clara `[defquery q [:?loc] …]`. Zero-arg defn returning Query.
;;   (:wat::rete::defquery :wq::temps-at
;;     :params [?loc]
;;     :when   […])
(:wat::core::defmacro :wat::rete::defquery
  [name <- :wat::WatAST
   & rest <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let [raw-name  (:wat::core::ast-name name)
                    name-str  (:wat::core::if (:wat::core::= (:wat::string::subs raw-name 0 1) ":")
                                 (:wat::string::subs raw-name 1 (:wat::string::length raw-name))
                                 raw-name)
                    params-vec (:wat::core::Option/expect
                                  (:wat::core::get rest 1)
                                  "defquery: missing :params vector")
                    when-vec   (:wat::core::Option/expect
                                  (:wat::core::get rest 3)
                                  "defquery: missing :when conditions vector")]
    `(:wat::core::defn ~name [] -> :wat::rete::Query
       (:wat::rete::make-query ~name-str
         (:wat::core::quote ~params-vec)
         (:wat::core::quote ~when-vec)))))

;; ⚠ THESE LIVE HERE, NOT IN wat/rete.wat, AND THE LOAD ORDER IS WHY.
;; rete.wat loads at STDLIB_FILES pos 36; compile-all is defined in wat/rete/compile.wat
;; (pos 37), insert-all in oracle/insert.wat, fire-rules in oracle/fire.wat (pos 43) — all
;; AFTER it. Placing these defns in rete.wat produced three real eval-time load-order
;; violations (arc 275's gate caught them by name). syntax.wat is the LAST rete file to
;; load, so every dependency is already defined — and it is already the caller-facing
;; mouth (`query` lives here), which is where a caller-facing scope form belongs anyway.
;; ── scoped work over a compiled network ─────────────────────────────────────
;; DESIGN-STONE-scoped-work-over-a-network: rete has an ACQUIRE/RELEASE pair with nothing
;; pairing it — compile-all already takes an intern lease (DESIGN-STONE-arm-at-compile) and
;; release-session drops it; every caller had to remember the release by hand. These two forms
;; are that missing shape, matching :wat::io::with-open-file: a plain defn that acquires its
;; own resource and releases it after the body runs.

;; with-network — compiles rules+queries into an armed Session, hands it to body-fn, releases
;; the lease after. Returns body-fn's result.
;; Both forms COMPILE their own network; neither accepts a Session. This is forced, not
;; stylistic: compile-all already arms, and arm-session's HIT path INCREMENTS the lease
;; (arm.rs:709) — so a wrapper handed an already-compiled Session could only add a lease it
;; then removes, leaking the lease compile-all took. Acquire and release must be the same scope.
;; The body's param is named `base` by convention, not signature: it is a VALUE the body can
;; hold and thread forward (accumulating across units is permitted here) — as opposed to
;; with-overlay's `overlay`, a VERB the body calls, which forbids it by having no base in scope.
(:wat::core::defn :wat::rete::with-network :- [T]
  [rules   <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   queries <- (:wat::core::PersistentVector :- [:wat::rete::Query])
   body-fn <- [:wat::rete::Session :-> T]]
  -> T
  (:wat::core::let [base   (:wat::rete::compile-all rules queries)
                    result (body-fn base)]
    (:wat::core::do
      (:wat::rete::release-session base)
      result)))

;; with-overlay — same acquire/release scope as with-network (built ON it: one release site,
;; not two), plus a structural guarantee: the body receives not the Session but an Overlay
;; (facts -> fired Session), always re-seeded from the compiled base. The base is never in
;; scope, so threading one unit of work's facts into the next has no form. `(overlay facts)`
;; returns a FIRED Session, not a seeded one — the caller never wants the unfired form.
;; N distinct units of work still cost ONE network build and ONE lease: the Session is a fact
;; overlay over circuits it does not own (arm.rs:572) and is immutable, so each call re-seeds
;; from `base` and `base` itself is never touched.
(:wat::core::defn :wat::rete::with-overlay :- [T]
  [rules   <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   queries <- (:wat::core::PersistentVector :- [:wat::rete::Query])
   body-fn <- [:wat::rete::Overlay :-> T]]
  -> T
  (:wat::rete::with-network rules queries
    (:wat::core::fn [base <- :wat::rete::Session] -> T
      (body-fn
        (:wat::core::fn [facts <- (:wat::core::PersistentVector :- [:wat::core::Record])]
          -> :wat::rete::Session
          (:wat::rete::fire-rules (:wat::rete::insert-all base facts)))))))
