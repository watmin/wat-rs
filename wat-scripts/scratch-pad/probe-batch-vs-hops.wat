;; probe-batch-vs-hops.wat — is the 143us hop FIXED overhead, or does it scale with payload?
;;
;; probe-hop-cost.wat: a do-nothing hop is 143us (thread) / 183us (process).
;; probe-guard-cost.wat: the serve loop's size guard is ~1-2.4us of that. Not the term.
;; The remaining suspect is the park/wake pair -- the caller's recv parks, the server's select
;; wakes -- which is FIXED per hop and would explain why thread and process are within 1.27x.
;;
;; THE TEST. Move the same 1000 items across the boundary three ways. If per-hop cost is fixed
;; overhead, total time collapses as the batch grows and the attack is FEWER HOPS. If it scales
;; with payload, the hop itself is the target and batching buys nothing.

(:wat::core::defsurface :probe::Bulk :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Bulk::TakeRequest
     [items <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :probe::Bulk::TakeResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(take [self <- :probe::Bulk  req <- :probe::Bulk::TakeRequest]
     -> :probe::Bulk::TakeResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::bulk
  :satisfies :probe::Bulk
  :durable   []
  :ephemeral []
  :impls
  [(take [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some
         (:probe::Bulk::Reply::Take
           (:probe::Bulk::TakeResponse::Ok
             (:wat::core::count (:probe::Bulk::TakeRequest/items req)))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Bulk::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::bulk::Op])])))])

(:wat::core::defn :probe::dial-bulk
  [a <- (:wat::kernel::Address :- [:probe::Bulk::Op :probe::Bulk::Reply])] -> :probe::Bulk
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "dial-bulk failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::batch [k <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::str i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 k)))

;; total-items stays 1000; only the batch size changes.
(:wat::core::defn :probe::run-ms
  [c <- :probe::Bulk  calls <- :wat::core::i64  per <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [payload (:probe::batch per)
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::nil  _i <- :wat::core::i64] -> :wat::core::nil
            (:wat::core::match (:probe::Bulk/take c (:probe::Bulk::TakeRequest :items payload))
              ((:wat::kernel::RecvOutcome::Message _r) nil)
              (_ (:wat::kernel::assertion-failed! "take failed" :wat::core::None :wat::core::None))))
          nil
          (:wat::core::range 0 calls))
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)))

(:wat::core::defn :probe::sweep-thread [] -> :wat::core::String
  (:wat::core::let
    [h (:probe::bulk/start :locus (:wat::spawn::thread) :record (:probe::bulk::Record))
     c (:probe::dial-bulk (:probe::bulk::Handle/addr h))
     a (:probe::run-ms c 1000 1)
     b (:probe::run-ms c 100 10)
     d (:probe::run-ms c 10 100)
     e (:probe::run-ms c 1 1000)
     out (:wat::core::format "1000x1={a}ms 100x10={b}ms 10x100={d}ms 1x1000={e}ms" :a a :b b :d d :e e)]
    out))

(:wat::core::defn :probe::sweep-process [] -> :wat::core::String
  (:wat::core::let
    [h (:probe::bulk/start :locus (:wat::spawn::process) :record (:probe::bulk::Record))
     c (:probe::dial-bulk (:probe::bulk::Handle/addr h))
     a (:probe::run-ms c 1000 1)
     b (:probe::run-ms c 100 10)
     d (:probe::run-ms c 10 100)
     e (:probe::run-ms c 1 1000)
     out (:wat::core::format "1000x1={a}ms 100x10={b}ms 10x100={d}ms 1x1000={e}ms" :a a :b b :d d :e e)]
    out))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:probe::sweep-thread))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [t (:probe::sweep-thread)
     p (:probe::sweep-process)]
    (:wat::kernel::println (:wat::core::format "THREAD  {t}" :t t))
    (:wat::kernel::println (:wat::core::format "PROCESS {p}" :p p))))
