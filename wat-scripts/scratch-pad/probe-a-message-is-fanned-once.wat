;; probe-a-message-is-fanned-once.wat
;;
;; ⚠ RETARGETED by "the inbox holds messages, not pairs" (excursus 001). The split
;; this probe was built to catch NO LONGER EXISTS, and its absence is the finding.
;;
;; Before: the inbox held (message, subscriber) pairs, so a 2-message publish to
;; nsubs 4 offered 8 pairs into a cap-6 inbox, took 6 msg-major, and needed a
;; `rem`/`need` top-up for message 1's missing 2 pairs — Accepted 1 (floor).
;;
;; Now: the inbox holds MESSAGES. nsubs 4, inbox cap 6, publish 2 → 2 bodies into
;; room 6 → Accepted(2), both rows present, bodies "p0|{t0b}" and "p1|{t0b}" with
;; no subscriber index. A message is one row: it cannot be half-admitted, so there
;; is no split case and no top-up to repair one.
;; nsubs 7, publish 10 against cap 64: a count, no assertion (10 rows, not 70).

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
          :record (:demo::topic::Record :inbox-addr (:queue::queue::Handle/addr iqh) :inbox-lost 0 :inbox-closed 0 :inbox-timedout 0))
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
          :record (:demo::topic::Record :inbox-addr (:queue::queue::Handle/addr iqh) :inbox-lost 0 :inbox-closed 0 :inbox-timedout 0))
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
