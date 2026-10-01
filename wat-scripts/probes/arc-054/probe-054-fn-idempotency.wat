;; probe-054-fn-idempotency.wat — DISCONFIRMING PROBE for (B): the arc-054 idempotency GAP on the fn side.
;;
;; A record declared STANDALONE and again inside a defsurface's :messages is BYTE-EQUIVALENT.
;; arc-054 (types.rs:534) no-ops the TYPE re-registration — but the CONSTRUCTOR fn registration
;; (runtime.rs:1146, bare `contains_key → DuplicateDefine`) does NOT honor arc-054. This is the exact
;; double-registration closure_extract ships (the retained defsurface source-form contains its :messages
;; records, AND the members ship standalone) — minimally reproduced, no bracket/fork.
;;
;; EXPECT (pre-fix, the gap): DuplicateDefine :probe::Echo::EchoRequest (at the constructor).
;; EXPECT (post-(B)-fix):     the re-declaration is a no-op → prints "ok".
;;
;; CLAIM (exit 0): re-declaring :probe::Echo::EchoRequest standalone AND inside the defsurface's
;; :messages is a no-op at BOTH the type AND the constructor-fn registration — no DuplicateDefine.
;; There is no computed value past "did loading this file raise" (the claim is a load-time/startup
;; property, same class as a --check claim), so exit 0 is the whole proof; the println is a sentinel.

(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- wat.type/String])   ;; standalone

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- wat.type/String])   ;; SAME record, re-declared
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- wat.type/String] :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
                                                                                                      :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println "ok"))
