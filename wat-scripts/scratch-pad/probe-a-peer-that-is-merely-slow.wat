;; a-peer-that-is-merely-slow — delay-bp + delay-ms on faulting-store.
;; Wait is `after` (timer peer), not a sleep. Knobs default OFF.
;;
;;   :slow::run-rates          delay-bp 0 vs 10000, delay-ms 200
;;   :slow::run-late-vs-absent recv-by-deadline 50 then 2000: Message vs TimedOut
;;   :slow::run-desync         re-ask leaves a second frame; re-recv drains
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-a-peer-that-is-merely-slow.wat

(:wat::config::set-redef! true)
(:wat::load-file! "../query/faulting-store.wat")

(:wat::core::defn :slow::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    (_ (:wat::kernel::assertion-failed! "slow: dial failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :slow::row
  [sk <- :wat::core::String]
  -> :wat::query::StoredRow
  (:wat::query::StoredRow
    :pk "u#1" :sk sk :data "{:v 1}"
    :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey])))

(:wat::core::defn :slow::put-req
  [sk <- :wat::core::String]
  -> :wat::query::Store::PutRequest
  (:wat::query::Store::PutRequest
    (:wat::core::Vector :- [:wat::query::StoredRow] (:slow::row sk))))

(:wat::core::defn :slow::recv-name
  [o <- (:wat::kernel::RecvOutcome :- [:wat::query::Store::Reply])]
  -> :wat::core::String
  (:wat::core::match o
    ((:wat::kernel::RecvOutcome::Message _m) "Message")
    ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
    ((:wat::kernel::RecvOutcome::Malformed _c) "Malformed")))

(:wat::core::defn :slow::put-name
  [o <- (:wat::kernel::RecvOutcome :- [:wat::query::Store::PutResponse])]
  -> :wat::core::String
  (:wat::core::match o
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::PutResponse::Success) "Success")
        (_ "other-put")))
    ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
    ((:wat::kernel::RecvOutcome::Malformed _c) "Malformed")))

(:wat::core::defn :slow::send-put
  [p <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   sk <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::send p (:wat::query::Store::Op::Put (:slow::put-req sk)))
    (:wat::kernel::SendOutcome::Sent "Sent")
    (:wat::kernel::SendOutcome::Closed "Closed")
    (:wat::kernel::SendOutcome::Stopped "Stopped")
    ((:wat::kernel::SendOutcome::Lost _c) "Lost")))

(:wat::core::defn :slow::delays-of
  [ph <- :query::faulting-store::Handle]
  -> :wat::core::i64
  (:wat::core::match (:query::faulting-store/stop ph)
    ((:wat::service::StopOutcome::Stopped rec)
      (:query::faulting-store::Record/delays-fired rec))
    (_ -1)))

(:wat::core::defn :slow::boot
  [drop-bp <- :wat::core::i64  die-bp <- :wat::core::i64
   delay-bp <- :wat::core::i64  delay-ms <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:query::faulting-store::Handle
                            :wat::query::mem-store::Handle])
  (:wat::core::let
    [sh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     ph (:query::faulting-store/start :locus (:wat::spawn::thread)
          :record (:query::faulting-store::Record
                    :real-addr (:wat::query::mem-store::Handle/addr sh)
                    :drop-reply-bp drop-bp
                    :die-bp die-bp
                    :delay-bp delay-bp
                    :delay-ms delay-ms
                    :seed 1
                    :drops-fired 0
                    :dies-fired 0
                    :delays-fired 0))]
    (:wat::core::Tuple ph sh)))

(:wat::core::defn :slow::one-rate
  [delay-bp <- :wat::core::i64  delay-ms <- :wat::core::i64  sk <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let
    [boot (:slow::boot 0 0 delay-bp delay-ms)
     ph (:wat::core::first boot)
     sh (:wat::core::second boot)
     p (:slow::dial-store (:query::faulting-store::Handle/addr ph))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     lab (:slow::put-name (:wat::query::Store/put p (:slow::put-req sk)))
     elapsed (:wat::i64::/ (:wat::i64::- (:wat::time::epoch-nanos (:wat::time::now)) t0) 1000000)
     fired (:slow::delays-of ph)]
    (:wat::core::format "put={p};elapsed-ms={e};delays-fired={d}"
      :p lab :e elapsed :d fired)))

