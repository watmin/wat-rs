;; probe-the-topic-publishes-a-batch.wat — Topic::publish carries a vector, cap 10.
;;
;; Through `:demo::Topic/publish`: 10 messages accepted (inbox depth +10×nsubs),
;; 11 rejected RequestTooManyEntries(11,10) with depth unchanged, and
;; `:demo::Topic::PUBLISH-MAX-ENTRIES` readable as 10.

(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")

(:wat::core::defn :tp::msgs [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::format "m{i}" :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :tp::publish-tag
  [t <- :demo::Topic  n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:demo::Topic/publish t (:demo::Topic::PublishRequest :msgs (:tp::msgs n)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:demo::Topic::PublishResponse::Ok) "Ok")
        ((:demo::Topic::PublishResponse::Full _d _c) "Full")
        ((:demo::Topic::PublishResponse::RequestTooLarge _b _c) "RequestTooLarge")
        ((:demo::Topic::PublishResponse::RequestTooManyEntries e c)
          (:wat::core::format "RequestTooManyEntries({e},{c})" :e e :c c))
        ((:demo::Topic::PublishResponse::RequestMalformed _p _e _g) "RequestMalformed")))
    (_ "recv-failed")))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [nsubs 2
     ish (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     iqh (:queue::queue/start :locus (:wat::spawn::thread)
           :record (:queue::queue::Record :cap 64
                     :store-addr (:wat::query::mem-store::Handle/addr ish)
                     :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     th (:demo::topic/start :locus (:wat::spawn::thread)
          :record (:demo::topic::Record :nsubs nsubs :inbox-addr (:queue::queue::Handle/addr iqh)))
     t  (:demo::dial-topic (:demo::topic::Handle/addr th))
     d0 (:demo::depth-of-topic t)
     t11 (:tp::publish-tag t 11)
     d11 (:demo::depth-of-topic t)
     t10 (:tp::publish-tag t 10)
     d10 (:demo::depth-of-topic t)
     _keep th
     _keep2 iqh
     _keep3 ish]
    (:wat::core::format
      "cap={cap};nsubs={ns};11={t11};depth {d0}->{d11};10={t10};depth {d11}->{d10}"
      :cap :demo::Topic::PUBLISH-MAX-ENTRIES
      :ns nsubs :t11 t11 :d0 d0 :d11 d11 :t10 t10 :d10 d10)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
