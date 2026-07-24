;; tests/channel/probe_arc209_connection_primitive.wat — co-located fixture, slurped via
;; `call_beside(file!(), ":user::compute")` (just-eval rubric, docs/CONVENTIONS.md § Test
;; idioms: a VALUE/TYPE claim — mints a connected `Peer'` pair without spawning, round-trips
;; a request/response over it via `select'`, returns the i64 reply). No process boundary
;; participates, so no crash/stdio/IPC surface is under test here.
;;
;; Migrated verbatim off the inline `eval_expr` driver string (arc 278 no-inlined-wat
;; crusade) — same expression, same Stone 259 ServiceEvent shape, byte-identical.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair   (:wat::kernel::peer-pair' :wat::core::i64 :wat::core::i64)
     server (:wat::core::first pair)
     client (:wat::core::second pair)
     _      (:wat::core::match (:wat::kernel::send' client 42) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     chosen (:wat::kernel::select'
              (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64> server))]
    (:wat::core::match chosen
      
      ((:wat::spawn::ServiceEvent::Message _idx req)
        (:wat::core::let [_ (:wat::core::match (:wat::kernel::send' server (:wat::core::* req 2)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                          ;; arc 278 recv'-outcome wall — recv' returns a matchable
                          ;; RecvOutcome<i64>. OWNER role (the test is the final caller):
                          ;; on ::Lost surface the cause loudly (eprintln, divergent);
                          ;; ::Closed likewise. ::Message m flows out exactly as reply did.
                          r     (:wat::kernel::recv' client)
                          reply (:wat::core::match r
                                  ((:wat::kernel::RecvOutcome::Message m) m)
                                  ((:wat::kernel::RecvOutcome::Lost cause)
                                    (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                                  (:wat::kernel::RecvOutcome::Closed
                                    (:wat::kernel::assertion-failed! "recv': client closed before replying" :wat::core::None :wat::core::None)))]
          reply))
      (_ (:wat::kernel::assertion-failed!
           "connection primitive: unexpected ServiceEvent variant"
           :wat::core::None :wat::core::None)))))
