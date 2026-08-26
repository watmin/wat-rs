;; PROBE — does a `(Vector :- [WatAST])` cross a real defservice op boundary, and can the far side
;; build a world from it and fire the rule?
;;
;; WHY THIS EXISTS. `probe-arc278-rules-ship-as-declared-payload.wat` proved a declared payload
;; carries a rule AND the fn its `where` calls — but IN-PROCESS, through `eval-with-defs!` called
;; directly. The payload never touched a pipe. Every `(Vector :- [WatAST])` in the corpus is a MACRO
;; PARAMETER or a local fold accumulator; not one is a `defsurface :features` field, a
;; `:messages` entry, or a service op parameter. So WatAST-at-the-boundary is untested in both
;; directions — and task #77 is on the board precisely because that coverage was LOST
;; ("does a defservice `:durable` accept `(Vector :- [WatAST])`? No fixture asks any more").
;;
;; ⚠ THE LOCUS IS `process` ON PURPOSE. A thread peer shares the parent's address space and could
;; hand the value across without an EDN round-trip, which would prove nothing about the wire. A
;; process forces encode → pipe → decode. If this probe is ever "sped up" to a thread locus it
;; stops testing its own subject.
;;
;; ⚠ NON-VACUITY — two arms differing in ONE form of the payload:
;;   SUBJECT — defs carry the records, `:usr::big?`, AND the rule. Expect `DERIVED n=1`.
;;   CONTROL — the same defs with `:usr::big?` omitted. Expect `REJECTED check-failed`.
;; SUBJECT=1 is itself strong: it requires the vector to have arrived intact (an empty or
;; mangled vector gives `collect-rules` nothing → 0, or a check failure). CONTROL then proves the
;; service is evaluating what it RECEIVED rather than anything ambient in its own world.
;;
;; ⚠ WHAT A GREEN RUN DOES NOT PROVE: nothing about payload SIZE (one small rule set), nothing
;; about `:max-request-bytes` pressure at scale, and nothing about per-connection isolation —
;; this is one connection, one install. Those are separate questions.
;;
;; ⛔ MEASURED 2026-08-12 — REFUTED, and BOTH arms failed identically, so this is the mechanism
;; failing rather than the differential firing:
;;   "SUBJECT (helper IN payload) => LOST disconnected"
;;   "CONTROL (helper OMITTED)    => LOST disconnected"
;;
;; The file's own closing line says any other pairing refutes it. It does.
;;
;; ROOT ISOLATED by `probe-arc278-watast-on-the-wire-decomposed.wat` — read its header for the
;; full chain. In short: the service shape here is fine (`echo(i64)` round-trips), the payload
;; semantics were never reached, and the cause is a DECODE-VALIDATION BUG, not a transport limit:
;;
;;   REQUEST-MALFORMED  expected=:wat::WatAST  got=List   at path ["defs" "[0]"]
;;
;; The frame arrived and the validator walked into element 0, so the value crossed. It then
;; compared the decoded kind name "List" against the declared ":wat::WatAST" and refused — but a
;; List IS a WatAST. A name comparison, not a transport boundary.
;;
;; ⚠ DO NOT "FIX" THIS BY SHIPPING TEXT. That was the apparatus's first instinct here and it is a
;; workaround for a bug; the builder refused the premise (*"why is (Vector WatAST) a problem?"*).
;; Forms are pure EDN and DO ship (wat/repl.wat:20-22). `write-forms`/`read-string` remain the
;; right answer for CHUNKED transmission at size (the lifecycle stone's own scope), which is a
;; different question from whether a form can be a typed field at all.
;;
;; ★ THIS FILE IS KEPT RED-BY-MEASUREMENT, not deleted. It is the acceptance test for the decode
;; fix: once a `(Vector :- [WatAST])` validates, this must print `DERIVED n=1` / `REJECTED
;; check-failed`. Rewrite this verdict then; do not leave it standing once it goes green.
;;
;; ★ MEASURED AGAIN 2026-08-12, after the identity arm (DESIGN-STONE-watast-is-the-wire.md,
;; `edn::render::edn_to_typed_value_inner`'s `:wat::WatAST` case): STILL RED, UNCHANGED —
;;
;;   "SUBJECT (helper IN payload) => LOST disconnected"
;;   "CONTROL (helper OMITTED)    => LOST disconnected"
;;
;; NOT a regression and NOT evidence the identity arm is wrong: `probe-arc278-watast-on-the-
;; wire-decomposed.wat`'s THREAD arm (which shares the exact same walker this stone fixed) now
;; reads `Ok n=3`, proving the walker itself is correct. This probe is process-locus ONLY (by
;; design — see the header above), and process locus has a SEPARATE, upstream defect: the
;; generic message-dispatch decode (`edn::render::edn_to_value_caps`'s `Edn::Symbol` arm)
;; refuses any EDN Symbol before a type-directed walk is even reachable, and
;; a real WatAST form always contains symbols. Full chain in the decomposed probe's own header
;; (its "FINDING 3"), captured via `strace -f` on the child. Out of this stone's blast radius
;; (fixing it means loosening the general untyped reader, not "one arm in one walker").

;; ── the surface — `defs` is the SUBJECT: a (Vector :- [WatAST]) as a request field ───────────────
(:wat::core::defsurface :probe::RuleWire :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::RuleWire::InstallRequest
     [defs <- (:wat::core::Vector :- [:wat::WatAST])])
   (:wat::core::defenum :probe::RuleWire::InstallResponse :wat::enum::Pure
     :Derived          [n <- :wat::core::i64]
     :Rejected         [reason <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(install [self <- :probe::RuleWire  req <- :probe::RuleWire::InstallRequest] -> :probe::RuleWire::InstallResponse :max-request-bytes 524288)])

;; ── the evaluand the SERVICE runs, in the world built from what it was handed ──────────────
;; `collect-rules` reflects the namespace of the world it is standing in — the service never
;; names a rule; it asks the world the client shipped it. 150 > 100 derives exactly one Hot.
(:wat::core::defn :probe::evaluand [] -> :wat::WatAST
  (:wat::core::quote
    (:wat::core::length
      (:wat::rete::query
        (:wat::rete::fire-rules
          (:wat::rete::insert
            (:wat::rete::compile-all
              (:wat::rete::collect-rules :usr)
              (:wat::core::PersistentVector
                (:wat::rete::make-query "usr::Hot"
                  (:wat::core::quote [])
                  (:wat::core::quote [(:usr::Hot)]))))
            (:usr::Temp :c 150)))
        (:wat::rete::make-query "usr::Hot"
          (:wat::core::quote [])
          (:wat::core::quote [(:usr::Hot)]))))))

;; ── the service — receives defs off the wire, builds a world, fires, replies with the count ──
(:wat::service::defservice :probe::rulewiresvc
  :satisfies :probe::RuleWire
  :durable   [installs <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::rulewiresvc::Record] -> :probe::rulewiresvc::State
          (:probe::rulewiresvc::State :durable record))
  :impls
  [(install [s ctx req]
     (:wat::core::match
       (:wat::eval-with-defs! (:probe::evaluand) (:probe::RuleWire::InstallRequest/defs req))
       (:wat::eval::FormOutcome::Declared
         (:wat::service::Outcome::Reply s (:probe::RuleWire::InstallResponse::Rejected "declared")))
       ((:wat::eval::FormOutcome::Evaluated v)
         (:wat::service::Outcome::Reply s (:probe::RuleWire::InstallResponse::Derived v)))
       ((:wat::eval::FormOutcome::CheckFailed _cause)
         (:wat::service::Outcome::Reply s (:probe::RuleWire::InstallResponse::Rejected "check-failed")))
       ((:wat::eval::FormOutcome::Raised _cause)
         (:wat::service::Outcome::Reply s (:probe::RuleWire::InstallResponse::Rejected "raised")))))])

;; ── the two payloads, differing in ONE form ───────────────────────────────────────────────
(:wat::core::defn :probe::payload-complete [] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::Vector :wat::WatAST
    (:wat::core::quote (:wat::core::defrecord :usr::Temp [c <- :wat::core::i64]))
    (:wat::core::quote (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64]))
    (:wat::core::quote
      (:wat::rete::core::defn :usr::big? [n <- :wat::core::i64] -> :wat::core::bool
        (:wat::rete::i64::> n 100)))
    (:wat::core::quote
      (:wat::rete::defrule :usr::rule-userfn
        :when [(:usr::Temp (?c <- :c)) (:wat::rete::where (:usr::big? ?c))]
        :then [(:usr::Hot :c ?c)]))))

