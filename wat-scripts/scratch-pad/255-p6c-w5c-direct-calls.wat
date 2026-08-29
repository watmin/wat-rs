;; Scratch probe — arc 255 Stone P6-c-W5c, acceptance row 5.
;;
;; One correct-arity call per verb, edn-written. Confirmed byte-identical against the pre-image
;; (a real `git clone --local` of HEAD, built and run before homing — never a `git stash`).

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
    (:wat::kernel::println (:wat::string::concat "axis-violation-pure= " (:wat::edn::write (:wat::rete::axis-violation (:wat::core::quote (:wat::rete::i64::> ?c 5)) :wat::rete::Axis::Pure))))
    ;; axis-violation — a non-rete-primitive head violates RetePrimitive.
    (:wat::kernel::println (:wat::string::concat "axis-violation-viol= " (:wat::edn::write (:wat::rete::axis-violation (:wat::core::quote (:wat::core::+ 1 2)) :wat::rete::Axis::RetePrimitive))))
    ;; step-payload via the explain walk (its one real caller, rete.wat's `explain`).
    (:wat::core::let [rules   (:wat::rete::collect-rules :w5cprobe)
                      session (:wat::rete::compile rules)
                      session (:wat::rete::insert session (:w5cprobe::Temperature :celsius 10 :location "Oslo"))
                      session (:wat::rete::insert session (:w5cprobe::WindSpeed :kph 40 :location "Oslo"))
                      ex      (:wat::rete::fire-rules-explain session)
                      root    (:wat::rete::explain ex (:w5cprobe::ColdAndWindy :location "Oslo"))
                      step0   (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationNode/via root) 0) "via[0]")]
      (:wat::core::do
        (:wat::kernel::println (:wat::string::concat "step-payload-pattern= " (:wat::edn::write (:wat::rete::DerivationStep/pattern step0))))
        (:wat::kernel::println (:wat::string::concat "step-payload-bindings= " (:wat::edn::write (:wat::rete::DerivationStep/bindings step0))))
        (:wat::kernel::println (:wat::string::concat "step-payload-constraints= " (:wat::edn::write (:wat::rete::DerivationStep/constraints step0))))))))
