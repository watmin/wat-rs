;; probe-a-local-fn-can-be-generic.wat — arc 278.
;;
;; ⛔ THIS PROBE WAS WRONG ON 2026-09-07 AND IS CORRECTED HERE.
;;
;; The first version concluded "a generic local fn dies at EVERY call site" and
;; quoted `parameter #1 expects :wat::core::T; got :wat::core::i64`. That error was
;; MY SYNTAX, not the language: I declared `:- [T]` and then wrote the parameters as
;; `:wat::core::T`. The correct form — `wat/io.wat:40` is the exemplar — declares
;; `:- [T]` and writes the parameters as `:T` / `T`. Everything the old probe
;; claimed rested on a type name that was never the type parameter.
;;
;; ★ THE ACTUAL RULE, measured with the correct syntax:
;;
;;   TOP-LEVEL defn   applied at i64 AND String  ->  BOTH WORK ("i64=3 String=hi!!")
;;   LOCAL fn         applied at i64             ->  works
;;                    then applied at String     ->  "(value head): parameter #1
;;                                                    expects :wat::core::i64;
;;                                                    got :wat::core::String"
;;
;; A local `fn`'s type parameter is instantiated ONCE, at its first use, and then
;; frozen. A top-level `defn` instantiates PER CALL SITE.
;;
;; ★★ WHY IT MATTERS HERE. circuit.wat's worker holds three retry ladders (check
;; :513, mark :574, ack :591). Two speak to a `Seen` peer, one to a `Queue` peer, so
;; one shared combinator must serve two types. That rules out a local `fn` — but NOT
;; a top-level `defn`. The blocker for a top-level defn is different and separate:
;; the worker runs in a process child with `:env-fn "(:wat::program::EmptyEnv)"`
;; (:2086) and cannot see `circuit.wat`'s own defns. It CAN see the stdlib — the file
;; makes 71 calls into `:wat::` from inside service impls.
;;
;; ★★★ So a single shared combinator IS buildable — as a generic defn in `wat/`,
;; frozen into the binary. It is not buildable inside circuit.wat. That is the
;; choice this probe exists to make explicit.
(:wat::core::defn :probe::twice :- [T]
  [x <- :T  f <- [T :-> T]] -> :T
  (f (f x)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [bump  (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ i 1))
     shout (:wat::core::fn [s <- :wat::core::String] -> :wat::core::String (:wat::string::concat s "!"))
     ;; a TOP-LEVEL generic defn, applied at two different types
     n (:probe::twice 1 bump)
     s (:probe::twice "hi" shout)
     ;; a LOCAL generic fn, applied at ONE type — a second type here is a
     ;; type error, which is the whole finding
     once-only (:wat::core::fn :- [T] [x <- :T  f <- [T :-> T]] -> :T (f x))
     k (once-only 10 bump)]
    (:wat::kernel::println
      (:wat::core::format "top: i64={n} String={s} | local (one type only): {k}"
        :n n :s s :k k))))
