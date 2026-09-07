;; probe-what-publish-costs.wat — five primitives, one quiet box, then unit × count vs 37.3 s.
;;
;; Arc 278 perf. Last stone took publish 51.3 s → 37.3 s and named the leader, but
;; count-index is only 11 % of that wall. The rest is unattributed. This stone does
;; not fix; it aims. No production edit.
;;
;; Five units, one run:
;;   1. bare round trip, THREAD locus   — interpretation + dispatch floor
;;   2. bare round trip, PROCESS locus  — same, plus IPC (circuit queues are process)
;;   3. Store/put, one row, sqlite :memory:, queue already at depth ~60
;;   4. Store/count-index               — the cap gate
;;   5. Store/scan-index limit 1        — contrast
;;
;; ⚠ two loci cannot share a code path: process Handle is (Handle :- [Wire]), thread
;; Handle is (Handle :- [Shared]). Two functions, not one with a flag.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

;; ── bare ping: one nullary arm, a constant. No format, no state touch, no alloc
;; beyond the reply enum. If this is not the floor, STOP-1.
(:wat::core::defsurface :pp::Ping :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :pp::Ping::GoRequest [])
   (:wat::core::defenum :pp::Ping::GoResponse :wat::enum::Pure
     :Pong []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(go [self <- :pp::Ping  req <- :pp::Ping::GoRequest]
     -> :pp::Ping::GoResponse :max-request-bytes 524288)])

(:wat::service::defservice :pp::ping
  :satisfies :pp::Ping
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :pp::ping::Record] -> :pp::ping::State
          (:pp::ping::State :durable record))
  :impls
  [(go [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:pp::Ping::Reply::Go (:pp::Ping::GoResponse::Pong)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:pp::Ping::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:pp::ping::Op])])))])

(:wat::core::defn :pp::one-go
  [p <- (:wat::kernel::Peer :- [:pp::Ping::Op :pp::Ping::Reply])] -> :wat::core::nil
  (:wat::core::match (:pp::Ping/go p (:pp::Ping::GoRequest))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ (:wat::kernel::assertion-failed! "pp: ping failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :pp::loop-go
  [p <- (:wat::kernel::Peer :- [:pp::Ping::Op :pp::Ping::Reply])  n <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [_a <- :wat::core::nil  _i <- :wat::core::i64] -> :wat::core::nil
      (:pp::one-go p))
    nil
    (:wat::core::range 0 n)))

;; ns for n ops → µs/op. Same arithmetic as probe-what-a-scan-costs.
(:wat::core::defn :pp::us-per [n <- :wat::core::i64  ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::/ ns 1000) n))

(:wat::core::defn :pp::rt-thread [] -> :wat::core::i64
  (:wat::core::let
    [h (:pp::ping/start :locus (:wat::spawn::thread) :record (:pp::ping::Record :n 0))
     p (:wat::core::match (:wat::kernel::connect (:pp::ping::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "pp: thread dial failed" :wat::core::None :wat::core::None)))
     _w (:pp::loop-go p 50)
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:pp::loop-go p 1000)
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:pp::us-per 1000 (:wat::i64::- t1 t0))))

(:wat::core::defn :pp::rt-process [] -> :wat::core::i64
  (:wat::core::let
    [h (:pp::ping/start :locus (:wat::spawn::process) :record (:pp::ping::Record :n 0))
     p (:wat::core::match (:wat::kernel::connect (:pp::ping::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "pp: process dial failed" :wat::core::None :wat::core::None)))
     _w (:pp::loop-go p 50)
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:pp::loop-go p 1000)
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:pp::us-per 1000 (:wat::i64::- t1 t0))))

(:wat::core::defn :pp::pids [pl <- :wat::spawn::ProcessLaunch]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))

(:wat::core::defn :pp::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "pp: dial store failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :pp::dial-q
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])] -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "pp: dial q failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :pp::send-n [q <- :queue::Queue  n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [_a <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
      (:wat::core::match
        (:queue::Queue/send q
          (:queue::Queue::SendRequest :queue "q"
            :bodies (:wat::core::Vector :- [:wat::core::String] "m")
            :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
        (_ nil)))
    nil
    (:wat::core::range 0 n)))

(:wat::core::defn :pp::one-count
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   hi-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/count-index st
      (:wat::query::Store::CountIndexRequest
        :index "by-visible-at" :ipk "q"
        :isk-lo (:wat::edn::write (:wat::time::at-nanos 0))
        :isk-hi (:wat::edn::write (:wat::time::at-nanos hi-ns))
        :limit 100000))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::CountIndexResponse::Ok n) n)
        (_ -1)))
    (_ -2)))

