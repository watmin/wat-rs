;; tests/process/wat_arc208_process_io_result.wat — co-located fixture for
;; wat_arc208_process_io_result.rs (T2-T5). Each entry fn spawns its own child
;; process(es) internally (Rust no longer builds the spawn-process AST by hand
;; and binds a live Process value into an Environment) — the process-boundary
;; mechanics move into the fixture verbatim; the Rust driver just calls the
;; named entry via call_beside and inspects the returned typed value.
;;
;; T1 (CheckEnv type-scheme registration) and T6/T7 (walker MUST-REJECT
;; fixtures) carry no wat here — T1 has no wat-under-test; T6/T7 already live
;; in their own co-located wat_arc208_process_io_result_bad_{println,readln}.wat.

;; ─── T2 — println then readln+drain against the SAME live echo-server peer ──
;; Mirrors the original two-pass Rust-built AST exactly: pass1 wraps
;; Process/println's Result via match (the only legal non-expect position per
;; the comm-position walker); pass2 wraps Process/readln's Result the same way,
;; then drains+joins. Both raw Results travel back to Rust in a Tuple so the
;; existing unwrap_ok helper stays unchanged.
(:wat::core::defn :user::t2-println-then-readln [] ->
  :(wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::LociDiedError>>,wat::core::Result<wat::core::String,wat::core::Vector<wat::kernel::LociDiedError>>)
  (:wat::core::let
    [server (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let
                    [line (:wat::kernel::readln )]
                    (:wat::kernel::println line)))))
     pass1 (:wat::core::let
             [rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
              tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
              peer (:wat::kernel::ProcessPeer :rx rx :tx tx)]
             (:wat::core::match (:wat::kernel::Process/println peer "arc208-ok")
               
               ((:wat::core::Ok _)  (:wat::core::Ok ()))
               ((:wat::core::Err e) (:wat::core::Err e))))
     pass2 (:wat::core::let
             [rx    (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
              tx    (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
              peer  (:wat::kernel::ProcessPeer :rx rx :tx tx)
              reply (:wat::core::match (:wat::kernel::Process/readln peer)
                      
                      ((:wat::core::Ok v)  (:wat::core::Ok v))
                      ((:wat::core::Err e) (:wat::core::Err e)))
              _done (:wat::kernel::Process/drain-and-join server)]
             reply)]
    (:wat::core::Tuple pass1 pass2)))

;; ─── T3 — Process/println on a peer whose subprocess already exited ─────────
;; Faithful transplant: spawns+drains a first (unused) immediate-exit server —
;; a vestigial artifact of the original Rust-built version, preserved exactly —
;; then spawns a SECOND immediate-exit server, drains it too, and attempts
;; println on the now-dead peer.
(:wat::core::defn :user::t3-println-dead-peer [] -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::LociDiedError>>
  (:wat::core::let
    [server1 (:wat::kernel::spawn-process (:wat::core::forms (:wat::core::defn :user::main [] -> :wat::core::nil nil)))
     _djoin1 (:wat::kernel::Process/drain-and-join server1)
     server2 (:wat::kernel::spawn-process (:wat::core::forms (:wat::core::defn :user::main [] -> :wat::core::nil nil)))
     rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server2))
     tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server2))
     peer (:wat::kernel::ProcessPeer :rx rx :tx tx)
     _    (:wat::kernel::Process/drain-and-join server2)]
    (:wat::core::match (:wat::kernel::Process/println peer "should-fail")
      
      ((:wat::core::Ok _)  (:wat::core::Ok ()))
      ((:wat::core::Err e) (:wat::core::Err e)))))

;; ─── T4 (and T5, same body) — Process/readln on a peer whose subprocess ─────
;; already exited. T5 reuses this fn: T4's and T5's original Rust-built ASTs
;; were byte-identical, so one fixture entry serves both probes.
(:wat::core::defn :user::t4-readln-dead-peer [] -> :wat::core::Result<wat::core::String,wat::core::Vector<wat::kernel::LociDiedError>>
  (:wat::core::let
    [server (:wat::kernel::spawn-process (:wat::core::forms (:wat::core::defn :user::main [] -> :wat::core::nil nil)))
     rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
     tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
     peer (:wat::kernel::ProcessPeer :rx rx :tx tx)
     _    (:wat::kernel::Process/drain-and-join server)]
    (:wat::core::match (:wat::kernel::Process/readln peer)
      
      ((:wat::core::Ok v)  (:wat::core::Ok v))
      ((:wat::core::Err e) (:wat::core::Err e)))))
