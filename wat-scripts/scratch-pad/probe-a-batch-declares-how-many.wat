;; probe-a-batch-declares-how-many.wat — `:max-entries [bodies 10]` on Queue::send.
;;
;; 11 bodies into a cap-10 send must be RequestTooManyEntries{11,10} with depth
;; unchanged, at BOTH thread and process loci. 10 still passes (Ok, depth +10).
;; The cap is a readable def: :queue::Queue::SEND-MAX-ENTRIES = 10.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defn :bd::bodies [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::format "b{i}" :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :bd::dial-q
  [a <- (:wat::kernel::Address :- [:queue::Queue::Op :queue::Queue::Reply])] -> :queue::Queue
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "bd: dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :bd::depth [q <- :queue::Queue] -> :wat::core::i64
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks visible _unacked) visible)
        (other (:wat::kernel::assertion-failed! (:wat::core::format "bd: stats not Ok: {o}" :o other) :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::kernel::assertion-failed! (:wat::core::format "bd: stats Lost: {m}" :m (:wat::kernel::LociDiedError/message c)) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "bd: stats Closed" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "bd: stats Stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::TimedOut
      (:wat::kernel::assertion-failed! "bd: stats TimedOut" :wat::core::None :wat::core::None))))

(:wat::core::defn :bd::send-tag
  [q <- :queue::Queue  n <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest
        :queue "q" :bodies (:bd::bodies n)
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Ok) "Ok")
        ((:queue::Queue::SendResponse::Full _d _c) "Full")
        ((:queue::Queue::SendResponse::RequestTooLarge _b _c) "RequestTooLarge")
        ((:queue::Queue::SendResponse::RequestTooManyEntries e c)
          (:wat::core::format "RequestTooManyEntries({e},{c})" :e e :c c))
        ((:queue::Queue::SendResponse::RequestMalformed _p _e _g) "RequestMalformed")))
    (_ "recv-failed")))

(:wat::core::defn :bd::run-at
  [q <- :queue::Queue] -> :wat::core::String
  (:wat::core::let
    [d0 (:bd::depth q)
     t11 (:bd::send-tag q 11)
     d11 (:bd::depth q)
     t10 (:bd::send-tag q 10)
     d10 (:bd::depth q)]
    (:wat::core::format "{t11};depth {d0}->{d11};{t10};depth {d11}->{d10}"
      :t11 t11 :d0 d0 :d11 d11 :t10 t10 :d10 d10)))

(:wat::core::defn :bd::thread [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh  (:queue::queue/start :locus (:wat::spawn::thread)
            :record (:queue::queue::Record :cap 1024
                      :store-addr (:wat::query::mem-store::Handle/addr msh)
                      :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q   (:bd::dial-q (:queue::queue::Handle/addr qh))
     out (:bd::run-at q)
     _keep qh
     _keep2 msh]
    out))

(:wat::core::defn :bd::process [] -> :wat::core::String
  (:wat::core::let
    [sh (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
          :record (:wat::query::sqlite-store::Record :path ":memory:"
                    :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     qh (:queue::queue/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:wat::query::sqlite-store/grant sh
                       (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl)))))
          :record (:queue::queue::Record :cap 1024
                    :store-addr (:wat::query::sqlite-store::Handle/addr sh)
                    :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:bd::dial-q (:queue::queue::Handle/addr qh))
     out (:bd::run-at q)
     _keep qh
     _keep2 sh]
    out))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [cap :queue::Queue::SEND-MAX-ENTRIES
     field :queue::Queue::SEND-MAX-ENTRIES-FIELD
     th (:bd::thread)
     pr (:bd::process)]
    (:wat::core::format "cap={cap};field={field};thread={th};process={pr}"
      :cap cap :field field :th th :pr pr)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
