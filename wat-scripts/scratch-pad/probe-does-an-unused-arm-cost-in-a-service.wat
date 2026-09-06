;; probe-does-an-unused-arm-cost-in-a-service.wat
;;
;; THE GAP. probe-does-an-unused-arm-cost.wat and its payload sibling both refute
;; "a match pays for unused arms" — 0 and 18 ns/op on a quiet box. But every one
;; of those probes, and grok's two as well, drives a plain `defn` through `foldl`.
;;
;; The 934ms lived in a DEFSERVICE arm. Nobody has measured inside one.
;;
;; Two services satisfying ONE surface. Identical except that :huge's `ping` impl
;; wraps its Outcome in a match whose unused arms are large — exactly sqs.wat's
;; shape. Same locus, same transport, same call count.
;;
;; If huge >> tiny here while the defn probes say 0, the cost is specific to
;; service-arm evaluation and THAT is the mechanism. If they match, unused arms
;; are free everywhere and the 934ms came from something else entirely.

(:wat::core::defsurface :ar::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :ar::Echo::PingRequest [])
   (:wat::core::defenum :ar::Echo::PingResponse :wat::enum::Pure
     :Pong            []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :ar::Echo  req <- :ar::Echo::PingRequest] -> :ar::Echo::PingResponse
     :max-request-bytes 524288)])

;; The discriminant the impl matches on. Always :Go.
(:wat::core::defenum :ar::Step :wat::enum::Pure
  :Go        []
  :Transient [err <- :wat::core::String]
  :Constraint [err <- :wat::core::String]
  :Fatal     [err <- :wat::core::String])

(:wat::core::defn :ar::step [] -> :ar::Step (:ar::Step::Go))

;; ─── TINY: the impl as probe-roundtrip-cost.wat writes it ────────────────────
(:wat::service::defservice :ar::tiny
  :satisfies :ar::Echo
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :ar::tiny::Record] -> :ar::tiny::State
          (:ar::tiny::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::core::match (:ar::step)
       ((:ar::Step::Go)
         (:wat::service::Outcome::Continue s (:wat::core::Some (:ar::Echo::Reply::Ping (:ar::Echo::PingResponse::Pong))) (:wat::core::Vector :- [(:wat::service::Directed :- [:ar::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:ar::tiny::Op])])))
       ((:ar::Step::Transient _e)
         (:wat::kernel::assertion-failed! "tiny Transient" :wat::core::None :wat::core::None))
       ((:ar::Step::Constraint _e)
         (:wat::kernel::assertion-failed! "tiny Constraint" :wat::core::None :wat::core::None))
       ((:ar::Step::Fatal _e)
         (:wat::kernel::assertion-failed! "tiny Fatal" :wat::core::None :wat::core::None))))])

