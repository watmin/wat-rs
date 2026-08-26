;; tests/services/probe_arc278_sift_rules_arena.wat — arc 278 task #6 ARENA: the `sift-rules-defsvc`
;; macro's RICH-RECORD + SCALE + PAGED RED gate, proven end-to-end on BOTH loci. Co-located fixture
;; for the sibling .rs, slurped via startup_beside(file!()).
;;
;; Teaching goal: show rete-as-datalog reasoning over RICH, NESTED records (records composed of
;; records), not boxed ints — an HTTP/anomaly domain. Geo -> Client -> Event, an enum (Method), a
;; 2-level nested `where`-accessor (client.geo.country), a where-cascade (Event -> Suspect[Lemma] ->
;; {Anomaly,Breach}[Deductions], graded parallel at different thresholds off the SAME Lemma), a 2nd
;; independent cascade (Event -> Flagged[Lemma] -> Critical[Deduction]), and a direct single-level
;; branch (Event -> Overflow[Deduction], no gate). Modeled on tests/services/probe_arc278_sift_rules.wat
;; (the mem-store'/journal'/my-sift' grant-before-dial chain) and tests/services/probe_arc278_sift_arena.wat
;; (the flood-and-page-until-exhausted cursor idiom).
;;
;; ── the rich record domain (2 levels of nesting: Event.client.geo, Event.route.method is an enum) ──
;;   Geo    {country, asn}
;;   Client {ip, geo: Geo, reputation}
;;   Method  enum {GET, POST, PUT, DELETE}
;;   Route  {method: Method, path, status}
;;   Timing {dns-ns, total-ns}
;;   Event  {client: Client, route: Route, timing: Timing, bytes}
;;
;; ── the rule graph ──
;;   Event -where(total-ns>500k, reputation<0, 2-LEVEL client.geo.country=="XX")-> Suspect  (LEMMA)
;;   Suspect -where(total-ns>5,000,000, status==200)-> Anomaly   (DEDUCTION — high threshold)
;;   Suspect -where(total-ns>2,000,000, status==200)-> Breach    (DEDUCTION — low threshold; GRADED
;;                                                                 PARALLEL to Anomaly, same Lemma)
;;   Event   -where(bytes>10,000,000)->                Overflow (DEDUCTION — direct, single-level)
;;   Event -where(method==POST, reputation<-50)-> Flagged (LEMMA — 2nd independent cascade)
;;   Flagged -where(dns-ns>300,000)-> Critical (DEDUCTION — 2nd cascade's terminal)
;; derived    = {Suspect, Anomaly, Breach, Overflow, Flagged, Critical}
;; fired-upon = {Event, Suspect, Flagged}  (every :when condition's head type)
;; Deduction  = derived − fired-upon = {Anomaly, Breach, Overflow, Critical}   (returned, terminal)
;; Lemma      = derived ∩ fired-upon = {Suspect, Flagged}                     (internal gates, NEVER returned)
;;
;; ── the flood distribution (N=800, cycling 10 categories, 80 of each — validated in isolation at
;;    scratchpad/probe-arena-rich-graph.wat before wiring into the service) ──
;;   cat0 clean                      (US,  50, GET, 200,  100_000,  10_000,     1_000) -> 0
;;   cat1 hot but WRONG country      (US, -10, GET, 200, 6_000_000, 10_000,     1_000) -> 0  (Suspect gate fails on country)
;;   cat2 suspect, low total-ns      (XX, -10, GET, 200, 3_000_000, 10_000,     1_000) -> 1  (Breach only)
;;   cat3 suspect, high total-ns     (XX, -20, GET, 200, 6_000_000, 10_000,     1_000) -> 2  (Anomaly+Breach)
;;   cat4 suspect but bad status     (XX,  -5, GET, 404, 6_000_000, 10_000,     1_000) -> 0  (Suspect fires, Anomaly/Breach need status==200 — Lemma-only, must NOT leak)
;;   cat5 overflow only              (US,  50, GET, 200,   100_000, 10_000, 15_000_000) -> 1  (Overflow)
;;   cat6 flagged+critical           (US, -60, POST,200,   100_000,500_000,     1_000) -> 1  (Critical)
;;   cat7 flagged only               (US, -60, POST,200,   100_000,100_000,     1_000) -> 0  (Flagged fires, dns-ns too low — Lemma-only, must NOT leak)
;;   cat8 everything                 (XX, -60, POST,200, 9_000_000,900_000, 20_000_000) -> 4  (Anomaly+Breach+Overflow+Critical)
;;   cat9 clean, different variety   (CA,  10, PUT, 500,    50_000, 20_000,       500) -> 0
;; per-cycle(10) total = 0+0+1+2+0+1+1+0+4+0 = 9 deductions; 800/10 = 80 cycles -> EXACTLY 720
;; terminal Deductions. Paged at :limit 100 -> 800/100 = 8 exact pages.
;;
;; A second scenario proves the fail-closed guard still holds on the rich graph: a Log whose message
;; type is `:arena::Bogus` (deliberately NOT among the macro's `:defs`) makes the WHOLE page
;; `::Fatal`, never a silent skip.
;;
;; ── Phase 0 (the store ruin, grounded) ── a SINGLE `write-logs` call carrying this rich Event
;; payload crashes the journal' child on PROCESS locus ("peer closed / channel disconnected")
;; somewhere between 650 and 700 rows (bisected at scratchpad/probe-arena-scale-n*.wat) — a per-call
;; IPC-frame-size ceiling, NOT the ~1000-row total-store-duplication ruin (that one was reproduced
;; separately at N=3600, chunked-at-500/batch, giving 2781 instead of 2000 — a DIFFERENT failure
;; mode, still open, tracked, NOT hit here). Chunking the PROCESS-locus flood into 2 batches of 400
;; (well under the ~650 single-call ceiling) avoids the crash; the accumulated total of 800 rows in
;; the store reads back with EXACT counts on both loci (proven at scratchpad/probe-arena-scale-
;; chunked800.wat) — no duplication observed at this scale. THREAD locus needs no chunking (a single
;; 800-row write-logs call is fine — in-process channel, no IPC frame limit).

(:wat::core::defrecord :arena::Geo    [country <- :wat::core::String  asn <- :wat::core::i64])
(:wat::core::defrecord :arena::Client [ip <- :wat::core::String  geo <- :arena::Geo  reputation <- :wat::core::i64])
(:wat::core::defenum   :arena::Method :wat::enum::Pure :GET :POST :PUT :DELETE)
(:wat::core::defrecord :arena::Route  [method <- :arena::Method  path <- :wat::core::String  status <- :wat::core::i64])
(:wat::core::defrecord :arena::Timing [dns-ns <- :wat::core::i64  total-ns <- :wat::core::i64])
(:wat::core::defrecord :arena::Event  [client <- :arena::Client  route <- :arena::Route  timing <- :arena::Timing  bytes <- :wat::core::i64])
(:wat::core::defrecord :arena::Bogus  [x <- :wat::core::i64]) ;; deliberately NOT in :defs below

(:wat::query::sift-rules-defsvc
  :name :arena::my-sift
  :defs [(:wat::core::defrecord :arena::Geo    [country <- :wat::core::String  asn <- :wat::core::i64])
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
         (:wat::core::defrecord :arena::Critical [client <- :arena::Client])]
  :rules [;; RULE 1 (Event -> Lemma): 2-level nested-accessor `where` — client.geo.country.
          (:wat::rete::defrule :arena::suspect-rule
            :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing) (?bytes <- :bytes))
                   (:wat::rete::where (:wat::rete::core::i64::> (:arena::Timing/total-ns ?timing) 500000))
                   (:wat::rete::where (:wat::rete::core::i64::< (:arena::Client/reputation ?client) 0))
                   (:wat::rete::where (:wat::rete::string::= (:arena::Geo/country (:arena::Client/geo ?client)) "XX"))]
            :then [(:arena::Suspect :client ?client :route ?route :timing ?timing :bytes ?bytes)])
          ;; RULE 2a (Lemma -> Deduction, the cascade, HIGH threshold).
          (:wat::rete::defrule :arena::anomaly-rule
            :when [(:arena::Suspect (?client <- :client) (?route <- :route) (?timing <- :timing))
                   (:wat::rete::where (:wat::rete::core::i64::> (:arena::Timing/total-ns ?timing) 5000000))
                   (:wat::rete::where (:wat::rete::core::i64::= (:arena::Route/status ?route) 200))]
            :then [(:arena::Anomaly :client ?client)])
          ;; RULE 2b (Lemma -> Deduction, GRADED PARALLEL to 2a — same Lemma, LOW threshold).
          (:wat::rete::defrule :arena::breach-rule
            :when [(:arena::Suspect (?client <- :client) (?route <- :route) (?timing <- :timing))
                   (:wat::rete::where (:wat::rete::core::i64::> (:arena::Timing/total-ns ?timing) 2000000))
                   (:wat::rete::where (:wat::rete::core::i64::= (:arena::Route/status ?route) 200))]
            :then [(:arena::Breach :client ?client)])
          ;; RULE 3 (Event -> Deduction, DIRECT single-level, no gate).
          (:wat::rete::defrule :arena::overflow-rule
            :when [(:arena::Event (?bytes <- :bytes))
                   (:wat::rete::where (:wat::rete::core::i64::> ?bytes 10000000))]
            :then [(:arena::Overflow :bytes ?bytes)])
          ;; RULE 4 (Event -> Lemma, a 2nd independent gate — enum equality on route.method).
          (:wat::rete::defrule :arena::flagged-rule
            :when [(:arena::Event (?client <- :client) (?route <- :route) (?timing <- :timing))
                   (:wat::rete::where (:wat::rete::core::enum::= (:arena::Route/method ?route) :arena::Method::POST))
                   (:wat::rete::where (:wat::rete::core::i64::< (:arena::Client/reputation ?client) -50))]
            :then [(:arena::Flagged :client ?client :route ?route :timing ?timing)])
          ;; RULE 5 (Lemma -> Deduction, the 2nd cascade's terminal).
          (:wat::rete::defrule :arena::critical-rule
            :when [(:arena::Flagged (?client <- :client) (?timing <- :timing))
                   (:wat::rete::where (:wat::rete::core::i64::> (:arena::Timing/dns-ns ?timing) 300000))]
            :then [(:arena::Critical :client ?client)])])

;; page-loop accumulator — orchestrator-side (not inside a forked service, so a plain top-level
;; record is fine — mirrors :arena::PageAcc's precedent).
(:wat::core::defrecord :arena::PageAcc
  [done  <- :wat::core::bool
   cur   <- (:wat::core::Option :wat::core::String)
   acc   <- :wat::core::i64
   clean <- :wat::core::bool])

;; event-for-i / log-for-i — shared orchestrator-side helpers (also not inside a forked service,
;; same cross-fork reasoning as probe_arc278_sift_arena.wat's note). Maps a flat index to one of 10
;; cycling categories (see the distribution table in the file banner) and builds its Log.
(:wat::core::defn :arena::event-for-i [i <- :wat::core::i64] -> :arena::Event
  (:wat::core::let
    [cat (:wat::core::mod i 10)
     mk  (:wat::core::fn
           [ctry <- :wat::core::String rep <- :wat::core::i64 method <- :arena::Method status <- :wat::core::i64
            total-ns <- :wat::core::i64 dns-ns <- :wat::core::i64 bytes <- :wat::core::i64]
           -> :arena::Event
           (:arena::Event
             :client (:arena::Client :ip "1.2.3.4" :geo (:arena::Geo :country ctry :asn 64500) :reputation rep)
             :route  (:arena::Route :method method :path "/api" :status status)
             :timing (:arena::Timing :dns-ns dns-ns :total-ns total-ns)
             :bytes  bytes))]
    (:wat::core::if (:wat::core::= cat 0)
      (mk "US" 50 :arena::Method::GET 200 100000 10000 1000)
      (:wat::core::if (:wat::core::= cat 1)
        (mk "US" -10 :arena::Method::GET 200 6000000 10000 1000)
        (:wat::core::if (:wat::core::= cat 2)
          (mk "XX" -10 :arena::Method::GET 200 3000000 10000 1000)
          (:wat::core::if (:wat::core::= cat 3)
            (mk "XX" -20 :arena::Method::GET 200 6000000 10000 1000)
            (:wat::core::if (:wat::core::= cat 4)
              (mk "XX" -5 :arena::Method::GET 404 6000000 10000 1000)
              (:wat::core::if (:wat::core::= cat 5)
                (mk "US" 50 :arena::Method::GET 200 100000 10000 15000000)
                (:wat::core::if (:wat::core::= cat 6)
                  (mk "US" -60 :arena::Method::POST 200 100000 500000 1000)
                  (:wat::core::if (:wat::core::= cat 7)
                    (mk "US" -60 :arena::Method::POST 200 100000 100000 1000)
                    (:wat::core::if (:wat::core::= cat 8)
                      (mk "XX" -60 :arena::Method::POST 200 9000000 900000 20000000)
                      (mk "CA" 10 :arena::Method::PUT 500 50000 20000 500))))))))))))

(:wat::core::defn :arena::log-for-i
  [i <- :wat::core::i64  tags <- (:wat::core::HashMap :wat::core::keyword :wat::core::String)]
  -> :wat::telemetry::Log
  (:wat::telemetry::Log :namespace "arena-rules-ns" :uuid (:wat::uuid::nil) :tags tags
    :time-ns (:wat::i64::+ i 1) :emitted-from (:wat::kernel::call-site)
    :level :wat::telemetry::Level::Info :message (:wat::edn::write (:arena::event-for-i i))))

;; ── THREAD locus — flood 800 rich Events (10-way cycle) in ONE write-logs call (fine on thread —
;; in-process channel, no IPC frame limit), page sift-rules at :limit 100, assert exactly 720. ──
;;
;; NOTE — a real finding: the page-loop below is INLINED directly in each of the two driver fns
;; below (duplicated), never factored into a shared top-level `:arena::page-loop [svc <- (Peer' :- […])]`
;; helper — that factoring was tried first and broke BOTH loci with "channel disconnected" on the
;; very first `sift-rules` send, even on THREAD (no fork involved, so this is NOT the cross-fork
;; :messages-reachability gap probe_arc278_sift_arena.wat's own note describes — that one raises
;; UnresolvedReference at COMPILE time; this is a runtime IPC failure). Root cause not chased
;; (out of this task's scope) — bisected at scratchpad/probe-arena-bisect{1,2,3}.wat: passing a
;; connected `(Peer' :- [...])` client handle as a PARAMETER into a separate top-level `defn` and issuing
;; the RPC from inside that callee reproduces the crash; keeping the same call inline in the
;; `let` that created the connection does not. Mirrors the codebase's own established idiom
;; (probe_arc278_sift_arena.wat's `:cons::consumer'/sift` cursor-loop is inline for a related-but-
;; distinct reason) — inlining here is simply the proven-safe shape, not a workaround for THIS bug
;; specifically discovered mid-task.
(:wat::core::defn :user::sift-rules-arena-thread [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     idxs  (:wat::core::range 0 800)
     logs  (:wat::core::into (:wat::core::Vector :wat::telemetry::Log)
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log (:arena::log-for-i i tags))
               idxs))
     _wr   (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs))
     sh    (:arena::my-sift'/start :locus (:wat::spawn::thread)
             :record (:arena::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::core::match (:wat::kernel::connect (:arena::my-sift'::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     page-idxs (:wat::core::range 0 12)
     initial (:arena::PageAcc :done false :cur :wat::core::None :acc 0 :clean true)
     final (:wat::core::foldl
             (:wat::core::fn [state <- :arena::PageAcc _i <- :wat::core::i64] -> :arena::PageAcc
               (:wat::core::if (:arena::PageAcc/done state)
                 state
                 (:wat::core::let
                   [resp (:arena::my-sift/sift-rules svc
                           (:arena::my-sift::SiftRulesRequest :namespace "arena-rules-ns"
                             :time-lo 0 :time-hi 100000000 :limit 100
                             :cursor (:arena::PageAcc/cur state)))]
                   (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
                     ((:arena::my-sift::SiftRulesResponse::Deductions items cur)
                       (:wat::core::let
                         [page-clean (:wat::core::foldl
                                       (:wat::core::fn [ok <- :wat::core::bool v <- :wat::core::Value] -> :wat::core::bool
                                         (:wat::core::if ok
                                           (:wat::core::not
                                             (:wat::core::or
                                               (:wat::core::= (:wat::core::type v) "arena::Suspect")
                                               (:wat::core::= (:wat::core::type v) "arena::Flagged")))
                                           false))
                                       true
                                       items)
                          new-acc (:wat::core::+ (:arena::PageAcc/acc state) (:wat::core::length items))
                          new-clean (:wat::core::and (:arena::PageAcc/clean state) page-clean)]
                         (:wat::core::match cur 
                           (:wat::core::None (:arena::PageAcc :done true :cur :wat::core::None :acc new-acc :clean new-clean))
                           ((:wat::core::Some c) (:arena::PageAcc :done false :cur (:wat::core::Some c) :acc new-acc :clean new-clean)))))
                     ((:arena::my-sift::SiftRulesResponse::Fatal _err)
                       (:arena::PageAcc :done true :cur :wat::core::None :acc -999999 :clean false))
                     ((:arena::my-sift::SiftRulesResponse::RequestTooLarge _bytes _cap)
                       (:wat::kernel::assertion-failed! "sift-rules-arena: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
                     ((:arena::my-sift::SiftRulesResponse::RequestMalformed mpath mexpected mgot)
                       (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))))
             initial
             page-idxs)]
    (:wat::core::if (:arena::PageAcc/clean final) (:arena::PageAcc/acc final) -1)))

;; ── PROCESS locus — the loci-agnostic proof. SAME scenario across a FORK: mem-store' + journal' +
;; arena::my-sift' all on process, grant-before-dial at every hop. The flood is CHUNKED into 2
;; batches of 400 (Phase-0 grounded: a single write-logs call of this rich payload crashes the
;; journal' child somewhere between 650-700 rows on PROCESS locus; 400/batch is comfortably under
;; that ceiling). The page loop itself runs in the MAIN process, calling `sift-rules` across the
;; wire each iteration — no cross-fork concern for the loop body. ──
(:wat::core::defn :user::sift-rules-arena-process [] -> :wat::core::i64
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::process)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::query::mem-store/grant msh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     idxs1 (:wat::core::range 0 400)
     idxs2 (:wat::core::range 400 800)
     logs1 (:wat::core::into (:wat::core::Vector :wat::telemetry::Log)
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log (:arena::log-for-i i tags))
               idxs1))
     logs2 (:wat::core::into (:wat::core::Vector :wat::telemetry::Log)
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log (:arena::log-for-i i tags))
               idxs2))
     _wr1  (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs1))
     _wr2  (:wat::telemetry::Journal/write-logs journal (:wat::telemetry::Journal::WriteLogsRequest logs2))
     sh    (:arena::my-sift'/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::telemetry::journal/grant jh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:arena::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::core::match (:wat::kernel::connect (:arena::my-sift'::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     page-idxs (:wat::core::range 0 12)
     initial (:arena::PageAcc :done false :cur :wat::core::None :acc 0 :clean true)
     final (:wat::core::foldl
             (:wat::core::fn [state <- :arena::PageAcc _i <- :wat::core::i64] -> :arena::PageAcc
               (:wat::core::if (:arena::PageAcc/done state)
                 state
                 (:wat::core::let
                   [resp (:arena::my-sift/sift-rules svc
                           (:arena::my-sift::SiftRulesRequest :namespace "arena-rules-ns"
                             :time-lo 0 :time-hi 100000000 :limit 100
                             :cursor (:arena::PageAcc/cur state)))]
                   (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
                     ((:arena::my-sift::SiftRulesResponse::Deductions items cur)
                       (:wat::core::let
                         [page-clean (:wat::core::foldl
                                       (:wat::core::fn [ok <- :wat::core::bool v <- :wat::core::Value] -> :wat::core::bool
                                         (:wat::core::if ok
                                           (:wat::core::not
                                             (:wat::core::or
                                               (:wat::core::= (:wat::core::type v) "arena::Suspect")
                                               (:wat::core::= (:wat::core::type v) "arena::Flagged")))
                                           false))
                                       true
                                       items)
                          new-acc (:wat::core::+ (:arena::PageAcc/acc state) (:wat::core::length items))
                          new-clean (:wat::core::and (:arena::PageAcc/clean state) page-clean)]
                         (:wat::core::match cur 
                           (:wat::core::None (:arena::PageAcc :done true :cur :wat::core::None :acc new-acc :clean new-clean))
                           ((:wat::core::Some c) (:arena::PageAcc :done false :cur (:wat::core::Some c) :acc new-acc :clean new-clean)))))
                     ((:arena::my-sift::SiftRulesResponse::Fatal _err)
                       (:arena::PageAcc :done true :cur :wat::core::None :acc -999999 :clean false))
                     ((:arena::my-sift::SiftRulesResponse::RequestTooLarge _bytes _cap)
                       (:wat::kernel::assertion-failed! "sift-rules-arena: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
                     ((:arena::my-sift::SiftRulesResponse::RequestMalformed mpath mexpected mgot)
                       (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))))
             initial
             page-idxs)]
    (:wat::core::if (:arena::PageAcc/clean final) (:arena::PageAcc/acc final) -1)))

;; ── THREAD locus — fail-closed: one Log's message is `:arena::Bogus`, NOT among :defs. The WHOLE
;; page must come back ::Fatal (never a silent skip / partial result), on the rich graph too. ──
(:wat::core::defn :user::sift-rules-arena-fatal-thread [] -> :wat::core::bool
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     l1    (:wat::telemetry::Log :namespace "arena-rules-fatal-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1 :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:arena::event-for-i 2)))
     l2    (:wat::telemetry::Log :namespace "arena-rules-fatal-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2 :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:arena::Bogus :x 1)))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :wat::telemetry::Log l1 l2)))
     sh    (:arena::my-sift'/start :locus (:wat::spawn::thread)
             :record (:arena::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::core::match (:wat::kernel::connect (:arena::my-sift'::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     resp  (:arena::my-sift/sift-rules svc
             (:arena::my-sift::SiftRulesRequest :namespace "arena-rules-fatal-ns" :time-lo 0 :time-hi 100000000
               :limit 50 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:arena::my-sift::SiftRulesResponse::Fatal _err) true)
      (_ false))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; ── PROCESS locus — same fail-closed guard, across a FORK. ──
(:wat::core::defn :user::sift-rules-arena-fatal-process [] -> :wat::core::bool
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::process)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::query::mem-store/grant msh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     l1    (:wat::telemetry::Log :namespace "arena-rules-fatal-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 1 :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:arena::event-for-i 2)))
     l2    (:wat::telemetry::Log :namespace "arena-rules-fatal-ns" :uuid (:wat::uuid::nil) :tags tags
             :time-ns 2 :emitted-from (:wat::kernel::call-site) :level :wat::telemetry::Level::Info
             :message (:wat::edn::write (:arena::Bogus :x 1)))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest (:wat::core::Vector :wat::telemetry::Log l1 l2)))
     sh    (:arena::my-sift'/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::telemetry::journal/grant jh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:arena::my-sift'::Record) :journal-addr jaddr)
     svc   (:wat::core::match (:wat::kernel::connect (:arena::my-sift'::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     resp  (:arena::my-sift/sift-rules svc
             (:arena::my-sift::SiftRulesRequest :namespace "arena-rules-fatal-ns" :time-lo 0 :time-hi 100000000
               :limit 50 :cursor :wat::core::None))]
    (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:arena::my-sift::SiftRulesResponse::Fatal _err) true)
      (_ false))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
