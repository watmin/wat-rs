;; wat/kernel/services/stdin.wat — Stone 214.8.2: StdInService reborn.
;;
;; Arc 214 Slice 8 (TaggedEvent shape): the service collapses to ONE pure fn.
;; The Event defenum, channel typealiases, routing vector, handle-add/remove,
;; dispatch, driver loop, and spawn fn ALL died in this stone — the universe
;; (Rust src/services/ spawn_service_peer) now owns the fan-in/fan-out and
;; drives the loop.
;;
;; The Rust service peer (src/services/ spawn_service_peer) calls:
;;   apply_function(handle, [req_value, reader_value], sym, span)
;; and routes the tagged Rep line back to the requesting thread's reply channel.
;;
;; Loading order: must load AFTER wat/kernel/channel.wat (uses IOReader);
;; loads FIRST of the three services (it declares :wat::kernel::ThreadId,
;; which stdout.wat and stderr.wat consume).

;; ─── ThreadId ─────────────────────────────────────────────────────────────
;;
;; Mirrors `pub type ThreadId = i64` from src/thread_io.rs. Declared here
;; because stdin.wat loads first of the trio; the whole trio's Req/Rep
;; records consume it. (8.2 scoring catch: the rebirth's first draft DROPPED
;; this typealias and every gate stayed green — the checker is LENIENT on
;; undeclared field-type keywords; see the #[ignore]'d
;; probe_diag_typealias_leniency nursery probe, banked for arc 255.)
;; rune:exigere(attested-arc) — arc 255 (docs/arc/2026/06/255-builtin-registry/DESIGN.md); the leniency probe un-ignores when 255 makes undeclared type keywords check errors

(:wat::core::typealias :wat::kernel::ThreadId
  :wat::core::i64)

;; ─── Request record ───────────────────────────────────────────────────────
;;
;; Scalars only — no channel handles (the 254.1 uniform-portability
;; requirement; the 214.8.2 disconfirming gate checks this).
(:wat::core::defstruct :wat::kernel::services::StdInService::Req
  [thread-id <- :wat::kernel::ThreadId])

;; ─── Reply record ─────────────────────────────────────────────────────────
;;
;; Carries the thread-id (for routing) and the line read from fd 0.
(:wat::core::defstruct :wat::kernel::services::StdInService::Rep
  [thread-id <- :wat::kernel::ThreadId
   line      <- :wat::core::String])

;; ─── Pure handle fn ───────────────────────────────────────────────────────
;;
;; Called by the Rust service loop for every Req: read the next line from the
;; IOReader (blocking on fd 0), return the tagged Rep.
;; No loop, no select, no routing table, no spawn — the universe drives it.
;;
;; EOF arm: EOF on fd 0 is a lock-step contract violation and MUST panic the
;; service via assertion-failed! (which calls std::panic::panic_any). The
;; service thread dies; reply-txs in the reply registry drop; every blocked
;; caller's reply_rx.recv() returns Err → ChannelDisconnected. Cascades
;; cleanly down forked child trees (each child's StdInService panics the
;; same way when its parent's pipe closes). The Rust loop needs NO
;; catch_unwind and NO EOF arm — the pre-proven composition fact: a panicking
;; handle kills the loop thread through apply_function BY DESIGN.
(:wat::core::defn :wat::kernel::services::StdInService/handle
  [req <- :wat::kernel::services::StdInService::Req
   in  <- :wat::io::IOReader]
   -> :wat::kernel::services::StdInService::Rep
  (:wat::core::match (:wat::io::IOReader/read-line in)
      -> :wat::kernel::services::StdInService::Rep
    ((:wat::core::Some line)
      (:wat::kernel::services::StdInService::Rep/new
        (:wat::kernel::services::StdInService::Req/thread-id req)
        line))
    (:wat::core::None
      ;; EOF on fd 0: client (parent process / pipe writer) disconnected.
      ;; Per lock-step doctrine + feedback_silent_disconnect_hang, this
      ;; is a contract violation and MUST surface as a panic — not be
      ;; silently swallowed (the old `()` no-op spun the service
      ;; thread forever on EOF, leaving callers' recv blocked).
      ;; assertion-failed! invokes std::panic::panic_any: the service
      ;; thread dies, reply-txs in the reply registry drop, every
      ;; blocked caller's readln returns ChannelDisconnected.
      ;; Cascades cleanly down forked child trees (each child's
      ;; StdInService panics the same way when its parent's pipe closes).
      (:wat::kernel::assertion-failed!
        "StdInService: EOF on fd 0 — client (parent process or pipe writer) disconnected. Lock-step contract violation; process must die."
        :wat::core::None :wat::core::None))))