;; ─── HUGE: same taken arm; the three unused arms carry large bodies ──────────
(:wat::service::defservice :ar::huge
  :satisfies :ar::Echo
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :ar::huge::Record] -> :ar::huge::State
          (:ar::huge::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::core::match (:ar::step)
       ((:ar::Step::Go)
         (:wat::service::Outcome::Continue s (:wat::core::Some (:ar::Echo::Reply::Ping (:ar::Echo::PingResponse::Pong))) (:wat::core::Vector :- [(:wat::service::Directed :- [:ar::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:ar::huge::Op])])))
       ((:ar::Step::Transient e)
         (:wat::core::let
           [a01 (:wat::core::count e) a02 (:wat::i64::+ a01 1) a03 (:wat::i64::* a02 3)
            a04 (:wat::core::format "{p}-{q}" :p e :q a03)
            a05 (:wat::core::Vector :- [:wat::core::String] e a04 "x" "y")
            a06 (:wat::core::foldl :ar::add 0 (:wat::core::range 0 8))
            a07 (:wat::core::format "{p}/{q}" :p a04 :q a06)
            a08 (:wat::core::count a07)
            a09 (:wat::core::match (:ar::step)
                  ((:ar::Step::Go) a08)
                  ((:ar::Step::Transient x) (:wat::core::count x))
                  ((:ar::Step::Constraint x) (:wat::core::count x))
                  ((:ar::Step::Fatal x) (:wat::core::count x)))
            a10 (:wat::i64::+ a09 (:wat::core::count a05))]
           (:wat::service::Outcome::Continue s (:wat::core::Some (:ar::Echo::Reply::Ping (:ar::Echo::PingResponse::RequestTooLarge a10 a06))) (:wat::core::Vector :- [(:wat::service::Directed :- [:ar::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:ar::huge::Op])]))))
       ((:ar::Step::Constraint e)
         (:wat::core::let
           [b01 (:wat::core::count e) b02 (:wat::i64::* b01 5) b03 (:wat::i64::+ b02 7)
            b04 (:wat::core::format "{p}|{q}" :p e :q b03)
            b05 (:wat::core::Vector :- [:wat::core::String] b04 e "z")
            b06 (:wat::core::foldl :ar::add 0 (:wat::core::range 0 11))
            b07 (:wat::core::format "{p}~{q}" :p b04 :q b06)
            b08 (:wat::core::count b07)
            b09 (:wat::core::match (:ar::step)
                  ((:ar::Step::Go) b08)
                  ((:ar::Step::Transient x) (:wat::core::count x))
                  ((:ar::Step::Constraint x) (:wat::core::count x))
                  ((:ar::Step::Fatal x) (:wat::core::count x)))
            b10 (:wat::i64::+ b09 (:wat::core::count b05))]
           (:wat::service::Outcome::Continue s (:wat::core::Some (:ar::Echo::Reply::Ping (:ar::Echo::PingResponse::RequestTooLarge b10 b06))) (:wat::core::Vector :- [(:wat::service::Directed :- [:ar::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:ar::huge::Op])]))))
       ((:ar::Step::Fatal e)
         (:wat::core::let
           [c01 (:wat::core::count e) c02 (:wat::i64::* c01 13) c03 (:wat::i64::+ c02 17)
            c04 (:wat::core::format "{p}#{q}" :p e :q c03)
            c05 (:wat::core::Vector :- [:wat::core::String] c04 e "w" "v" "u")
            c06 (:wat::core::foldl :ar::add 0 (:wat::core::range 0 14))
            c07 (:wat::core::format "{p}^{q}" :p c04 :q c06)
            c08 (:wat::core::count c07)
            c09 (:wat::core::match (:ar::step)
                  ((:ar::Step::Go) c08)
                  ((:ar::Step::Transient x) (:wat::core::count x))
                  ((:ar::Step::Constraint x) (:wat::core::count x))
                  ((:ar::Step::Fatal x) (:wat::core::count x)))
            c10 (:wat::i64::+ c09 (:wat::core::count c05))]
           (:wat::service::Outcome::Continue s (:wat::core::Some (:ar::Echo::Reply::Ping (:ar::Echo::PingResponse::RequestTooLarge c10 c06))) (:wat::core::Vector :- [(:wat::service::Directed :- [:ar::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:ar::huge::Op])]))))))])

(:wat::core::defn :ar::add [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ acc x))

(:wat::core::defn :ar::ping-n
  [c <- (:wat::kernel::Peer :- [:ar::Echo::Op :ar::Echo::Reply])  n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= n 0)
    nil
    (:wat::core::let
      [_r (:wat::core::match (:ar::Echo/ping c (:ar::Echo::PingRequest))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil)
            (:wat::kernel::RecvOutcome::TimedOut nil))]
      (:ar::ping-n c (:wat::i64::- n 1)))))

(:wat::core::defn :ar::run [] -> :wat::core::String
  (:wat::core::let
    [n  8000
     ht (:ar::tiny/start :locus (:wat::spawn::thread) :record (:ar::tiny::Record :n 0))
     hh (:ar::huge/start :locus (:wat::spawn::thread) :record (:ar::huge::Record :n 0))
     ct (:wat::core::match (:wat::kernel::connect (:ar::tiny::Handle/addr ht))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     ch (:wat::core::match (:wat::kernel::connect (:ar::huge::Handle/addr hh))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     t0 (:wat::time::epoch-nanos (:wat::time::now))  _1 (:ar::ping-n ct n)
     t1 (:wat::time::epoch-nanos (:wat::time::now))  _2 (:ar::ping-n ch n)
     t2 (:wat::time::epoch-nanos (:wat::time::now))  _3 (:ar::ping-n ct n)
     t3 (:wat::time::epoch-nanos (:wat::time::now))
     tiny-avg (:wat::i64::/ (:wat::i64::+ (:wat::i64::- t1 t0) (:wat::i64::- t3 t2)) 2)]
    (:wat::core::format
      "n={n};tiny1-ns/op={a};huge-ns/op={b};tiny2-ns/op={c};huge-minus-tiny-ns/op={d}"
      :n n
      :a (:wat::i64::/ (:wat::i64::- t1 t0) n)
      :b (:wat::i64::/ (:wat::i64::- t2 t1) n)
      :c (:wat::i64::/ (:wat::i64::- t3 t2) n)
      :d (:wat::i64::/ (:wat::i64::- (:wat::i64::- t2 t1) tiny-avg) n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:ar::run)))
