;; arc 278 S4c: the counter's protocol is LIFTED into an explicit surface (:my::Counter,
;; :nature :wat::kernel::Peer') the service WEARS via :satisfies + :impls (the retired :ops clause
;; is gone). This probe HAND-DRIVES the generated `serve` loop directly — Op/Reply are now the
;; SURFACE's synthesized enums (:my::Counter::Op / :my::Counter::Reply); serve + the lineage
;; Status/Admin peers stay per-service (:my::counter::serve, :my::counter::Status/Admin).
(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest        [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :my::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :my::Counter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get       [self <- :my::Counter  req <- :my::Counter::GetRequest]       -> :my::Counter::GetResponse :max-request-bytes 524288)
   (increment [self <- :my::Counter  req <- :my::Counter::IncrementRequest] -> :my::Counter::IncrementResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::counter
  :satisfies :my::Counter
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s
       (:my::Counter::GetResponse::Ok (:my::counter::Record/count (:my::counter::State/durable s)))))
   (increment [s ctx req]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s))
                                             (:my::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply (:my::counter::State :durable (:my::counter::Record :count c))
                                      (:my::Counter::IncrementResponse::Ok c))))])

;; Unwrap a Reply enum → extract the `value` field from the inner Response record.
;; Each Reply variant carries `resp <- <Op>Response`; Response carries `value <- :i64`.
(:wat::core::defn :user::reply-value [r <- :my::Counter::Reply] -> :wat::core::i64
  (:wat::core::match r 
    ((:my::Counter::Reply::Get resp)
     (:wat::core::match resp ((:my::Counter::GetResponse::Ok value) value)
       ((:my::Counter::GetResponse::RequestTooLarge bytes cap)
         (:wat::kernel::assertion-failed! "reply-value: unexpected GetResponse::RequestTooLarge"
           :wat::core::None :wat::core::None))
       ((:my::Counter::GetResponse::RequestMalformed mpath mexpected mgot)
         (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:my::Counter::Reply::Increment resp)
     (:wat::core::match resp ((:my::Counter::IncrementResponse::Ok value) value)
       ((:my::Counter::IncrementResponse::RequestTooLarge bytes cap)
         (:wat::kernel::assertion-failed! "reply-value: unexpected IncrementResponse::RequestTooLarge"
           :wat::core::None :wat::core::None))
       ((:my::Counter::IncrementResponse::RequestMalformed mpath mexpected mgot)
         (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ;; arc 278 no-hidden-failures — the reserved protocol-tier failure. Unreachable here
    ;; (recv' surfaces Reply::Failed as a raise BEFORE this helper sees it), but the
    ;; hand-written match must stay exhaustive + honest: surface the cause, never `_`-swallow.
    ((:my::Counter::Reply::Failed cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause)
        :wat::core::None :wat::core::None))))

;; Hand-drive the GENERATED serve (C.3 wraps start + clients). Mirrors c0b1b's thread-tier
;; driver: parent mints the listener, spawns serve with the captured listener + empty clients +
;; initial state (State (Record 0)), connects a client, round-trips two ops, reads the typed Reply.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :my::Counter::Op :my::Counter::Reply)
     l    (:wat::spawn::Bound/listener pair)
     addr (:wat::spawn::Bound/address pair)
     ;; arc 291 3a-ii-β: serve's `self` is the lineage self-peer (Peer'<Status,Admin>),
     ;; not a client peer. The clients Vector stays the client type (Peer'<Reply,Op>).
     ;; arc 278 the call context — the GENERATED `serve` grew a 5th positional arg (`next-id`,
     ;; the monotonic caller-id counter) and its `clients` slot is now Tuple<i64,Peer<…>>
     ;; entries (the id travels WITH its peer), not the bare Peer vector.
     svc  (:wat::test::spawn-peer (:wat::spawn::thread)
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:my::counter::Status :my::counter::Admin])] -> :wat::core::nil
              (:my::counter::serve self l
                ;; arc 278 the call context — the Op slot must be the SERVICE superset type
                ;; (`my::counter::Op`, not the surface `my::Counter::Op`) DIRECTLY: the
                ;; surface-Op<:service-Op widening is proven for a BARE Peer compare
                ;; (assignable's Peer-specific branch) but does not propagate through a
                ;; Tuple wrapper (unify recurses into tuple elements without re-entering
                ;; assignable) — so an empty vector built at the widened surface type no
                ;; longer round-trips once `clients`' element is Tuple<i64,Peer<…>>.
                (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 (:wat::kernel::Peer :- [:my::Counter::Reply :my::counter::Op])]))
                0
                (:my::counter::State :durable (:my::counter::Record :count 0)))))
     c    (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _    (:wat::core::match (:wat::kernel::send c (:my::Counter::Op::Increment (:my::Counter::IncrementRequest :n 5))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))  ;; arc 278 #73 — the recv' below already faces the stop
     r1   (:wat::core::match (:wat::kernel::recv c)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed! "recv': stopped before the Increment reply — the peer was ALIVE" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': c closed before the Increment reply" :wat::core::None :wat::core::None)))
     _    (:wat::core::match (:wat::kernel::send c (:my::Counter::Op::Get (:my::Counter::GetRequest))) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))  ;; arc 278 #73 — the recv' below already faces the stop
     r2   (:wat::core::match (:wat::kernel::recv c)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed! "recv': stopped before the Get reply — the peer was ALIVE" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': c closed before the Get reply" :wat::core::None :wat::core::None)))]
    ;; Increment 5 → state 0→5, reply IncrementResponse{5}; Get → reply GetResponse{5}.
    ;; Assert the Get reply's value is 5.
    ;; Scope-exit drops `svc` → RAII drain → :Shutdown → serve exits → join completes.
    (:user::reply-value r2)))
