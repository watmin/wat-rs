;; tests/comms/probe_a_peer_remembers_its_address.wat — co-located fixture
;; for probe_a_peer_remembers_its_address.rs. a-peer-remembers-its-address.
;;
;; Process-tier: a dialed client remembers; an accepted peer reports None;
;; connect the remembered address (a real second connection); the original
;; peer still serves after the peek.
;; Thread-tier: both sides None (no socket address exists).
;; Timer: None (dead sentinel).

(:wat::core::defn :probe::render
  [o <- (:wat::core::Option :- [(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64])])]
  -> :wat::core::String
  (:wat::core::match o
    (:wat::core::None "None")
    ((:wat::core::Some _) "Some")))

(:wat::core::defn :probe::connect
  [addr <- (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64])]
  -> (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::accept
  [l <- (:wat::kernel::Listener :- [:wat::core::i64 :wat::core::i64])]
  -> (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:wat::kernel::accept l)
    ((:wat::kernel::AcceptOutcome::Accepted p) p)
    (:wat::kernel::AcceptOutcome::Closed
      (:wat::kernel::assertion-failed! "accept: closed" :wat::core::None :wat::core::None))
    ((:wat::kernel::AcceptOutcome::Failed c)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::send
  [p <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])  n <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::send p n)
    (:wat::kernel::SendOutcome::Sent nil)
    (:wat::kernel::SendOutcome::Closed
      (:wat::kernel::assertion-failed! "send: closed" :wat::core::None :wat::core::None))
    ((:wat::kernel::SendOutcome::Lost _c)
      (:wat::kernel::assertion-failed! "send: lost" :wat::core::None :wat::core::None))
    (:wat::kernel::SendOutcome::Stopped
      (:wat::kernel::assertion-failed! "send: stopped" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::recv
  [p <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::match (:wat::kernel::recv p)
    ((:wat::kernel::RecvOutcome::Message m) m)
    ((:wat::kernel::RecvOutcome::Lost c)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message c) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv: stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv: closed" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::TimedOut
      (:wat::kernel::assertion-failed! "recv: timed out" :wat::core::None :wat::core::None))
    ((:wat::kernel::RecvOutcome::Malformed _c)
      (:wat::kernel::assertion-failed! "recv: malformed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::echo
  [client <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])
   server <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])
   n <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [_ (:probe::send client n)
     got (:probe::recv server)
     _ (:probe::send server (:wat::core::* got 2))]
    (:probe::recv client)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [pb (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
     pl (:wat::spawn::Bound/listener pb)
     pa (:wat::spawn::Bound/address pb)
     pc (:probe::connect pa)
     ps (:probe::accept pl)
     pc-df (:probe::render (:wat::kernel::dialed-from pc))
     ps-df (:probe::render (:wat::kernel::dialed-from ps))
     orig (:probe::echo pc ps 5)
     fresh-addr (:wat::core::match (:wat::kernel::dialed-from pc)
                  ((:wat::core::Some a) a)
                  (:wat::core::None
                    (:wat::kernel::assertion-failed! "process client reported None" :wat::core::None :wat::core::None)))
     fc (:probe::connect fresh-addr)
     fs (:probe::accept pl)
     fc-df (:probe::render (:wat::kernel::dialed-from fc))
     fresh (:probe::echo fc fs 7)
     tb (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     tl (:wat::spawn::Bound/listener tb)
     ta (:wat::spawn::Bound/address tb)
     tc (:probe::connect ta)
     ts (:probe::accept tl)
     tc-df (:probe::render (:wat::kernel::dialed-from tc))
     ts-df (:probe::render (:wat::kernel::dialed-from ts))
     thread-orig (:probe::echo tc ts 5)
     timer (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Milliseconds 1) 0)
     timer-df (:wat::core::match (:wat::kernel::dialed-from timer)
                (:wat::core::None "None")
                ((:wat::core::Some _) "Some"))]
    (:wat::core::format
      "client-proc={c};accepted-proc={s};fresh={f};orig={o};fresh-reply={fr};client-thread={tc};accepted-thread={ts};thread-orig={to};timer={tm}"
      :c pc-df :s ps-df :f fc-df :o orig :fr fresh :tc tc-df :ts ts-df :to thread-orig :tm timer-df)))
