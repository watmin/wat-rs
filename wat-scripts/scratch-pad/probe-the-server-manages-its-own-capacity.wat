;; probe-the-server-manages-its-own-capacity.wat
;;
;; Admission: prefix only when n0 > cap; otherwise all-or-nothing.
;;   cap 10, depth 6, send 8 (8 <= cap) → Accepted 0, depth unchanged
;;   drain 5 (depth 1, room 9), send 8 → Accepted 8
;;   cap 10, send 15 (15 > cap) → Accepted 10, first 10 bodies in order
;; nsubs 7, publish 10 against cap 64 (70 > 64) → a count, no assertion.

(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")

(:wat::core::defn :cap::bodies [prefix <- :wat::core::String  n <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::format "{p}{i}" :p prefix :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :cap::dial-q
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])] -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "cap: dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :cap::depth [q <- :queue::Queue] -> :wat::core::i64
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks visible unacked _ _)
          (:wat::i64::+ visible unacked))
        (_ (:wat::kernel::assertion-failed! "cap: stats not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "cap: stats recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :cap::send-n
  [q <- :queue::Queue  prefix <- :wat::core::String  n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest :queue "q" :bodies (:cap::bodies prefix n)
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Accepted c) c)
        (_ (:wat::kernel::assertion-failed! "cap: send not Accepted" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "cap: send recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :cap::recv-all
  [q <- :queue::Queue] -> (:wat::core::Vector :- [:queue::Envelope])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue "q"
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns 1000000000000
        :limit 64
        :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "cap: receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "cap: receive recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :cap::join-bodies
  [envs <- (:wat::core::Vector :- [:queue::Envelope])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  e <- :queue::Envelope] -> :wat::core::String
      (:wat::core::if (:wat::core::= acc "")
        (:queue::Envelope/body e)
        (:wat::core::format "{a},{b}" :a acc :b (:queue::Envelope/body e))))
    ""
    envs))

(:wat::core::defn :cap::ack-n
  [q <- :queue::Queue  envs <- (:wat::core::Vector :- [:queue::Envelope])  n <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::let
    [ids (:wat::core::foldl
           (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
             -> (:wat::core::Vector :- [:wat::core::String])
             (:wat::core::conj acc (:queue::Envelope/id (:wat::core::nth envs i))))
           (:wat::core::Vector :- [:wat::core::String])
           (:wat::core::range 0 n))]
    (:wat::core::match
      (:queue::Queue/ack q (:queue::Queue::AckRequest :queue "q" :ids ids))
      ((:wat::kernel::RecvOutcome::Message _r) nil)
      (_ (:wat::kernel::assertion-failed! "cap: ack failed" :wat::core::None :wat::core::None)))))

(:wat::core::defn :cap::queue-phase [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 10
                     :store-addr (:wat::query::mem-store::Handle/addr ish)
                     :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:cap::dial-q (:queue::queue::Handle/addr qh))
     f   (:cap::send-n q "f" 6)
     d6  (:cap::depth q)
     a8  (:cap::send-n q "m" 8)
     d8  (:cap::depth q)
     envs (:cap::recv-all q)
     _   (:cap::ack-n q envs 5)
     a5  (:cap::send-n q "y" 8)
     da  (:cap::depth q)
     _keep qh
     _keep2 ish]
    (:wat::core::format
      "fill={f};depth6={d6};send8={a8};depth-after-8={d8};drain5-send8={a5};depth-after={da}"
      :f f :d6 d6 :a8 a8 :d8 d8 :a5 a5 :da da)))

(:wat::core::defn :cap::above-cap [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 10
                     :store-addr (:wat::query::mem-store::Handle/addr ish)
                     :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:cap::dial-q (:queue::queue::Handle/addr qh))
     n   (:cap::send-n q "z" 15)
     envs (:cap::recv-all q)
     stored (:cap::join-bodies envs)
     _keep qh
     _keep2 ish]
    (:wat::core::format "accepted={n};n={c};stored={st}"
      :n n :c (:wat::core::count envs) :st stored)))

(:wat::core::defn :cap::msgs [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:cap::bodies "p" n))

(:wat::core::defn :cap::publish-n
  [t <- :demo::Topic  n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:demo::Topic/publish t (:demo::Topic::PublishRequest :msgs (:cap::msgs n)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::PublishResponse::Accepted c)
          (:wat::core::format "Accepted({c})" :c c))
        ((:demo::Topic::PublishResponse::RequestTooManyEntries e c)
          (:wat::core::format "RequestTooManyEntries({e},{c})" :e e :c c))
        (_ "other")))
    (_ "recv-failed")))

(:wat::core::defn :cap::topic-at
  [nsubs <- :wat::core::i64  cap <- :wat::core::i64  n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap cap
                     :store-addr (:wat::query::mem-store::Handle/addr ish)
                     :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs nsubs :inbox-addr (:queue::queue::Handle/addr iqh)))
     t  (:demo::dial-topic (:demo::topic::Handle/addr th))
     tag (:cap::publish-n t n)
     _keep th
     _keep2 iqh
     _keep3 ish]
    tag))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [q (:cap::queue-phase)
     ac (:cap::above-cap)
     t1 (:cap::topic-at 4 6 10)
     t7 (:cap::topic-at 7 64 10)]
    (:wat::core::format "queue={q};above-cap={ac};nsubs4-room6={t1};nsubs7-pub10={t7}"
      :q q :ac ac :t1 t1 :t7 t7)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
