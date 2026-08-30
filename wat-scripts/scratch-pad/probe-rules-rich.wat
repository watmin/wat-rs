;; Arena-richness crux (corrected): reason over RICH, NESTED records via `(:wat::rete::where <expr>)`
;; for the compound nested-field constraints (accessors OK — the expr is fenced pure∧det), and carry
;; BOUND sub-records forward in data-carrying Lemmas (the RHS is bindings+literals-only, so a Lemma
;; carries the bound ?client/?route records, and the downstream rule `where`-filters THEM). Alpha-only:
;; each rule matches ONE fact (the Event, or the derived Suspect); richness lives in the where-exprs.

(:wat::core::defrecord :usr::Geo    [country <- :wat::core::String  asn <- :wat::core::i64])
(:wat::core::defrecord :usr::Client [ip <- :wat::core::String  geo <- :usr::Geo  reputation <- :wat::core::i64])
(:wat::core::defenum   :usr::Method :wat::enum::Pure :GET :POST :PUT :DELETE)
(:wat::core::defrecord :usr::Route  [method <- :usr::Method  path <- :wat::core::String  status <- :wat::core::i64])
(:wat::core::defrecord :usr::Event  [client <- :usr::Client  route <- :usr::Route  latency-ns <- :wat::core::i64])

;; a data-carrying Lemma (a gate carrying bound sub-records forward) + a terminal Deduction
(:wat::core::defrecord :usr::Suspect [client <- :usr::Client  route <- :usr::Route  lat <- :wat::core::i64])
(:wat::core::defrecord :usr::Anomaly [client <- :usr::Client])

;; RULE 1 (Event → Lemma): match the Event, `where`-filter over the NESTING (latency, client.reputation,
;; client.geo.country — a 2-level nested accessor), carry the bound client/route + lat into a Suspect.
(:wat::rete::defrule :usr::suspect
  :when [(:usr::Event (?client <- :client) (?route <- :route) (?lat <- :latency-ns))
         (:wat::rete::where (:wat::core::> ?lat 1000000))
         (:wat::rete::where (:wat::core::< (:usr::Client/reputation ?client) 0))
         (:wat::rete::where (:wat::core::= (:usr::Geo/country (:usr::Client/geo ?client)) "XX"))]
  :then [(:usr::Suspect :client ?client :route ?route :lat ?lat)])

;; RULE 2 (Lemma → Deduction, the cascade): fire on the derived Suspect, `where`-filter its carried
;; records (very-high latency AND route.status == 200) → Anomaly (terminal).
(:wat::rete::defrule :usr::anomaly
  :when [(:usr::Suspect (?client <- :client) (?route <- :route) (?lat <- :lat))
         (:wat::rete::where (:wat::core::> ?lat 5000000))
         (:wat::rete::where (:wat::core::= (:usr::Route/status ?route) 200))]
  :then [(:usr::Anomaly :client ?client)])

(:wat::rete::defquery :usr::q-Suspect
  :params []
  :when [(?fact <- :usr::Suspect)])


(:wat::rete::defquery :usr::q-Anomaly
  :params []
  :when [(?fact <- :usr::Anomaly)])


(:wat::core::defn :usr::fire-one [template <- :wat::rete::Session seed <- :usr::Event] -> :wat::core::String
  (:wat::core::let
    [fired (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::insert template seed) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     s (:wat::core::length (:wat::rete::query fired (:usr::q-Suspect)))
     a (:wat::core::length (:wat::rete::query fired (:usr::q-Anomaly)))]
    (:wat::core::string::concat "Suspect=" (:wat::core::string::concat (:wat::core::str s)
      (:wat::core::string::concat " Anomaly=" (:wat::core::str a))))))

(:wat::core::defn :usr::mk [ctry <- :wat::core::String rep <- :wat::core::i64 lat <- :wat::core::i64 st <- :wat::core::i64] -> :usr::Event
  (:usr::Event
    :client (:usr::Client :ip "1.2.3.4" :geo (:usr::Geo :country ctry :asn 64500) :reputation rep)
    :route  (:usr::Route :method :usr::Method::POST :path "/api" :status st)
    :latency-ns lat))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::let [rules (:wat::core::PersistentVector (:usr::suspect) (:usr::anomaly))
                      template (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:usr::q-Suspect) (:usr::q-Anomaly))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))]
      (:wat::core::do
        (:wat::kernel::println (:wat::core::string::concat "XX/-5/9e6/200 (want S=1 A=1): " (:usr::fire-one template (:usr::mk "XX" -5 9000000 200))))
        (:wat::kernel::println (:wat::core::string::concat "US/-5/9e6/200 (want S=0 A=0): " (:usr::fire-one template (:usr::mk "US" -5 9000000 200))))
        (:wat::kernel::println (:wat::core::string::concat "XX/-5/2e6/200 (want S=1 A=0): " (:usr::fire-one template (:usr::mk "XX" -5 2000000 200))))
        (:wat::kernel::println (:wat::core::string::concat "XX/5/9e6/200  (want S=0 A=0): " (:usr::fire-one template (:usr::mk "XX" 5 9000000 200))))))))