(:wat::core::defn :probe::payload-missing-helper [] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::Vector :wat::WatAST
    (:wat::core::quote (:wat::core::defrecord :usr::Temp [c <- :wat::core::i64]))
    (:wat::core::quote (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64]))
    (:wat::core::quote
      (:wat::rete::defrule :usr::rule-userfn
        :when [(:usr::Temp (?c <- :c)) (:wat::rete::where (:usr::big? ?c))]
        :then [(:usr::Hot :c ?c)]))))

;; ── the client ────────────────────────────────────────────────────────────────────────────
(:wat::core::defn :probe::connect! [h <- :probe::rulewiresvc::Handle] -> :probe::RuleWire
  (:wat::core::match (:wat::kernel::connect (:probe::rulewiresvc::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::install!
  [label <- :wat::core::String
   defs  <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::core::nil
  (:wat::core::let
    [h (:probe::rulewiresvc/start :locus (:wat::spawn::process)
         :record (:probe::rulewiresvc::Record :installs 0))
     c (:probe::connect! h)]
    (:wat::core::match (:probe::RuleWire/install c (:probe::RuleWire::InstallRequest :defs defs))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::RuleWire::InstallResponse::Derived n)
            (:wat::kernel::println
              (:wat::string::concat label " => DERIVED n=" (:wat::i64::to-string n))))
          ((:probe::RuleWire::InstallResponse::Rejected reason)
            (:wat::kernel::println (:wat::string::concat label " => REJECTED " reason)))
          ((:probe::RuleWire::InstallResponse::RequestTooLarge bytes cap)
            (:wat::kernel::println
              (:wat::string::concat label " => REQUEST-TOO-LARGE bytes="
                (:wat::i64::to-string bytes) " cap=" (:wat::i64::to-string cap))))
          ((:probe::RuleWire::InstallResponse::RequestMalformed _p expected got)
            (:wat::kernel::println
              (:wat::string::concat label " => REQUEST-MALFORMED expected=" expected " got=" got)))))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::println
          (:wat::string::concat label " => LOST " (:wat::kernel::LociDiedError/message cause))))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::println (:wat::string::concat label " => STOPPED before reply")))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::println (:wat::string::concat label " => CLOSED before reply"))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:probe::install! "SUBJECT (helper IN payload)" (:probe::payload-complete))
    (:probe::install! "CONTROL (helper OMITTED)   " (:probe::payload-missing-helper))
    (:wat::kernel::println
      "READ: SUBJECT DERIVED n=1 AND CONTROL REJECTED check-failed => a Vector<WatAST> crosses a process service boundary intact and the far side fires the rule. Any other pairing refutes it.")))
