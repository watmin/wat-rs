;; Validate the RICH arena rule graph's per-category deduction math BEFORE wiring into the
;; sift-rules-defsvc macro / service arena. Direct rete calls (no service), one seed per category,
;; printing counts for Suspect/Flagged (Lemmas, must be internal/never asserted as returned) and
;; Anomaly/Breach/Overflow/Critical (Deductions, terminal).
;;
;; Graph:
;;   Event -where(2-level: client.geo.country)-> Suspect (Lemma, fired-upon)
;;   Suspect -where-> Anomaly (Deduction, terminal, HIGH total-ns threshold)
;;   Suspect -where-> Breach  (Deduction, terminal, LOW total-ns threshold)  [graded parallel]
;;   Event -bytes-> Overflow (Deduction, terminal, direct single-level)
;;   Event -where(method==POST)-> Flagged (Lemma, fired-upon)
;;   Flagged -where(dns-ns)-> Critical (Deduction, terminal, 2nd cascade)

(:wat::core::defrecord :arena::Geo    [country <- :wat::core::String  asn <- :wat::core::i64])
(:wat::core::defrecord :arena::Client [ip <- :wat::core::String  geo <- :arena::Geo  reputation <- :wat::core::i64])
(:wat::core::defenum   :arena::Method :wat::enum::Pure :GET :POST :PUT :DELETE)
(:wat::core::defrecord :arena::Route  [method <- :arena::Method  path <- :wat::core::String  status <- :wat::core::i64])
(:wat::core::defrecord :arena::Timing [dns-ns <- :wat::core::i64  total-ns <- :wat::core::i64])
(:wat::core::defrecord :arena::Event  [client <- :arena::Client  route <- :arena::Route  timing <- :arena::Timing  bytes <- :wat::core::i64])

(:wat::core::defrecord :arena::Suspect [client <- :arena::Client  route <- :arena::Route  timing <- :arena::Timing  bytes <- :wat::core::i64])
(:wat::core::defrecord :arena::Flagged [client <- :arena::Client  route <- :arena::Route  timing <- :arena::Timing])
(:wat::core::defrecord :arena::Anomaly  [client <- :arena::Client])
(:wat::core::defrecord :arena::Breach   [client <- :arena::Client])
(:wat::core::defrecord :arena::Overflow [bytes  <- :wat::core::i64])
(:wat::core::defrecord :arena::Critical [client <- :arena::Client])

(:wat::rete::defrule :arena::suspect-rule
  :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing) (?bytes <- :bytes))
         (:wat::rete::where (:wat::core::> (:arena::Timing/total-ns ?timing) 500000))
         (:wat::rete::where (:wat::core::< (:arena::Client/reputation ?client) 0))
         (:wat::rete::where (:wat::core::= (:arena::Geo/country (:arena::Client/geo ?client)) "XX"))]
  :then [(:arena::Suspect :client ?client :route ?route :timing ?timing :bytes ?bytes)])

(:wat::rete::defrule :arena::anomaly-rule
  :when [(:arena::Suspect (?client <- :client) (?route <- :route) (?timing <- :timing))
         (:wat::rete::where (:wat::core::> (:arena::Timing/total-ns ?timing) 5000000))
         (:wat::rete::where (:wat::core::= (:arena::Route/status ?route) 200))]
  :then [(:arena::Anomaly :client ?client)])

(:wat::rete::defrule :arena::breach-rule
  :when [(:arena::Suspect (?client <- :client) (?route <- :route) (?timing <- :timing))
         (:wat::rete::where (:wat::core::> (:arena::Timing/total-ns ?timing) 2000000))
         (:wat::rete::where (:wat::core::= (:arena::Route/status ?route) 200))]
  :then [(:arena::Breach :client ?client)])

(:wat::rete::defrule :arena::overflow-rule
  :when [(:arena::Event (?bytes <- :bytes))
         (:wat::rete::where (:wat::core::> ?bytes 10000000))]
  :then [(:arena::Overflow :bytes ?bytes)])

(:wat::rete::defrule :arena::flagged-rule
  :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing))
         (:wat::rete::where (:wat::core::= (:arena::Route/method ?route) :arena::Method::POST))
         (:wat::rete::where (:wat::core::< (:arena::Client/reputation ?client) -50))]
  :then [(:arena::Flagged :client ?client :route ?route :timing ?timing)])

(:wat::rete::defrule :arena::critical-rule
  :when [(:arena::Flagged (?client <- :client) (?timing <- :timing))
         (:wat::rete::where (:wat::core::> (:arena::Timing/dns-ns ?timing) 300000))]
  :then [(:arena::Critical :client ?client)])

(:wat::rete::defquery :arena::q-Suspect
  :params []
  :when [(?fact <- :arena::Suspect)])


(:wat::rete::defquery :arena::q-Flagged
  :params []
  :when [(?fact <- :arena::Flagged)])


(:wat::rete::defquery :arena::q-Anomaly
  :params []
  :when [(?fact <- :arena::Anomaly)])


(:wat::rete::defquery :arena::q-Breach
  :params []
  :when [(?fact <- :arena::Breach)])


