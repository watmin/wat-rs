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

;; ─── Default frame cap ────────────────────────────────────────────────────
;;
;; Arc 255 escape-hatch — single source of truth for the default readln cap.
;; The `readln` macro injects this as the cap arg when no :max-buffer-bytes
;; kwarg is supplied; `readln'` always takes an explicit max (no Rust default).
;; 512 × 1024 = 524 288 bytes (512 KiB) — mirrors DEFAULT_MAX_FRAME_BYTES in
;; src/edn_shim.rs (kept for the Receiver/from-pipe channel path which has
;; no macro layer).
(:wat::core::def :wat::kernel::MAX-READLN-BYTES
  (:wat::core::i64::* 512 1024))

;; ─── Request record ───────────────────────────────────────────────────────
;;
;; Arc 255 escape-hatch: added `max-buffer-bytes` field carrying the caller's
;; optional frame-size cap (forwarded from readln' → Req → handle → read-frame).
;; Scalars only — no channel handles (the 254.1 uniform-portability
;; requirement; the 214.8.2 disconfirming gate checks this).
(:wat::core::defstruct :wat::kernel::services::StdInService::Req
  [thread-id        <- :wat::kernel::ThreadId
   max-buffer-bytes <- :wat::core::i64])

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
;; Arc 255 escape-hatch: reads max-buffer-bytes from the Req and passes it to
;; read-frame so the caller's cap propagates to the frame accumulator.
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
  (:wat::core::match (:wat::io::IOReader/read-frame in (:wat::kernel::services::StdInService::Req/max-buffer-bytes req))
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

;; ─── readln macro ─────────────────────────────────────────────────────────
;;
;; Arc 255 escape-hatch. `readln` is the user-facing defmacro; `readln'`
;; (the prime) is the kernel-restricted positional primitive they expand to.
;;
;; Per the kwargs-is-always-a-macro doctrine: the exposed surface is kwargs
;; (readln :max-buffer-bytes N -> :T), the lean prime is positional.
;;
;; Shape:
;;   (readln -> :T)                        → (readln' :wat::kernel::MAX-READLN-BYTES -> :T)
;;   (readln :max-buffer-bytes N -> :T)    → (readln' N -> :T)
;;
;; The `-> :T` annotation is forwarded intact so the checker can infer
;; readln's polymorphic return type from the call-site arrow (see
;; infer_kernel_readln_prime in src/check.rs).
;;
;; Arg parse: if the first element of `args` is the `:max-buffer-bytes`
;; keyword (checked via ast-kind + ast-name), consume it + the next element
;; (N) and emit `(readln' N <rest>)`; otherwise emit `(readln' <args>)`.
;;
;; The program-body path (no leading quasiquote) runs in the fenced macro
;; evaluator; `args` is bound as a Value::Vec of Value::wat__WatAST nodes.
;; `get` returns Option<Value::wat__WatAST>; `Option/expect` unwraps it.
;;
;; `readln'` is a Rust intrinsic (always available at expand time — no
;; load-order dependency on any wat file). This macro therefore has no
;; load-order constraint and lives in stdin.wat as the natural home.
(:wat::core::defmacro :wat::kernel::readln
  [& args <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::let
    [n-args    (:wat::core::length args)
     ;; Check whether the first form is the :max-buffer-bytes keyword.
     ;; Use get (safe on empty vector) and compare by ast-kind + ast-name.
     first-opt (:wat::core::get args 0)]
    (:wat::core::if
      ;; Is there a first arg AND is it a keyword?
      (:wat::core::if
        (:wat::core::= n-args 0)
        -> :wat::core::bool
        false
        (:wat::core::= (:wat::core::ast-kind
                         (:wat::core::Option/expect  
                           first-opt
                           "readln macro: internal error — first-opt is None but n-args > 0"))
                       "keyword"))
      -> :wat::WatAST
      ;; First arg is a keyword. Check if it's :max-buffer-bytes.
      (:wat::core::let
        [first-node (:wat::core::Option/expect  
                       first-opt
                       "readln macro: internal error — first-node")]
        (:wat::core::if
          (:wat::core::= (:wat::core::ast-name first-node) ":max-buffer-bytes")
          -> :wat::WatAST
          ;; :max-buffer-bytes N -> :T  →  (readln' N -> :T)
          (:wat::core::let
            [cap-expr (:wat::core::Option/expect  
                          (:wat::core::get args 1)
                          "readln: :max-buffer-bytes requires a value (e.g. :max-buffer-bytes (* 2 1024 1024))")
             rest     (:wat::core::rest (:wat::core::rest args))]
            `(:wat::kernel::readln' ~cap-expr ~@rest))
          ;; Unknown keyword as first arg — pass through to readln' for a clean error.
          `(:wat::kernel::readln' ~@args)))
      ;; First arg is not a keyword (or args is empty) — plain form:
      ;; (readln -> :T) → (readln' :wat::kernel::MAX-READLN-BYTES -> :T).
      ;; The macro injects the default cap so readln' always gets an explicit max.
      `(:wat::kernel::readln' :wat::kernel::MAX-READLN-BYTES ~@args))))
