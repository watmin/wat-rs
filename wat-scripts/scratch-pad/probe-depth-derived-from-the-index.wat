;; probe-depth-derived-from-the-index.wat — can the queue's depth be READ instead of COUNTED?
;;
;; sqs.wat keeps `visible` and `unacked` as two hand-maintained i64 counters in the queue's
;; :ephemeral State. They cannot be correct: `take` scans `by-visible-at` for `isk <= now`, so
;; a never-received row and an EXPIRED LEASE are the same thing by construction (correct SQS
;; semantics). Visibility expiry is therefore not an EVENT -- it happens because the clock
;; moved, and no code runs. A counter can only be updated by code that runs.
;;
;; The store already holds the truth in one field: a row's `isk` IS its visible-at instant.
;;   visible  = rows with isk <= now
;;   unacked  = all rows - visible
;;
;; THE DISCONFIRMING QUESTION: can a caller scan that index and count, at two ranges, and do
;; the two numbers track reality across a visibility expiry -- WHERE THE COUNTERS DO NOT?
;;
;; Cells, on ONE queue, 3 messages, 1 received with a 200 ms lease:
;;   t1 (lease live)     derived should be [2/1]  and stats should AGREE
;;   t2 (lease expired)  derived should be [3/0]  and stats should DISAGREE  <- both halves
;;
;; A probe that only showed the derived numbers would prove the mechanism and not the defect.
;; A probe that only showed stats would prove the defect and not the fix. This shows both in
;; one run, from one state, so neither half rests on a separate setup.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defn :dd::await-timer-ms [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds ms) :done))
    ((:wat::kernel::RecvOutcome::Message _m) nil)
    (_ nil)))

(:wat::core::defn :dd::dial-q
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])] -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "dd: dial queue failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :dd::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "dd: dial store failed" :wat::core::None :wat::core::None))))

;; THE CANDIDATE MECHANISM: count rows of "q" whose visible-at falls in [lo-ns, hi-ns].
;; limit 1000 is far above any cap this probe uses; a real implementation passes cap+1 so an
;; overflow is VISIBLE rather than silently truncated.
(:wat::core::defn :dd::count-in-range
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   lo-ns <- :wat::core::i64  hi-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/scan-index st
      (:wat::query::Store::ScanIndexRequest
        :index "by-visible-at" :ipk "q"
        :isk-lo (:wat::edn::write (:wat::time::at-nanos lo-ns))
        :isk-hi (:wat::edn::write (:wat::time::at-nanos hi-ns))
        :limit 1000 :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::ScanIndexResponse::Success rows _c) (:wat::core::count rows))
        (_ -1)))
    (_ -2)))

(:wat::core::defn :dd::stats-pair [q <- :queue::Queue] -> :wat::core::String
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks visible unacked)
          (:wat::core::format "[{v}/{u}]" :v visible :u unacked))
        (_ "[not-ok]")))
    (_ "[lost]")))

(:wat::core::defn :dd::derived-pair
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::String
  (:wat::core::let
    [now (:wat::time::epoch-nanos (:wat::time::now))
     all (:dd::count-in-range st 0 4000000000000000000)
     vis (:dd::count-in-range st 0 now)]
    (:wat::core::format "[{v}/{u}]" :v vis :u (:wat::i64::- all vis))))

(:wat::core::defn :dd::send-n [q <- :queue::Queue  bodies <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest :queue "q" :bodies bodies
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ (:wat::kernel::assertion-failed! "dd: send failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :dd::take-one [q <- :queue::Queue  vis-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest :queue "q"
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns vis-ns :limit 1 :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) (:wat::core::count envs))
        (_ -1)))
    (_ -2)))

(:wat::core::defn :dd::run [] -> :wat::core::String
  (:wat::core::let
    [sh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record :cap 64 :store-addr (:wat::query::mem-store::Handle/addr sh) :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:dd::dial-q (:queue::queue::Handle/addr qh))
     st (:dd::dial-store (:wat::query::mem-store::Handle/addr sh))
     _s (:dd::send-n q (:wat::core::Vector :- [:wat::core::String] "m0" "m1" "m2"))
     took (:dd::take-one q 200000000)
     d1 (:dd::derived-pair st)
     c1 (:dd::stats-pair q)
     _w (:dd::await-timer-ms 350)
     d2 (:dd::derived-pair st)
     c2 (:dd::stats-pair q)]
    (:wat::core::format
      "took={t};LEASE-LIVE derived={a} counters={b} agree={ab};LEASE-EXPIRED derived={c} counters={d} agree={cd}"
      :t took :a d1 :b c1 :ab (:wat::core::if (:wat::core::= d1 c1) "yes" "NO")
      :c d2 :d c2 :cd (:wat::core::if (:wat::core::= d2 c2) "yes" "NO"))))

(:wat::core::defn :user::compute [] -> :wat::core::String (:dd::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:dd::run)))
