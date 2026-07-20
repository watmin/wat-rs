;; scratchpad/probe-sift-rules-stop1.wat — STOP-1 disconfirming probe for
;; docs/arc/2026/06/278-rules-engine/BRIEF-STONE-sift-rules.md.
;;
;; THE CRUX: can a `defmacro` (taking a user's `defrecord` form + rule pieces as unevaluated
;; args) emit a `do` whose children are THEMSELVES macro/special-form calls — a `defsurface`
;; (:messages = ~@ the user's def, spliced) and a `:satisfies` `defservice` (whose op compiles +
;; fires the rule on a typed item) — and have BOTH downstream forms actually expand, AND have the
;; spliced user def reach the forked worker's freeze (PROCESS locus)?
;;
;; If either fails (macro-generating-a-macro-call rejected, or the spliced def unresolved in the
;; child), STOP — this is a load-bearing finding for the whole Rules-form UX.

;; ── the outer macro ────────────────────────────────────────────────────────────────────────
;; Kept minimal: rule pieces (name/when/then) taken as separate WatAST args rather than a nested
;; `defrule` form — sidesteps a KNOWN, ALREADY-PROVEN pitfall (tests/services/probe_arc278_sift_
;; arena.wat's note, lines 80-91): a forked child does NOT inherit plain top-level defns living
;; outside the surface's `:messages`/the defservice's own internals — a separately-shipped
;; `defrule`-generated defn (like the arena's `PageState` helper) would NOT reach the child. The
;; full build must inline-compile `~@:rules` (mirroring `make-rule`) rather than call out to
;; standalone rule-fns for exactly this reason — proven here directly.
(:wat::core::defmacro :probe::rules-defsvc
  [def-form  <- :wat::WatAST
   rule-name <- :wat::WatAST
   when-vec  <- :wat::WatAST
   then-vec  <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [raw-name (:wat::core::ast-name rule-name)
     name-str (:wat::core::if (:wat::core::= (:wat::core::string::subs raw-name 0 1) ":")
                (:wat::core::string::subs raw-name 1 (:wat::core::string::length raw-name))
                raw-name)]
    `(:wat::core::do
       (:wat::core::defsurface :probe::Svc :nature :wat::kernel::Peer'
         :messages
         [~def-form
          (:wat::core::defrecord :probe::Hot [c <- :wat::core::i64])
          (:wat::core::defrecord :probe::Svc::FireRequest [c <- :wat::core::i64])
          (:wat::core::defenum :probe::Svc::FireResponse :wat::enum::Pure
            :Deduction [n <- :wat::core::i64])]
         :features
         [(fire [self <- :probe::Svc req <- :probe::Svc::FireRequest] -> :probe::Svc::FireResponse)])
       (:wat::service::defservice :probe::svc'
         :satisfies :probe::Svc
         :durable []
         :impls
         [(fire [s req]
            (:wat::core::let
              [c     (:probe::Svc::FireRequest/c req)
               ;; typed-construct an instance of the SPLICED user def — the freeze-reach proof.
               item  (:probe::Temp :c c)
               rule  (:wat::rete::make-rule ~name-str
                       (:wat::core::quote ~when-vec)
                       (:wat::core::quote ~then-vec))
               sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
               s1    (:wat::rete::insert sess0 item)
               fired (:wat::rete::fire-rules s1)
               ded   (:wat::rete::query fired :probe::Hot)]
              (:wat::service::Outcome::Reply s
                (:probe::Svc::FireResponse::Deduction (:wat::core::count ded)))))]))))

;; ── the "user" invocation — their defrecord + their rule, as literal (unevaluated) forms ────
(:probe::rules-defsvc
  (:wat::core::defrecord :probe::Temp [c <- :wat::core::i64])
  :probe::hot
  [(:probe::Temp (?c <- :c) (:wat::core::> ?c 50))]
  [(:wat::rete::insert (:probe::Hot :c ?c))])

;; ── orchestration: /start on PROCESS (prime spawn via /start), connect' (prime), fire twice ──
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h    (:probe::svc'/start :locus (:wat::spawn::process) :record (:probe::svc'::Record))
     addr (:probe::svc'::Handle/addr h)
     cli  (:wat::kernel::connect' addr)
     hot  (:probe::Svc/fire cli (:probe::Svc::FireRequest :c 99))
     cold (:probe::Svc/fire cli (:probe::Svc::FireRequest :c 1))]
    (:wat::kernel::println (:wat::core::str "hot=" hot " cold=" cold))))