(:wat::core::defn :pp::one-scan1
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   hi-ns <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/scan-index st
      (:wat::query::Store::ScanIndexRequest
        :index "by-visible-at" :ipk "q"
        :isk-lo (:wat::edn::write (:wat::time::at-nanos 0))
        :isk-hi (:wat::edn::write (:wat::time::at-nanos hi-ns))
        :limit 1 :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::ScanIndexResponse::Success rows _c) (:wat::core::count rows))
        (_ -1)))
    (_ -2)))

(:wat::core::defn :pp::one-put
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   i <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::let
    [isk (:wat::edn::write (:wat::time::at-nanos (:wat::i64::+ (:wat::time::epoch-nanos (:wat::time::now)) i)))
     row (:wat::query::StoredRow
           :pk "q" :sk (:wat::core::format "p{i}" :i i) :data "x"
           :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                         "by-visible-at" (:wat::query::IndexKey :ipk "q" :isk isk)))]
    (:wat::core::match
      (:wat::query::Store/put st
        (:wat::query::Store::PutRequest
          :rows (:wat::core::Vector :- [:wat::query::StoredRow] row)))
      ((:wat::kernel::RecvOutcome::Message _r) nil)
      (_ (:wat::kernel::assertion-failed! "pp: put failed" :wat::core::None :wat::core::None)))))

(:wat::core::defn :pp::run [] -> :wat::core::String
  (:wat::core::let
    [rt-t (:pp::rt-thread)
     rt-p (:pp::rt-process)
     sh (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
          :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names
                    (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh (:queue::queue/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:wat::query::sqlite-store/grant sh (:pp::pids pl))))
          :record (:queue::queue::Record :cap 64
                    :store-addr (:wat::query::sqlite-store::Handle/addr sh)
                    :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:pp::dial-q (:queue::queue::Handle/addr qh))
     st (:pp::dial-store (:wat::query::sqlite-store::Handle/addr sh))
     _f (:pp::send-n q 60)
     now (:wat::time::epoch-nanos (:wat::time::now))
     _cw (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
             (:wat::i64::+ acc (:pp::one-count st now)))
           0 (:wat::core::range 0 50))
     c0 (:wat::time::epoch-nanos (:wat::time::now))
     gotc (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
              (:wat::i64::+ acc (:pp::one-count st now)))
            0 (:wat::core::range 0 1000))
     c1 (:wat::time::epoch-nanos (:wat::time::now))
     count-us (:pp::us-per 1000 (:wat::i64::- c1 c0))
     _sw (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
             (:wat::i64::+ acc (:pp::one-scan1 st now)))
           0 (:wat::core::range 0 50))
     s0 (:wat::time::epoch-nanos (:wat::time::now))
     gots (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
              (:wat::i64::+ acc (:pp::one-scan1 st now)))
            0 (:wat::core::range 0 1000))
     s1 (:wat::time::epoch-nanos (:wat::time::now))
     scan-us (:pp::us-per 1000 (:wat::i64::- s1 s0))
     _pw (:wat::core::foldl
           (:wat::core::fn [_a <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
             (:pp::one-put st i))
           nil (:wat::core::range 0 50))
     p0 (:wat::time::epoch-nanos (:wat::time::now))
     _p (:wat::core::foldl
          (:wat::core::fn [_a <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
            (:pp::one-put st (:wat::i64::+ 50 i)))
          nil (:wat::core::range 0 1000))
     p1 (:wat::time::epoch-nanos (:wat::time::now))
     put-us (:pp::us-per 1000 (:wat::i64::- p1 p0))
     ;; DESIGN formula as written. put and count-index are full client calls
     ;; (already include a process RT); SCORE says whether 2×RT is the right count.
     pred-us (:wat::i64::* 8000 (:wat::i64::+ put-us (:wat::i64::+ count-us (:wat::i64::* 2 rt-p))))
     pred-ms (:wat::i64::/ pred-us 1000)
     actual 37300
     gap (:wat::i64::- actual pred-ms)
     honest-us (:wat::i64::* 8000 (:wat::i64::+ put-us count-us))
     honest-ms (:wat::i64::/ honest-us 1000)
     hgap (:wat::i64::- actual honest-ms)]
    (:wat::core::format
      "rt_thread_us={t} rt_process_us={p} put_us={u} count_us={c} scan1_us={s} count_n={cn} scan1_n={sn} ;; predicted_ms={pm} actual_ms={am} gap_ms={g} ;; honest_put+count_ms={hm} honest_gap_ms={hg}"
      :t rt-t :p rt-p :u put-us :c count-us :s scan-us
      :cn (:wat::i64::/ gotc 1000) :sn (:wat::i64::/ gots 1000)
      :pm pred-ms :am actual :g gap
      :hm honest-ms :hg hgap)))

(:wat::core::defn :user::compute [] -> :wat::core::String (:pp::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:pp::run)))
