;; Arc 278 BRIEF-client-validates-locally / DESIGN-STONE-the-client-validates-locally.md —
;; the generated client method refuses an over-budget request LOCALLY, against the surface's
;; own `:max-request-bytes` contract, and does NOT `recv` after a send it never made.
;;
;; STOP-1's own warning: a fat request refused locally and a fat request refused by the
;; SERVER's per-op guard (serve-op-arms' `:max-request-bytes` check) produce the byte-identical
;; `RequestTooLarge{bytes,cap}` value — asserting only "the response is RequestTooLarge" passes
;; identically whether the fix ran or not. THE DISCRIMINATOR here is deliberate:
;;
;;   surface :max-request-bytes = 100   (the CONTRACT both sides compile against)
;;   service :max-frame-bytes   = 2048  (FOO — the deployment's transport-level ceiling)
;;   poison payload              ~3200 bytes  (over BOTH — specifically over FOO)
;;
;; FOO is a pre-decode, transport-level byte check (`poll'` routes an over-FOO frame to
;; `ServiceEvent::Rejected` BEFORE any op-specific dispatch even runs — see
;; probe_arc278_service_max_frame_bytes.wat). So for THIS payload size, the exact
;; `RequestTooLarge{bytes,cap}` value is UNREACHABLE via the wire: if the client actually sent
;; it, FOO intercepts first and the connection is EVICTED (a Reply::Failed naming
;; `max-frame-bytes`, surfacing as `RecvOutcome::Lost`), never a matchable RequestTooLarge.
;; Getting that value back is therefore proof the client fabricated it without ever touching
;; the wire. And because FOO-eviction actually CLOSES the connection, a follow-up in-budget
;; request on the SAME peer succeeding is the literal "the peer's receiver has nothing
;; pending" proof STOP-1 asks for: an actually-sent poison frame would have killed `c` outright.

(:wat::core::defsurface :probe::Budget :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Budget::PutRequest [payload <- :wat::core::String])
   (:wat::core::defenum :probe::Budget::PutResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :probe::Budget  req <- :probe::Budget::PutRequest] -> :probe::Budget::PutResponse :max-request-bytes 100)])

(:wat::service::defservice :probe::budgetsvc
  :satisfies :probe::Budget
  :max-frame-bytes 2048
  :durable   []
  :ephemeral []
  :impls
  [(put [s ctx req] (:wat::service::Outcome::Reply s (:probe::Budget::PutResponse::Ok 7)))])

;; A String of exactly n*32 bytes (kept LOCAL — mirrors :probe::payload-of in the sibling FOO
;; fixture so this file has no cross-file dependency).
(:wat::core::defn :probe::budget-payload-of [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc "0123456789ABCDEF0123456789ABCDEF"))
    ""
    (:wat::core::range 0 n)))

;; THE DISCRIMINATOR PROBE. Terminal caller throughout: every unexpected shape SURFACES via
;; `assertion-failed!` rather than swallowing it — this probe's only job is to catch a wrong
;; shape, never to hide one. Returns the follow-up's `ok` (7) when everything holds.
(:wat::core::defn :user::over-budget-refused-locally-then-connection-survives [] -> :wat::core::i64
  (:wat::core::let
    [poison  (:probe::budget-payload-of 100)   ;; 100*32 = 3200 bytes > FOO(2048) > cap(100)
     h       (:probe::budgetsvc/start :locus (:wat::spawn::process) :record (:probe::budgetsvc::Record))
     c       (:wat::core::match (:wat::kernel::connect (:probe::budgetsvc::Handle/addr h))
                ((:wat::kernel::ConnectOutcome::Connected p) p)
                ((:wat::kernel::ConnectOutcome::Refused cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
                ((:wat::kernel::ConnectOutcome::Failed cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None)))
     r1      (:probe::Budget/put c (:probe::Budget::PutRequest :payload poison))
     ;; r1 MUST be the fabricated-locally RequestTooLarge{100, >100} — never Ok (accepted),
     ;; never Lost/Closed (only reachable if the poison frame actually reached the wire and
     ;; FOO evicted `c` — THE DISCRIMINATOR's RED signal).
     _check1 (:wat::core::match r1
               ((:wat::kernel::RecvOutcome::Message resp)
                 (:wat::core::match resp
                   ((:probe::Budget::PutResponse::RequestTooLarge bytes cap)
                     (:wat::core::if (:wat::core::= cap 100)
                       (:wat::core::if (:wat::core::i64::> bytes cap)
                         nil
                         (:wat::kernel::assertion-failed! "RequestTooLarge.bytes must exceed cap" :wat::core::None :wat::core::None))
                       (:wat::kernel::assertion-failed!
                         "STOP-4: cap must name the SURFACE contract (100), never the service's FOO (2048)"
                         :wat::core::None :wat::core::None)))
                   ((:probe::Budget::PutResponse::Ok _ok)
                     (:wat::kernel::assertion-failed! "poison request must be refused, not accepted" :wat::core::None :wat::core::None))
                   ((:probe::Budget::PutResponse::RequestMalformed _mpath _mexpected _mgot)
                     (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
               ((:wat::kernel::RecvOutcome::Lost cause)
                 (:wat::kernel::assertion-failed!
                   (:wat::string::concat
                     "DISCRIMINATOR: the poison request reached the wire (client did not refuse locally) — "
                     (:wat::kernel::LociDiedError/message cause))
                   :wat::core::None :wat::core::None))
               (:wat::kernel::RecvOutcome::Stopped
                 (:wat::kernel::assertion-failed!
                   "DISCRIMINATOR: stopped mid-read — the substrate was asked to stop; the peer was ALIVE"
                   :wat::core::None :wat::core::None))
               (:wat::kernel::RecvOutcome::Closed
                 (:wat::kernel::assertion-failed!
                   "DISCRIMINATOR: the poison request reached the wire and the connection was closed"
                   :wat::core::None :wat::core::None)))
     ;; THE DISCRIMINATOR: the SAME connection completes a normal in-budget request. An
     ;; actually-sent poison frame would have tripped FOO and evicted `c` — this succeeding
     ;; is the "the peer's receiver has nothing pending" proof.
     r2      (:probe::Budget/put c (:probe::Budget::PutRequest :payload "hi"))]
    (:wat::core::match r2
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::Budget::PutResponse::Ok ok) ok)
          ((:probe::Budget::PutResponse::RequestTooLarge _bytes _cap)
            (:wat::kernel::assertion-failed! "unexpected RequestTooLarge on an in-budget follow-up" :wat::core::None :wat::core::None))
          ((:probe::Budget::PutResponse::RequestMalformed _mpath _mexpected _mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed!
          (:wat::string::concat "DISCRIMINATOR: the connection did not survive — "
            (:wat::kernel::LociDiedError/message cause))
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "DISCRIMINATOR: stopped before the follow-up replied — the peer was ALIVE" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "DISCRIMINATOR: the connection did not survive (closed)" :wat::core::None :wat::core::None)))))
