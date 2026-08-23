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

(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])   ;; standalone

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])   ;; SAME record, re-declared
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "ok"))
