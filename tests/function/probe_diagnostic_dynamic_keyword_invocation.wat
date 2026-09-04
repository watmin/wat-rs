;; tests/function/probe_diagnostic_dynamic_keyword_invocation.wat
;; Arc 232 Stone 232.0 — :wat::core::apply regression guard.
;; Co-located fixture, slurped via startup_beside(file!()).
;; EVAL-fail negative cases (they START UP CLEAN — hence .wat, not .wat.bad, arc 278 C18):
;;   probe_diagnostic_non_keyword.wat (probe 7), probe_diagnostic_non_vector.wat (probe 8).
;; Runtime-fail case: probe 6 (special-form head) — in this fixture, eval expected to Err.

;; Probe 1 — bound substrate-verb keyword dispatched via apply (result 5)
(:wat::core::defn :user::probe-1 [] -> :wat::core::i64
  (:wat::core::let [plus :wat::core::i64::+]
    (:wat::core::apply  plus [2 3])))

;; Probe 2 — runtime-built keyword dispatched via apply (result 5)
(:wat::core::defn :user::probe-2 [] -> :wat::core::i64
  (:wat::core::let [plus (:wat::core::keyword/from-string "wat::core::i64::+")]
    (:wat::core::apply  plus [2 3])))

;; Probe 3 — mangled-namespace user defn via apply (result "hello world")
(:wat::core::defn :ns::greeting [name <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::concat "hello " name))
(:wat::core::defn :user::probe-3 [] -> :wat::core::String
  (:wat::core::let [verb (:wat::core::keyword/from-string "ns::greeting")]
    (:wat::core::apply  verb ["world"])))

;; Probe 4 — leading positional args + tail spread vector (result 10)
(:wat::core::defn :ns::add4
  [a <- :wat::core::i64 b <- :wat::core::i64
   c <- :wat::core::i64 d <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::do (:wat::core::i64::+ (:wat::core::i64::+ a b) (:wat::core::i64::+ c d))))
(:wat::core::defn :user::probe-4 [] -> :wat::core::i64
  (:wat::core::apply  :ns::add4 1 2 [3 4]))

;; Probe 5 — empty tail vector (result "hello")
(:wat::core::defn :ns::greet [] -> :wat::core::String "hello")
(:wat::core::defn :user::probe-5 [] -> :wat::core::String
  (:wat::core::apply  :ns::greet []))

;; Probe 6 — special-form head rejection (runtime error; startup succeeds since keyword is built at runtime)
(:wat::core::defn :user::probe-6-err [] -> :wat::core::String
  (:wat::core::apply 
    (:wat::core::keyword/from-string "wat::core::defn")
    []))
