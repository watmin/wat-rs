;; Faulting Store proxy — the-store-can-fail.
;;
;; Satisfies :wat::query::Store. Forwards every op to a real store, then at a
;; configurable rate DESTROYS its own reply. The write has landed; the caller
;; cannot know. That is §2d.
;;
;; ⛔ Forward FIRST, suppress SECOND. Skipping the call inverts the condition.
;; ⛔ Count at the suppression site (None / Stop), never at the dice roll.
;; ⛔ Count delays at the wait site (the `after` recv), never at the dice roll.
;;
;; store-drop-reply-bp  — suppress the reply, keep serving → caller TimedOut (~10 s)
;; store-die-bp         — suppress the reply, then exit     → caller Lost / Closed
;; delay-bp + delay-ms  — wait on a timer peer, then reply  → caller sees a SLOW peer
;;                        (defaults 0: OFF)
;;
;; Put and delete take the knobs (the six §2d arms). ensure-schema / scan /
;; scan-index / count-index always pass through so init and reads stay honest.

(:wat::core::defn :query::faulting-store::expect-msg :- [O]
  [o <- (:wat::kernel::RecvOutcome :- [:O])  what <- :wat::core::String]
  -> :O
  (:wat::core::match o
    ((:wat::kernel::RecvOutcome::Message m) m)
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::kernel::assertion-failed!
        (:wat::string::interpolate "faulting-store: {w} Lost: {m}"
          :w what :m (:wat::kernel::LociDiedError/message c))
        :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed!
        (:wat::string::interpolate "faulting-store: {w} Closed" :w what)
        :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed!
        (:wat::string::interpolate "faulting-store: {w} Stopped" :w what)
        :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::TimedOut
      (:wat::kernel::assertion-failed!
        (:wat::string::interpolate "faulting-store: {w} TimedOut" :w what)
        :wat::core::None :wat::core::None))
    ((:wat::kernel::RecvOutcome::Malformed c)
      (:wat::kernel::assertion-failed!
        (:wat::kernel::Failure/message c)
        :wat::core::None :wat::core::None))))

;; Timer-channel wait. Exemplar: wat/queue.wat:1643. Not a sleep — there is none.
;; Count delays-fired at the caller after this returns, never at the dice roll.
(:wat::core::defn :query::faulting-store::nap
  [delay-ms <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= delay-ms 0)
    nil
    (:wat::core::match
      (:wat::kernel::recv
        (:wat::kernel::after :wat::program::PeerKind::thread
          (:wat::time::Milliseconds delay-ms) :done))
      ((:wat::kernel::RecvOutcome::Message _m) nil)
      ((:wat::kernel::RecvOutcome::Lost _c) nil)
      (:wat::kernel::RecvOutcome::Stopped nil)
      (:wat::kernel::RecvOutcome::Closed nil)
      (:wat::kernel::RecvOutcome::TimedOut nil)
      ((:wat::kernel::RecvOutcome::Malformed _c) nil))))