(:wat::rete::defquery :arena::q-Overflow
  :params []
  :when [(?fact <- :arena::Overflow)])


(:wat::rete::defquery :arena::q-Critical
  :params []
  :when [(?fact <- :arena::Critical)])


(:wat::core::defn :arena::mk
  [ctry <- :wat::core::String rep <- :wat::core::i64 method <- :arena::Method status <- :wat::core::i64
   total-ns <- :wat::core::i64 dns-ns <- :wat::core::i64 bytes <- :wat::core::i64]
  -> :arena::Event
  (:arena::Event
    :client (:arena::Client :ip "1.2.3.4" :geo (:arena::Geo :country ctry :asn 64500) :reputation rep)
    :route  (:arena::Route :method method :path "/api" :status status)
    :timing (:arena::Timing :dns-ns dns-ns :total-ns total-ns)
    :bytes  bytes))

(:wat::core::defn :arena::fire-one [template <- :wat::rete::Session seed <- :arena::Event] -> :wat::core::String
  (:wat::core::let
    [fired (:wat::core::match (:wat::rete::fire-rules (:wat::rete::insert template seed)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     su (:wat::core::length (:wat::rete::query fired (:arena::q-Suspect)))
     fl (:wat::core::length (:wat::rete::query fired (:arena::q-Flagged)))
     an (:wat::core::length (:wat::rete::query fired (:arena::q-Anomaly)))
     br (:wat::core::length (:wat::rete::query fired (:arena::q-Breach)))
     ov (:wat::core::length (:wat::rete::query fired (:arena::q-Overflow)))
     cr (:wat::core::length (:wat::rete::query fired (:arena::q-Critical)))
     total (:wat::core::+ an (:wat::core::+ br (:wat::core::+ ov cr)))]
    (:wat::core::string::concat "Su=" (:wat::core::string::concat (:wat::core::str su)
      (:wat::core::string::concat " Fl=" (:wat::core::string::concat (:wat::core::str fl)
        (:wat::core::string::concat " An=" (:wat::core::string::concat (:wat::core::str an)
          (:wat::core::string::concat " Br=" (:wat::core::string::concat (:wat::core::str br)
            (:wat::core::string::concat " Ov=" (:wat::core::string::concat (:wat::core::str ov)
              (:wat::core::string::concat " Cr=" (:wat::core::string::concat (:wat::core::str cr)
                (:wat::core::string::concat " TOTAL=" (:wat::core::str total))))))))))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::core::PersistentVector
             (:arena::suspect-rule) (:arena::anomaly-rule) (:arena::breach-rule)
             (:arena::overflow-rule) (:arena::flagged-rule) (:arena::critical-rule))
     template (:wat::rete::compile-all rules (:wat::core::PersistentVector (:arena::q-Suspect) (:arena::q-Flagged) (:arena::q-Anomaly) (:arena::q-Breach) (:arena::q-Overflow) (:arena::q-Critical)))]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "cat0 clean            (want 0): " (:arena::fire-one template (:arena::mk "US" 50 :arena::Method::GET 200 100000 10000 1000))))
      (:wat::kernel::println (:wat::core::string::concat "cat1 hot-wrong-country (want 0): " (:arena::fire-one template (:arena::mk "US" -10 :arena::Method::GET 200 6000000 10000 1000))))
      (:wat::kernel::println (:wat::core::string::concat "cat2 suspect+breach    (want 1): " (:arena::fire-one template (:arena::mk "XX" -10 :arena::Method::GET 200 3000000 10000 1000))))
      (:wat::kernel::println (:wat::core::string::concat "cat3 susp+anom+breach  (want 2): " (:arena::fire-one template (:arena::mk "XX" -20 :arena::Method::GET 200 6000000 10000 1000))))
      (:wat::kernel::println (:wat::core::string::concat "cat4 susp-bad-status   (want 0): " (:arena::fire-one template (:arena::mk "XX" -5  :arena::Method::GET 404 6000000 10000 1000))))
      (:wat::kernel::println (:wat::core::string::concat "cat5 overflow          (want 1): " (:arena::fire-one template (:arena::mk "US" 50 :arena::Method::GET 200 100000 10000 15000000))))
      (:wat::kernel::println (:wat::core::string::concat "cat6 flagged+critical  (want 1): " (:arena::fire-one template (:arena::mk "US" -60 :arena::Method::POST 200 100000 500000 1000))))
      (:wat::kernel::println (:wat::core::string::concat "cat7 flagged-only      (want 0): " (:arena::fire-one template (:arena::mk "US" -60 :arena::Method::POST 200 100000 100000 1000))))
      (:wat::kernel::println (:wat::core::string::concat "cat8 everything        (want 4): " (:arena::fire-one template (:arena::mk "XX" -60 :arena::Method::POST 200 9000000 900000 20000000))))
      (:wat::kernel::println (:wat::core::string::concat "cat9 clean-variety     (want 0): " (:arena::fire-one template (:arena::mk "CA" 10 :arena::Method::PUT 500 50000 20000 500)))))))
