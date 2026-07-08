;; probe-s2-runner-count.wat — RED probe / acceptance target for 259 S2 (pool count on the locus).
;;
;; CLAIM: ThreadOpts/ProcessOpts carry a `runner-count <- :i64` field (default cpu-count), set by
;; the ctor helpers (:wat::spawn::process/runner-count N) / (:wat::spawn::thread/runner-count N).
;; No tier-blind method (YAGNI until remote); the bracket reads the field directly.
;;
;; RED at HEAD (the field + ctors don't exist). GREEN after S2: prints "8 <cpu-count> 4"
;; where <cpu-count> is (:wat::program::cpu-count).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; explicit process pool of 8
     p8    (:wat::spawn::process/runner-count 8)
     n8    (:wat::spawn::ProcessOpts/runner-count p8)
     ;; default process pool = cpu-count
     pdef  (:wat::spawn::process)
     ndef  (:wat::spawn::ProcessOpts/runner-count pdef)
     ;; explicit thread pool of 4
     t4    (:wat::spawn::thread/runner-count 4)
     n4    (:wat::spawn::ThreadOpts/runner-count t4)
     ;; sanity: a bare thread also defaults to cpu-count
     _tdef (:wat::spawn::ThreadOpts/runner-count (:wat::spawn::thread))]
    (:wat::kernel::println
      (:wat::core::string::concat
        (:wat::core::i64::to-string n8)
        (:wat::core::string::concat " "
          (:wat::core::string::concat (:wat::core::i64::to-string ndef)
            (:wat::core::string::concat " " (:wat::core::i64::to-string n4))))))))