(:wat::core::defn :slow::run-rates [] -> :wat::core::String
  (:wat::core::let
    [off (:slow::one-rate 0 200 "off")
     on  (:slow::one-rate 10000 200 "on")]
    (:wat::core::format "disarmed {a} ;; armed {b}" :a off :b on)))

(:wat::core::defn :slow::two-recv
  [drop-bp <- :wat::core::i64  delay-bp <- :wat::core::i64  delay-ms <- :wat::core::i64
   sk <- :wat::core::String  first-ms <- :wat::core::i64  second-ms <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::let
    [boot (:slow::boot drop-bp 0 delay-bp delay-ms)
     ph (:wat::core::first boot)
     sh (:wat::core::second boot)
     p (:slow::dial-store (:query::faulting-store::Handle/addr ph))
     sent (:slow::send-put p sk)
     a (:slow::recv-name (:wat::kernel::recv-by-deadline p first-ms))
     b (:slow::recv-name (:wat::kernel::recv-by-deadline p second-ms))
     fired (:slow::delays-of ph)]
    (:wat::core::format "sent={s};first={a};second={b};delays-fired={d}"
      :s sent :a a :b b :d fired)))

(:wat::core::defn :slow::run-late-vs-absent [] -> :wat::core::String
  (:wat::core::let
    [late (:slow::two-recv 0 10000 400 "late" 50 2000)
     absent (:slow::two-recv 10000 0 0 "absent" 50 300)]
    (:wat::core::format "late {a} ;; absent {b}" :a late :b absent)))

(:wat::core::defn :slow::run-desync [] -> :wat::core::String
  (:wat::core::let
    [boot-ask (:slow::boot 0 0 10000 400)
     ph-ask (:wat::core::first boot-ask)
     sh-ask (:wat::core::second boot-ask)
     p-ask (:slow::dial-store (:query::faulting-store::Handle/addr ph-ask))
     sent1 (:slow::send-put p-ask "a")
     first-ask (:slow::recv-name (:wat::kernel::recv-by-deadline p-ask 50))
     sent2 (:slow::send-put p-ask "b")
     after-reask (:slow::recv-name (:wat::kernel::recv-by-deadline p-ask 2000))
     leftover-ask (:slow::recv-name (:wat::kernel::recv-by-deadline p-ask 2000))
     _ (:slow::delays-of ph-ask)
     boot-recv (:slow::boot 0 0 10000 400)
     ph-recv (:wat::core::first boot-recv)
     sh-recv (:wat::core::second boot-recv)
     p-recv (:slow::dial-store (:query::faulting-store::Handle/addr ph-recv))
     sent3 (:slow::send-put p-recv "c")
     first-recv (:slow::recv-name (:wat::kernel::recv-by-deadline p-recv 50))
     drained (:slow::recv-name (:wat::kernel::recv-by-deadline p-recv 2000))
     sent4 (:slow::send-put p-recv "d")
     own (:slow::recv-name (:wat::kernel::recv-by-deadline p-recv 2000))
     leftover-recv (:slow::recv-name (:wat::kernel::recv-by-deadline p-recv 50))
     _ (:slow::delays-of ph-recv)]
    (:wat::core::format
      "reask sent1={s1};first={f1};sent2={s2};after-reask={ar};leftover={la} ;; rerecv sent1={s3};first={f3};drained={dr};sent2={s4};own={ow};leftover={lr}"
      :s1 sent1 :f1 first-ask :s2 sent2 :ar after-reask :la leftover-ask
      :s3 sent3 :f3 first-recv :dr drained :s4 sent4 :ow own :lr leftover-recv)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:slow::run-rates))
    (:wat::kernel::println (:slow::run-late-vs-absent))
    (:wat::kernel::println (:slow::run-desync))))
