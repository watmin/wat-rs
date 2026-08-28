;; Scratch probe — arc 255 Stone O, THE FULL CENSUS OF `:wat::core::apply`'s DOORS.
;;
;; Written 2026-08-28 after the builder refused this orchestrator's framing:
;;
;;     "(+) => 0 / (+ 1) => 1 / (+ 1 1) => 2 / (+ 1 1 1) => 3 ... right?.....
;;      this '+ needs two args' is baffling"
;;
;; He was right and the framing was wrong. `:wat::core::+` IS variadic-with-identity and
;; fully Clojure-compliant — rows 1-4 below prove it. What the earlier probe quoted was
;; `:wat::i64::+`, a defclause LEAF that is arity-2 BY DESIGN. Naming a leaf as if it were
;; the language made a correct design look like a broken one.
;; `[[feedback_an_adjacent_implementation_is_not_the_subject]]`
;;
;; Running the question properly turned up something bigger: `apply` is broken in THREE
;; distinct ways, and the FIRST one is the whole user-facing arithmetic surface.
;;
;;   DOOR 1  defclause head        -> REFUSED  "expected keyword, got clauses"
;;           29 defclauses exist (22 production): + - * / reduce sort sort-by into
;;           filterv mod quot rem run! reductions nth-spec ... none can be applied.
;;           `dispatch_keyword_head` HAS a clauses arm (runtime.rs:6758 ->
;;           eval_call_to_defclause); `eval_apply`'s Step 6 demands a keyword and stops.
;;   DOOR 2  registered intrinsic, no value_handler -> "unknown function"   (337 of 381)
;;   DOOR 3  registered intrinsic, with value_handler -> works, but wrong arity PANICS (44)
;;   DOOR 4  plain fn / defn                        -> works correctly
;;
;; ONE DEFECT THREE TIMES: a second dispatch path reimplementing the first from a private
;; picture, and each time what it cannot express is "I hold Values, not ASTs."

(:wat::core::defn :probe::o [r <- (:wat::core::Result :wat::core::Value :wat::core::EvalError)]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::core::Ok v)  (:wat::string::concat "ok:" (:wat::edn::write v)))
    ((:wat::core::Err e) (:wat::string::concat "ERR"))))

(:wat::core::defn :probe::p [n <- :wat::core::String f <- :wat::WatAST] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat n " => " (:probe::o (:wat::eval-ast! f)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ── The builder's four. `:wat::core::+` is Clojure-compliant. Nothing to fix here.
     _01 (:probe::p "(+)                        " (:wat::core::quote (:wat::core::+)))
     _02 (:probe::p "(+ 1)                      " (:wat::core::quote (:wat::core::+ 1)))
     _03 (:probe::p "(+ 1 1)                    " (:wat::core::quote (:wat::core::+ 1 1)))
     _04 (:probe::p "(+ 1 1 1)                  " (:wat::core::quote (:wat::core::+ 1 1 1)))
     _05 (:probe::p "(* )  identity             " (:wat::core::quote (:wat::core::*)))
     _06 (:probe::p "(- 5) negation             " (:wat::core::quote (:wat::core::- 5)))
     _07 (:probe::p "(- 10 1 2) left-fold       " (:wat::core::quote (:wat::core::- 10 1 2)))

     ;; ── DOOR 1 — the defclause. THE HEADLINE. Every one of these is ERR today.
     _08 (:probe::p "DOOR1 (apply + [1 2 3])    "
           (:wat::core::quote (:wat::core::apply :wat::core::+ (:wat::core::Vector :wat::core::i64 1 2 3))))
     _09 (:probe::p "DOOR1 (apply * [2 3])      "
           (:wat::core::quote (:wat::core::apply :wat::core::* (:wat::core::Vector :wat::core::i64 2 3))))
     _10 (:probe::p "DOOR1 (apply sort [v])     "
           (:wat::core::quote (:wat::core::apply :wat::core::sort (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::Vector :wat::core::i64 3 1 2)))))

     ;; ── DOOR 2 — registered, works directly, invisible to apply.
     _11 (:probe::p "DOOR2 direct  max-of       " (:wat::core::quote (:wat::f64::max-of 3.0 9.0 41.0)))
     _12 (:probe::p "DOOR2 (apply max-of [...]) "
           (:wat::core::quote (:wat::core::apply :wat::f64::max-of (:wat::core::Vector :wat::core::f64 3.0 9.0 41.0))))

     ;; ── DOOR 3 — registered WITH a value door: reachable, and unguarded.
     ;;    The wrong-arity case PANICS; it lives in its own probe so this one can finish.
     _13 (:probe::p "DOOR3 (apply i64::+ [20 22])"
           (:wat::core::quote (:wat::core::apply :wat::i64::+ (:wat::core::Vector :wat::core::i64 20 22))))

     ;; ── DOOR 4 — a plain registered fn. The one door that is simply correct.
     _14 (:probe::p "DOOR4 (apply count [v])    "
           (:wat::core::quote (:wat::core::apply :wat::core::count (:wat::core::Vector (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::Vector :wat::core::i64 1 2 3)))))]
    nil))
