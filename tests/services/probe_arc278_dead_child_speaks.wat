;; Co-located fixture for probe_arc278_dead_child_speaks.rs — arc 278: wat NEVER HIDES A FAILURE.
;;
;; Mechanism A at the GENERAL level (arc 278 Stone B re-point): any :satisfies service forked to a
;; PROCESS whose op-request record carries an OPEN Record-surface field will fault if the client sends
;; a value the forked child cannot decode. Here a tiny :probe::echo' service is forked; its
;; EchoRequest.payload is typed as the open surface :wat::query::Reason (zero features — any pure
;; record satisfies it ambiently). The parent sends a payload holding :probe::Note, a PARENT-ONLY record
;; absent from the forked child's baked type registry (top-level user defrecords do NOT cross a fork;
;; only the surface's :messages do). The child faults decoding it —
;;   "poll' (process tier): client message decode failed: ... unknown tag #probe/Note (body shape:
;;    map); no matching struct or enum in the type registry"
;; — and, at HEAD, that real reason was written to an ALREADY-CLOSED err pipe (EPIPE) and LOST; the
;; caller's `echo` recv' raised a MUTE "recv failed: peer closed / channel disconnected". Mechanism A:
;; poll' returns ServiceEvent::Malformed carrying the cause; the serve loop replies Reply::Failed{cause}
;; and keeps serving; recv' surfaces the raise CARRYING THE REASON. The .rs harness asserts the raised
;; error carries it. (Stone B removed telemetry as one incidental trigger — Log.message is now an
;; opaque String, so a Log CANNOT carry a foreign-typed message; the LAW is kept live at the general
;; open-surface-field level, where it belongs.)

;; the parent-only payload record — NOT baked into the forked child's registry.
(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

;; EXACT DATA: :user::compute returns a STRUCTURED :probe::Outcome — the RecvOutcome variant that
;; matched + a deterministic `reason-names-decode-failure?` bool computed IN-WAT (the per-run-variable
;; Failure location never leaves wat; only its boolean RESULT crosses to the .rs golden). The .rs
;; asserts the golden #probe.Outcome/Lost [true] exactly — mirroring probe_arc278_recv_outcome_wall.
;; "wat stdio is edn — assert the structure exactly" (builder; R55 REVOLVTIONE, NVLLA LARVA).
(:wat::core::defenum :probe::Outcome :wat::enum::Pure
  :Message []                                                ;; matched ::Message (.rs asserts NEVER)
  :Lost    [reason-names-decode-failure? <- :wat::core::bool] ;; matched ::Lost — true iff the cause names the decode failure (the LAW: the reason is carried)
  :Closed  []                                                ;; matched ::Closed (the mute we killed — .rs asserts NEVER)
  ;; arc 278 #73 — a stop is NOT a close, so it does not borrow ::Closed's label. The golden is
  ;; `#probe.Outcome/Lost [true]`, so adding a variant costs the passing path nothing; what it buys
  ;; is that if a stop ever DID fire here, the mismatch names a stop instead of pointing the next
  ;; reader at the channel layer — which is the precise false trail this whole stone removes.
  :Stopped [])                                               ;; .rs asserts NEVER (nothing here stops mid-read)

;; a minimal Peer' service whose op-request carries an OPEN Record-surface field (the general
;; capability). `:wat::query::Reason` is a zero-feature Record surface baked into the child (stdlib) —
;; any pure record satisfies it ambiently, exactly as the retired `LogMessage` did for `Log.message`.
;; The surface itself crosses the fork; a CONCRETE user record placed in the field does not.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [payload <- :wat::query::Reason])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo
  :durable   []
  :ephemeral []
  :impls
  [(echo [s ctx req] (:wat::service::Outcome::Reply s (:probe::Echo::EchoResponse::Ok)))])

;; arc 278 recv'-wall: a peer-read yields a MATCHABLE RecvOutcome — NEVER a raise (a raise unwinds
;; PAST the reader, which is the mask the wall kills). The client-method (:probe::Echo/echo) SCRUBS
;; the cause into a reason-free 500; the real decode reason travels via raw recv'. We send the op raw
;; and MATCH the outcome, RETURNING the child's rich Reply::Failed cause as a VALUE ("unknown tag
;; #probe/Note ... no matching struct or enum ..."). The .rs asserts is_ok + the returned reason —
;; mirroring the canonical gate probe_arc278_recv_outcome_wall.
(:wat::core::defn :user::compute [] -> :probe::Outcome
  (:wat::core::let
    [h    (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     echo (:wat::core::match (:wat::kernel::connect (:probe::echo::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _s   (:wat::kernel::send echo
            (:probe::Echo::Op::Echo
              (:probe::Echo::EchoRequest :payload (:probe::Note :text "boom"))))]
    (:wat::core::match (:wat::kernel::recv echo)
      ((:wat::kernel::RecvOutcome::Message _m) (:probe::Outcome::Message))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:probe::Outcome::Lost (:wat::string::contains? (:wat::kernel::LociDiedError/message cause) "no matching struct or enum")))
      ;; arc 278 #73 — reported as ITSELF. This test never stops mid-read, so the arm is
      ;; unreachable today; naming it honestly is what keeps it unreachable-and-legible rather
      ;; than unreachable-and-mislabelled.
      (:wat::kernel::RecvOutcome::Stopped (:probe::Outcome::Stopped))
      (:wat::kernel::RecvOutcome::Closed (:probe::Outcome::Closed)))))
