(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::Record/count (:my::counter::State/durable s)))))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply (:my::counter::State (:my::counter::Record c)) (:my::counter::IncrementResponse c))))])

;; Unwrap a Reply enum → extract the `value` field from the inner Response record.
;; Each Reply variant carries `resp <- <Op>Response`; Response carries `value <- :i64`.
(:wat::core::defn :user::reply-value [r <- :my::counter::Reply] -> :wat::core::i64
  (:wat::core::match r -> :wat::core::i64
    ((:my::counter::Reply::Get resp) (:my::counter::GetResponse/value resp))
    ((:my::counter::Reply::Increment resp) (:my::counter::IncrementResponse/value resp))))

;; Hand-drive the GENERATED serve (C.3 will wrap start + clients). Mirrors c0b1b's thread-tier
;; driver: parent mints the listener, spawns serve with the captured listener + empty clients +
;; initial state (State/new (Record 0)), connects a client, round-trips two ops, reads the typed Reply.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::thread) :my::counter::Op :my::counter::Reply)
     l    (:wat::spawn::Bound/listener pair)
     addr (:wat::spawn::Bound/address pair)
     ;; arc 291 3a-ii-β: serve's `self` is the lineage self-peer (Peer'<Status,Admin>),
     ;; not a client peer. The clients Vector stays the client type (Peer'<Reply,Op>).
     svc  (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<my::counter::Status,my::counter::Admin>] -> :wat::core::nil
              (:my::counter::serve self l
                (:wat::core::Vector :wat::kernel::Peer'<my::counter::Reply,my::counter::Op>)
                (:my::counter::State (:my::counter::Record 0)))))
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c (:my::counter::Op::Increment (:my::counter/increment-request 5)))
     r1   (:wat::kernel::recv' c)
     _    (:wat::kernel::send' c (:my::counter::Op::Get (:my::counter/get-request)))
     r2   (:wat::kernel::recv' c)]
    ;; Increment 5 → state 0→5, reply IncrementResponse{5}; Get → reply GetResponse{5}.
    ;; Assert the Get reply's value is 5.
    ;; Scope-exit drops `svc` → RAII drain → :Shutdown → serve exits → join completes.
    (:user::reply-value r2)))
