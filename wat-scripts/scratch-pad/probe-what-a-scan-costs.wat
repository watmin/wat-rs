;; probe-what-a-scan-costs.wat — what does ONE cap-gate scan cost, on sqlite, at depth?
;;
;; Arc 278 perf. publish went 26.6s -> 50.1s across four stones. The stage histograms
;; localize it exactly: `outbox` (t0 publish -> t1 topic-worker pickup) moved from
;; 50-250ms=7944 / max 260ms to 250-1000ms=7739 / max 5693ms, while t1->t2 and t2->t3
;; stayed <1ms for all 8000. So it is QUEUEING DELAY in the topic's inbox -- the topic
;; worker drains slower, so rows wait longer.
;;
;; The prime suspect is the cap gate. `depth is read, not counted` replaced two i64
;; field reads with TWO scan-index round trips on EVERY send (sqs.wat:293), and the
;; topic worker sends once per message. My own grading of that stone found three of the
;; four depth call sites immediately SUM the pair -- so the send path pays for a
;; visible/unacked split it discards, and one scan would do.
;;
;; ⛔ THE DISCONFIRMING QUESTION: is a scan actually expensive enough to explain +18s?
;; 8000 sends x 2 scans = 16000 scans. If a scan is ~1ms, that is ~16s -- the whole
;; regression. If it is ~50us, the cap gate is NOT the story and I am chasing the wrong
;; thing, which is exactly what this file exists to find out before a stone is drawn.
;;
;; Measured against the SAME store the circuit uses (sqlite), at the SAME depth the cap
;; gate sees (cap+1 = 65 limit, queue held near cap).

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defn :sc::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "sc: dial store failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :sc::dial-q
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])] -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "sc: dial q failed" :wat::core::None :wat::core::None))))

;; one scan of the by-visible-at index, exactly as the cap gate issues it
(:wat::core::defn :sc::one-scan
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   hi-ns <- :wat::core::i64  lim <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/scan-index st
      (:wat::query::Store::ScanIndexRequest
        :index "by-visible-at" :ipk "q"
        :isk-lo (:wat::edn::write (:wat::time::at-nanos 0))
        :isk-hi (:wat::edn::write (:wat::time::at-nanos hi-ns))
        :limit lim :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::ScanIndexResponse::Success rows _c) (:wat::core::count rows))
        (_ -1)))
    (_ -2)))

(:wat::core::defn :sc::one-count
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   hi-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/count-index st
      (:wat::query::Store::CountIndexRequest
        :index "by-visible-at" :ipk "q"
        :isk-lo (:wat::edn::write (:wat::time::at-nanos 0))
        :isk-hi (:wat::edn::write (:wat::time::at-nanos hi-ns))))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::CountIndexResponse::Ok n) n)
        (_ -1)))
    (_ -2)))

(:wat::core::defn :sc::loop-scans
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   n <- :wat::core::i64  hi-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
      (:wat::i64::+ acc (:sc::one-scan st hi-ns 65)))
    0
    (:wat::core::range 0 n)))

(:wat::core::defn :sc::send-n [q <- :queue::Queue  n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [_a <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
      (:wat::core::match
        (:queue::Queue/send q
          (:queue::Queue::SendRequest :queue "q"
            :bodies (:wat::core::Vector :- [:wat::core::String] (:wat::core::format "m{i}" :i i))
            :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
        (_ nil)))
    nil
    (:wat::core::range 0 n)))

(:wat::core::defn :sc::run [] -> :wat::core::String
  (:wat::core::let
    [sh (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names
                    (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record :cap 64
                    :store-addr (:wat::query::sqlite-store::Handle/addr sh)
                    :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:sc::dial-q (:queue::queue::Handle/addr qh))
     st (:sc::dial-store (:wat::query::sqlite-store::Handle/addr sh))
     _f (:sc::send-n q 60)
     now (:wat::time::epoch-nanos (:wat::time::now))
     _warm (:sc::loop-scans st 50 now)
     a0 (:wat::time::epoch-nanos (:wat::time::now))
     got (:sc::loop-scans st 1000 now)
     a1 (:wat::time::epoch-nanos (:wat::time::now))
     full-us (:wat::i64::/ (:wat::i64::- a1 a0) 1000)
     ;; Same query, limit 1: separates QUERY OVERHEAD from ROW MATERIALIZATION.
     b0 (:wat::time::epoch-nanos (:wat::time::now))
     got1 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
              (:wat::i64::+ acc (:sc::one-scan st now 1)))
            0 (:wat::core::range 0 1000))
     b1 (:wat::time::epoch-nanos (:wat::time::now))
     lim1-us (:wat::i64::/ (:wat::i64::- b1 b0) 1000)
     _cwarm (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
                (:wat::i64::+ acc (:sc::one-count st now)))
              0 (:wat::core::range 0 50))
     c0 (:wat::time::epoch-nanos (:wat::time::now))
     gotc (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
              (:wat::i64::+ acc (:sc::one-count st now)))
            0 (:wat::core::range 0 1000))
     c1 (:wat::time::epoch-nanos (:wat::time::now))
     count-us (:wat::i64::/ (:wat::i64::- c1 c0) 1000)]
    (:wat::core::format
      "LIMIT65 rows={r} us_per_scan={u} proj_s_16000={p} ;; LIMIT1 rows={r1} us_per_scan={u1} proj_s_16000={p1} ;; COUNT n={cn} us_per={cu} proj_s_8000={cp}"
      :r (:wat::i64::/ got 1000)
      :u (:wat::i64::/ full-us 1000)
      :p (:wat::i64::/ (:wat::i64::* (:wat::i64::/ full-us 1000) 16) 1000)
      :r1 (:wat::i64::/ got1 1000)
      :u1 (:wat::i64::/ lim1-us 1000)
      :p1 (:wat::i64::/ (:wat::i64::* (:wat::i64::/ lim1-us 1000) 16) 1000)
      :cn (:wat::i64::/ gotc 1000)
      :cu (:wat::i64::/ count-us 1000)
      :cp (:wat::i64::/ (:wat::i64::* (:wat::i64::/ count-us 1000) 8) 1000))))

(:wat::core::defn :user::compute [] -> :wat::core::String (:sc::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:sc::run)))
