;; Scratch probe — arc 255 Stone O-iv-a, acceptance row 1.
;;
;; STOP-2's positive form: prove BOTH branches of the new gate separately, and read the
;; EvalError's `kind` field (not just message prose — a message test cannot tell these
;; apart if someone later edits the text):
;;
;;   (apply :wat::f64::max-of […])        -> registered-but-unreachable, the NEW kind
;;   (apply :wat::not::a::real::verb […]) -> still UnknownFunction, the OLD kind
;;
;; `RuntimeErrorKind::NotValueDispatchable` has no dedicated `kind` string in
;; `runtime_error_to_eval_error_value` (src/runtime.rs) — it falls through that match's
;; wildcard arm to "runtime-error". That is still a DIFFERENT string from
;; "unknown-function", which is exactly what this probe needs to distinguish the two
;; branches without looking at message text.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [registered (:wat::eval-ast! (:wat::core::quote
                  (:wat::core::apply :wat::f64::max-of
                    (:wat::core::Vector :wat::core::f64 3.0 9.0 41.0))))
     unknown    (:wat::eval-ast! (:wat::core::quote
                  (:wat::core::apply :wat::not::a::real::verb
                    (:wat::core::Vector :wat::core::i64 1))))]
    (:wat::core::do
      (:wat::core::match registered
        ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "UNEXPECTED ok: " (:wat::edn::write v))))
        ((:wat::core::Err e)
          (:wat::kernel::println (:wat::string::concat "registered-but-unreachable  kind="
            (:wat::core::EvalError/kind e) "  message=" (:wat::core::EvalError/message e)))))
      (:wat::core::match unknown
        ((:wat::core::Ok v) (:wat::kernel::println (:wat::string::concat "UNEXPECTED ok: " (:wat::edn::write v))))
        ((:wat::core::Err e)
          (:wat::kernel::println (:wat::string::concat "genuinely-unknown           kind="
            (:wat::core::EvalError/kind e) "  message=" (:wat::core::EvalError/message e))))))))
