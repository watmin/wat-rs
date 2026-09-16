;; Scratch probe — arc 255 Stone P6-c-W5c, acceptance row 5.
;;
;; One correct-arity call per verb, edn-written. Confirmed byte-identical against the pre-image
;; (a real `git clone --local` of HEAD, built and run before homing — never a `git stash`).
;;
;; The four accessors below are genuinely LIVE, not phantom: `defrecord` (wat/rete.wat:374,388)
;; auto-generates a `Type/field` accessor per declared field at macro-expansion time, so none of
;; them is ever hand-typed as literal source text under src/ or wat/ — this gate's attestation
;; walk cannot see a macro-synthesized name, which is the gap being declared, not a retirement:
;; rune:lint(rete-name-unminted) :wat::rete::DerivationNode/via — macro-generated defrecord field accessor (wat/rete.wat:377); never appears as literal text, only synthesized at expansion time.
;; rune:lint(rete-name-unminted) :wat::rete::DerivationStep/pattern — macro-generated defrecord field accessor (wat/rete.wat:390); never appears as literal text, only synthesized at expansion time.
;; rune:lint(rete-name-unminted) :wat::rete::DerivationStep/bindings — macro-generated defrecord field accessor (wat/rete.wat:391); never appears as literal text, only synthesized at expansion time.
;; rune:lint(rete-name-unminted) :wat::rete::DerivationStep/constraints — macro-generated defrecord field accessor (wat/rete.wat:392); never appears as literal text, only synthesized at expansion time.

(:wat::core::defrecord :w5cprobe::Temperature  [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :w5cprobe::WindSpeed    [kph     <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :w5cprobe::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defrule :w5cprobe::cold-and-windy
  :when [(:w5cprobe::Temperature (?loc <- :location) (?c <- :celsius) (:wat::rete::i64::< ?c 20))
         (:w5cprobe::WindSpeed    (?loc <- :location) (?k <- :kph)     (:wat::rete::i64::> ?k 30))]
  :then [(:w5cprobe::ColdAndWindy :location ?loc)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; lower — returns nil on a successful lower.
    (:wat::kernel::println (:wat::string::concat "lower= " (:wat::edn::write (:wat::rete::lower (:wat::core::quote (:wat::rete::i64::> ?c 5))))))
    ;; collect-rules — one rule in :w5cprobe.
    (:wat::kernel::println (:wat::string::concat "collect-rules-len= " (:wat::edn::write (:wat::core::length (:wat::rete::collect-rules :w5cprobe)))))
    (:wat::kernel::println (:wat::string::concat "collect-rules-name= " (:wat::edn::write (:wat::rete::Rule/name (:wat::core::Option/expect (:wat::core::get (:wat::rete::collect-rules :w5cprobe) 0) "r0")))))
    ;; axis-violation — a rete-primitive comparison is pure/det/total: None on :Pure.
    (:wat::kernel::println (:wat::string::concat "axis-violation-pure= " (:wat::edn::write (:wat::rete::axis-violation (:wat::core::quote (:wat::rete::i64::> ?c 5)) :wat::rete::Axis.Pure))))
    ;; axis-violation — a non-rete-primitive head violates RetePrimitive.
    (:wat::kernel::println (:wat::string::concat "axis-violation-viol= " (:wat::edn::write (:wat::rete::axis-violation (:wat::core::quote (:wat::core::+ 1 2)) :wat::rete::Axis.RetePrimitive))))
    ;; step-payload via the explain walk (its one real caller, rete.wat's `explain`).
    (:wat::core::let [rules   (:wat::rete::collect-rules :w5cprobe)
                      session (:wat::core::match (:wat::rete::compile rules) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
                      session (:wat::core::match (:wat::rete::insert session (:w5cprobe::Temperature :celsius 10 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
                      session (:wat::core::match (:wat::rete::insert session (:w5cprobe::WindSpeed :kph 40 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
                      ex      (:wat::core::match (:wat::rete::fire-rules-explain session) [:wat::rete::FireOutcome.Fired {:value __explained} __explained] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules-explain: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules-explain: fixpoint round cap exceeded")])
                      root    (:wat::rete::explain ex (:w5cprobe::ColdAndWindy :location "Oslo"))
                      step0   (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationNode/via root) 0) "via[0]")]
      (:wat::core::do
        (:wat::kernel::println (:wat::string::concat "step-payload-pattern= " (:wat::edn::write (:wat::rete::DerivationStep/pattern step0))))
        (:wat::kernel::println (:wat::string::concat "step-payload-bindings= " (:wat::edn::write (:wat::rete::DerivationStep/bindings step0))))
        (:wat::kernel::println (:wat::string::concat "step-payload-constraints= " (:wat::edn::write (:wat::rete::DerivationStep/constraints step0))))))))
