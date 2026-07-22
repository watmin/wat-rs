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

;; a minimal Peer' service whose op-request carries an OPEN Record-surface field (the general
;; capability). `:wat::query::Reason` is a zero-feature Record surface baked into the child (stdlib) —
;; any pure record satisfies it ambiently, exactly as the retired `LogMessage` did for `Log.message`.
;; The surface itself crosses the fork; a CONCRETE user record placed in the field does not.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [payload <- :wat::query::Reason])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo
  :durable   []
  :ephemeral []
  :impls
  [(echo [s req] (:wat::service::Outcome::Reply s (:probe::Echo::EchoResponse::Ok)))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h    (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     echo (:wat::kernel::connect' (:probe::echo'::Handle/addr h))
     _e   (:probe::Echo/echo echo
            (:probe::Echo::EchoRequest :payload (:probe::Note :text "boom")))]
    2))
