;; probe-118B8-dorun-effect-count.wat — stone 118.B8 acceptance instrument #2: does `dorun`'s new
;; self-recursive `next`-walk body still force every element (side effects run, in order, exactly
;; once each)? Adapted from `probe-118B-memo-state-detector.wat`'s discipline (see that file's
;; header for why "prints 5" alone is not self-explanatory): `f` prints one line per invocation,
;; `dorun` (not `into`) drains the mapped stream, and the harness counts printed lines.
;;
;; RUN: ./target/release/wat wat-scripts/scratch-pad/probe-118B8-dorun-effect-count.wat | wc -l
;; PASS = exactly 5 (n elements in -> n forces out; dorun keeps nothing but must force everything).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [v  (:wat::core::range 0 5)
     f  (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::do
            (:wat::kernel::println "FORCED")
            x))]
    (:wat::core::dorun (:wat::core::map f v))))