(:wat::service::defservice :query::faulting-store
  :satisfies :wat::query::Store
  :durable   [real-addr        <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])
              drop-reply-bp    <- :wat::core::i64
              die-bp           <- :wat::core::i64
              delay-bp         <- :wat::core::i64
              delay-ms         <- :wat::core::i64
              seed             <- :wat::core::i64
              drops-fired      <- :wat::core::i64
              dies-fired       <- :wat::core::i64
              delays-fired     <- :wat::core::i64]
  :ephemeral [real <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  :peers     [:wat::query::Store]
  :init (:wat::core::fn
          [record <- :query::faulting-store::Record]
          -> :query::faulting-store::State
          (:query::faulting-store::State
            :durable record
            :real (:wat::core::match
                    (:wat::kernel::connect (:query::faulting-store::Record/real-addr record))
                    ((:wat::kernel::ConnectOutcome::Connected p) p)
                    (_ (:wat::kernel::assertion-failed!
                         "faulting-store: connect to real store failed"
                         :wat::core::None :wat::core::None)))))
  :impls
  [(ensure-schema [s ctx req]
     (:wat::core::let
       [real (:query::faulting-store::State/real s)
        resp (:query::faulting-store::expect-msg
               (:wat::query::Store/ensure-schema real req)
               "ensure-schema")]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:wat::query::Store::Reply::EnsureSchema resp))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:query::faulting-store::Op])]))))

   (scan [s ctx req]
     (:wat::core::let
       [real (:query::faulting-store::State/real s)
        resp (:query::faulting-store::expect-msg
               (:wat::query::Store/scan real req)
               "scan")]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:wat::query::Store::Reply::Scan resp))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:query::faulting-store::Op])]))))

   (scan-index [s ctx req]
     (:wat::core::let
       [real (:query::faulting-store::State/real s)
        resp (:query::faulting-store::expect-msg
               (:wat::query::Store/scan-index real req)
               "scan-index")]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:wat::query::Store::Reply::ScanIndex resp))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:query::faulting-store::Op])]))))

   (count-index [s ctx req]
     (:wat::core::let
       [real (:query::faulting-store::State/real s)
        resp (:query::faulting-store::expect-msg
               (:wat::query::Store/count-index real req)
               "count-index")]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:wat::query::Store::Reply::CountIndex resp))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:query::faulting-store::Op])]))))

   (put [s ctx req]
     (:wat::core::let
       [real (:query::faulting-store::State/real s)
        rec  (:query::faulting-store::State/durable s)
        ;; FORWARD — the write lands on the real store before any roll.
        forwarded (:query::faulting-store::expect-msg
                    (:wat::query::Store/put real req)
                    "put")
        drop-bp (:query::faulting-store::Record/drop-reply-bp rec)
        die-bp  (:query::faulting-store::Record/die-bp rec)
        delay-bp (:query::faulting-store::Record/delay-bp rec)
        delay-ms (:query::faulting-store::Record/delay-ms rec)
        pair    (:wat::core::if (:wat::core::or (:wat::i64::> drop-bp 0)
                                  (:wat::core::or (:wat::i64::> die-bp 0) (:wat::i64::> delay-bp 0)))
                  (:wat::rand::int-from (:query::faulting-store::Record/seed rec) 0 10000)
                  (:wat::core::Tuple (:query::faulting-store::Record/seed rec) 0))
        seed1   (:wat::core::first pair)
        bp      (:wat::core::second pair)
        die?    (:wat::core::and (:wat::i64::> die-bp 0) (:wat::i64::< bp die-bp))
        drop?   (:wat::core::and (:wat::core::not die?)
                  (:wat::core::and (:wat::i64::> drop-bp 0) (:wat::i64::< bp drop-bp)))
        delay?  (:wat::core::and (:wat::core::not die?)
                  (:wat::core::and (:wat::core::not drop?)
                    (:wat::core::and (:wat::i64::> delay-bp 0) (:wat::i64::< bp delay-bp))))
        wait?   (:wat::core::and delay? (:wat::i64::> delay-ms 0))
        ;; Count delays-fired HERE, at the wait, never at the dice roll.
        _park   (:wat::core::if wait?
                  (:query::faulting-store::nap delay-ms)
                  nil)
        rec'    (:query::faulting-store::Record
                  :real-addr (:query::faulting-store::Record/real-addr rec)
                  :drop-reply-bp drop-bp
                  :die-bp die-bp
                  :delay-bp delay-bp
                  :delay-ms delay-ms
                  :seed seed1
                  :drops-fired (:wat::i64::+ (:query::faulting-store::Record/drops-fired rec)
                                 (:wat::core::if drop? 1 0))
                  :dies-fired (:wat::i64::+ (:query::faulting-store::Record/dies-fired rec)
                                (:wat::core::if die? 1 0))
                  :delays-fired (:wat::i64::+ (:query::faulting-store::Record/delays-fired rec)
                                  (:wat::core::if wait? 1 0)))
        s' (:query::faulting-store::State :durable rec' :real real)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none-arms  (:wat::core::Vector :- [(:wat::service::Alarm :- [:query::faulting-store::Op])])]
       (:wat::core::if die?
         ;; Counted on rec' (dies-fired) at this Stop — the suppression site.
         (:wat::service::Outcome::Stop s' :wat::core::None none-sends)
         (:wat::service::Outcome::Continue s'
           (:wat::core::if drop?
             :wat::core::None
             (:wat::core::Some (:wat::query::Store::Reply::Put forwarded)))
           none-sends
           none-arms))))

   (delete [s ctx req]
     (:wat::core::let
       [real (:query::faulting-store::State/real s)
        rec  (:query::faulting-store::State/durable s)
        forwarded (:query::faulting-store::expect-msg
                    (:wat::query::Store/delete real req)
                    "delete")
        drop-bp (:query::faulting-store::Record/drop-reply-bp rec)
        die-bp  (:query::faulting-store::Record/die-bp rec)
        delay-bp (:query::faulting-store::Record/delay-bp rec)
        delay-ms (:query::faulting-store::Record/delay-ms rec)
        pair    (:wat::core::if (:wat::core::or (:wat::i64::> drop-bp 0)
                                  (:wat::core::or (:wat::i64::> die-bp 0) (:wat::i64::> delay-bp 0)))
                  (:wat::rand::int-from (:query::faulting-store::Record/seed rec) 0 10000)
                  (:wat::core::Tuple (:query::faulting-store::Record/seed rec) 0))
        seed1   (:wat::core::first pair)
        bp      (:wat::core::second pair)
        die?    (:wat::core::and (:wat::i64::> die-bp 0) (:wat::i64::< bp die-bp))
        drop?   (:wat::core::and (:wat::core::not die?)
                  (:wat::core::and (:wat::i64::> drop-bp 0) (:wat::i64::< bp drop-bp)))
        delay?  (:wat::core::and (:wat::core::not die?)
                  (:wat::core::and (:wat::core::not drop?)
                    (:wat::core::and (:wat::i64::> delay-bp 0) (:wat::i64::< bp delay-bp))))
        wait?   (:wat::core::and delay? (:wat::i64::> delay-ms 0))
        _park   (:wat::core::if wait?
                  (:query::faulting-store::nap delay-ms)
                  nil)
        rec'    (:query::faulting-store::Record
                  :real-addr (:query::faulting-store::Record/real-addr rec)
                  :drop-reply-bp drop-bp
                  :die-bp die-bp
                  :delay-bp delay-bp
                  :delay-ms delay-ms
                  :seed seed1
                  :drops-fired (:wat::i64::+ (:query::faulting-store::Record/drops-fired rec)
                                 (:wat::core::if drop? 1 0))
                  :dies-fired (:wat::i64::+ (:query::faulting-store::Record/dies-fired rec)
                                (:wat::core::if die? 1 0))
                  :delays-fired (:wat::i64::+ (:query::faulting-store::Record/delays-fired rec)
                                  (:wat::core::if wait? 1 0)))
        s' (:query::faulting-store::State :durable rec' :real real)
        none-sends (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])])
        none-arms  (:wat::core::Vector :- [(:wat::service::Alarm :- [:query::faulting-store::Op])])]
       (:wat::core::if die?
         (:wat::service::Outcome::Stop s' :wat::core::None none-sends)
         (:wat::service::Outcome::Continue s'
           (:wat::core::if drop?
             :wat::core::None
             (:wat::core::Some (:wat::query::Store::Reply::Delete forwarded)))
           none-sends
           none-arms))))])
