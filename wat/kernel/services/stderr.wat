;; wat/kernel/services/stderr.wat — Stone 214.8.1b: StdErrService reborn.
;;
;; Arc 214 Slice 8 (TaggedEvent shape): the service collapses to ONE pure fn.
;; The Event defenum, channel typealiases, routing helpers, dispatch, loop,
;; and spawn fn ALL died in this stone — the universe (Rust services home)
;; now owns the fan-in/fan-out and drives the loop.
;;
;; The Rust service peer (src/services/ spawn_write_service_peer) calls:
;;   apply_function(handle, [req_value, writer_value], sym, span)
;; and routes the tagged Rep ack back to the requesting thread's reply channel.
;;
;; Loading order: must load AFTER wat/kernel/channel.wat (uses IOWriter)
;; and AFTER wat/kernel/services/stdin.wat (:wat::kernel::ThreadId).

;; ─── Request record ───────────────────────────────────────────────────────
;;
;; Scalars only — no channel handles (the 254.1 uniform-portability
;; requirement; the 214.8.1b disconfirming gate checks this).
(:wat::core::defstruct :wat::kernel::services::StdErrService::Req
  [thread-id <- :wat::kernel::ThreadId
   line      <- :wat::core::String])

;; ─── Reply record ─────────────────────────────────────────────────────────
;;
;; Ack carries the thread-id so the Rust router knows which reply channel
;; to unblock.
(:wat::core::defstruct :wat::kernel::services::StdErrService::Rep
  [thread-id <- :wat::kernel::ThreadId])

;; ─── Pure handle fn ───────────────────────────────────────────────────────
;;
;; Called by the Rust service loop for every Req: write the line to the
;; IOWriter (appends a newline via writeln), return the tagged Rep.
;; No loop, no select, no routing table, no spawn — the universe drives it.
(:wat::core::defn :wat::kernel::services::StdErrService/handle
  [req <- :wat::kernel::services::StdErrService::Req
   out <- :wat::io::IOWriter]
   -> :wat::kernel::services::StdErrService::Rep
  (:wat::core::let
    [_bytes
      (:wat::io::IOWriter/writeln out
        (:wat::kernel::services::StdErrService::Req/line req))]
    (:wat::kernel::services::StdErrService::Rep/new
      (:wat::kernel::services::StdErrService::Req/thread-id req))))
