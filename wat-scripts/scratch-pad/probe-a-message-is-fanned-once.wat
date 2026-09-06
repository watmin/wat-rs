;; probe-a-message-is-fanned-once.wat
;;
;; nsubs 4, inbox cap 6, publish 2: first send takes 6 pairs (msg-major),
;; top-up of the 2 missing pairs is refused (cap full). Accepted 1 (floor).
;; Message 0 is fully fanned: exactly 4 pairs, bodies "{i}|p0".
;; nsubs 7, publish 10 against cap 64: a count, no assertion.

(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")

(:wat::core::defn :once::msgs [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::format "p{i}" :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :once::join
  [envs <- (:wat::core::Vector :- [:queue::Envelope])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  e <- :queue::Envelope] -> :wat::core::String
      (:wat::core::if (:wat::core::= acc "")
        (:queue::Envelope/body e)
        (:wat::core::format "{a},{b}" :a acc :b (:queue::Envelope/body e))))
    ""
    envs))

(:wat::core::defn :once::count-msg0
  [envs <- (:wat::core::Vector :- [:queue::Envelope])] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [n <- :wat::core::i64  e <- :queue::Envelope] -> :wat::core::i64
      (:wat::core::let [parts (:wat::string::split (:queue::Envelope/body e) "|")]
        (:wat::core::if (:wat::i64::< (:wat::core::count parts) 2)
          n
          (:wat::core::if (:wat::core::= (:wat::core::nth parts 1) "p0")
            (:wat::i64::+ n 1)
            n))))
    0
    envs))

(:wat::core::defn :once::recv-all [q <- :queue::Queue] -> (:wat::core::Vector :- [:queue::Envelope])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue "inbox"
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns 1000000000000
        :limit 64
        :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::kernel::assertion-failed! "once: receive not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "once: receive recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :once::split-case [] -> :wat::core::String
  (:wat::core::let
    [nsubs 4
     ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 6
                     :store-addr (:wat::query::mem-store::Handle/addr ish)
                     :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs nsubs :inbox-addr (:queue::queue::Handle/addr iqh)))
     t  (:demo::dial-topic (:demo::topic::Handle/addr th))
     q  (:demo::dial-queue (:queue::queue::Handle/addr iqh))
     tag (:wat::core::match
           (:demo::Topic/publish t (:demo::Topic::PublishRequest :msgs (:once::msgs 2)))
           ((:wat::kernel::RecvOutcome::Message r)
             (:wat::core::match r
               ((:demo::Topic::PublishResponse::Accepted c)
                 (:wat::core::format "Accepted({c})" :c c))
               (_ "other")))
           (_ "recv-failed"))
     envs (:once::recv-all q)
     n0 (:once::count-msg0 envs)
     bodies (:once::join envs)
     _keep th
     _keep2 iqh
     _keep3 ish]
    (:wat::core::format "split={tag};msg0={n0};n={n};bodies={b}"
      :tag tag :n0 n0 :n (:wat::core::count envs) :b bodies)))

(:wat::core::defn :once::cliff [] -> :wat::core::String
  (:wat::core::let
    [ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64
                     :store-addr (:wat::query::mem-store::Handle/addr ish)
                     :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs 7 :inbox-addr (:queue::queue::Handle/addr iqh)))
     t  (:demo::dial-topic (:demo::topic::Handle/addr th))
     tag (:wat::core::match
           (:demo::Topic/publish t (:demo::Topic::PublishRequest :msgs (:once::msgs 10)))
           ((:wat::kernel::RecvOutcome::Message r)
             (:wat::core::match r
               ((:demo::Topic::PublishResponse::Accepted c)
                 (:wat::core::format "Accepted({c})" :c c))
               (_ "other")))
           (_ "recv-failed"))
     _keep th
     _keep2 iqh
     _keep3 ish]
    tag))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::format "{a};nsubs7-pub10={c}" :a (:once::split-case) :c (:once::cliff)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
