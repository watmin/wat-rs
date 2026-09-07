;; probe-a-caller-chunks-to-the-declared-limit.wat
;;
;; send-all chunks to SEND-MAX-ENTRIES (64). Prefix is the first Accepted count
;; in order. A short chunk stops the rest. Plain send of 65 is still the wall.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defn :ch::bodies [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::format "b{i}" :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :ch::tag
  [r <- (:wat::kernel::RecvOutcome :- [:queue::Queue::SendResponse])] -> :wat::core::String
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message m)
      (:wat::core::match m
        ((:queue::Queue::SendResponse::Accepted n)
          (:wat::core::format "Accepted({n})" :n n))
        ((:queue::Queue::SendResponse::RequestTooLarge _b _c) "RequestTooLarge")
        ((:queue::Queue::SendResponse::RequestTooManyEntries e c)
          (:wat::core::format "RequestTooManyEntries({e},{c})" :e e :c c))
        ((:queue::Queue::SendResponse::RequestMalformed _p _e _g) "RequestMalformed")))
    (_ "recv-failed")))

(:wat::core::defn :ch::join
  [envs <- (:wat::core::Vector :- [:queue::Envelope])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  e <- :queue::Envelope]
      -> :wat::core::String
      (:wat::core::if (:wat::core::= acc "")
        (:queue::Envelope/body e)
        (:wat::core::format "{a},{b}" :a acc :b (:queue::Envelope/body e))))
    ""
    envs))

(:wat::core::defn :ch::recv-all
  [q <- :queue::Queue  name <- :wat::core::String] -> (:wat::core::Vector :- [:queue::Envelope])
  (:wat::core::match
    (:queue::Queue/receive q
      (:queue::Queue::ReceiveRequest
        :queue name
        :now-ns (:wat::time::epoch-nanos (:wat::time::now))
        :visibility-ns 1000000000000
        :limit 100
        :wait (:queue::Queue::Wait::Immediate)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::ReceiveResponse::Ok envs) envs)
        (_ (:wat::core::Vector :- [:queue::Envelope]))))
    (_ (:wat::core::Vector :- [:queue::Envelope]))))

(:wat::core::defn :ch::boot
  [cap <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::query::mem-store::Handle :queue::queue::Handle :queue::Queue])
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record :cap cap
                    :store-addr (:wat::query::mem-store::Handle/addr msh)
                    :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q (:user::dial-queue (:queue::queue::Handle/addr qh))]
    (:wat::core::Tuple msh qh q)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [now (:wat::time::epoch-nanos (:wat::time::now))
     ;; Gate 1+2: cap 6, send-all 10. One chunk (10<64). Accepted 6, bodies b0..b5, no b6+.
     b1 (:ch::boot 6)
     q1 (:wat::core::third b1)
     t1 (:ch::tag
          (:queue::Queue/send-all q1
            (:queue::Queue::SendRequest :queue "q" :bodies (:ch::bodies 10) :now-ns now)))
     e1 (:ch::recv-all q1 "q")
     j1 (:ch::join e1)
     n1 (:wat::core::count e1)
     ;; Gate 2 extra: cap 6, send-all 80. Chunk 1 is 64, admitted 6, stop. No b64.
     b2 (:ch::boot 6)
     q2 (:wat::core::third b2)
     t2 (:ch::tag
          (:queue::Queue/send-all q2
            (:queue::Queue::SendRequest :queue "q" :bodies (:ch::bodies 80) :now-ns now)))
     e2 (:ch::recv-all q2 "q")
     j2 (:ch::join e2)
     has-b64 (:wat::core::foldl
               (:wat::core::fn [hit <- :wat::core::bool  e <- :queue::Envelope]
                 -> :wat::core::bool
                 (:wat::core::or hit (:wat::core::= (:queue::Envelope/body e) "b64")))
               false
               e2)
     ;; Gate 3: cap 64, send-all 40. 40<64 → one chunk. Full accept.
     b3 (:ch::boot 64)
     q3 (:wat::core::third b3)
     t3 (:ch::tag
          (:queue::Queue/send-all q3
            (:queue::Queue::SendRequest :queue "q" :bodies (:ch::bodies 40) :now-ns now)))
     n3 (:wat::core::count (:ch::recv-all q3 "q"))
     nchunks40 (:wat::i64::/ (:wat::i64::+ 40 63) 64)
     ;; Gate 4: plain send of 65 is the wall.
     b4 (:ch::boot 64)
     q4 (:wat::core::third b4)
     t4 (:ch::tag
          (:queue::Queue/send q4
            (:queue::Queue::SendRequest :queue "q" :bodies (:ch::bodies 65) :now-ns now)))
     n4 (:wat::core::count (:ch::recv-all q4 "q"))]
    (:wat::core::format
      "cap={cap};field={field};prefix={t1};bodies={j1};n={n1};short={t2};short-bodies={j2};b64={b64};one={t3};one-n={n3};nchunks40={nc};wall={t4};wall-n={n4}"
      :cap :queue::Queue::SEND-MAX-ENTRIES
      :field :queue::Queue::SEND-MAX-ENTRIES-FIELD
      :t1 t1 :j1 j1 :n1 n1
      :t2 t2 :j2 j2 :b64 (:wat::core::if has-b64 "yes" "no")
      :t3 t3 :n3 n3 :nc nchunks40
      :t4 t4 :n4 n4)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
