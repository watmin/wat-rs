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
     _      (:wat::kernel::send' client 42)
     chosen (:wat::kernel::select'
              (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64> server))]
    (:wat::core::match chosen
      -> :wat::core::i64
      ((:wat::spawn::ServiceEvent::Message _idx req)
        (:wat::core::let [_ (:wat::kernel::send' server (:wat::core::* req 2))
                          reply (:wat::kernel::recv' client)]
          reply))
      (_ (:wat::kernel::assertion-failed!
           "connection primitive: unexpected ServiceEvent variant"
           :wat::core::None :wat::core::None)))))
