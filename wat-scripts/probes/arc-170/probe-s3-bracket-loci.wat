;; probe-s3-bracket-loci.wat — ACCEPTANCE target for 259 S3 (loci-agnostic bracket).
;;
;; The SAME work farmed over a thread pool AND a process pool — Ruby's Parallel, loci-agnostic.
;; RED at HEAD: bracket::map takes :wat::spawn::ThreadOpts, so (process/runner-count 2) — a
;; ProcessOpts — is a type error; and the not-shared path isn't built. GREEN after S3.
;; CLAIM: both tr (thread pool) and pr (process pool) == [2 4 6 8 10].

(:wat::core::defn :my::double [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [nums (wat.type/Vector :- [wat.type/i64] 1 2 3 4 5)
     expected (wat.type/Vector :- [wat.type/i64] 2 4 6 8 10)
     ;; a thread pool of 2 (shared memory — runs the fn directly)
     tr (:wat::bracket::map (:wat::spawn::thread::runner-count 2) nums :my::double)
     ;; the SAME call on a process pool of 2 (not-shared — fn-forms the work, ships forms)
     pr (:wat::bracket::map (:wat::spawn::process::runner-count 2) nums :my::double)]
    (:wat::core::do
      (:wat::test::assert-eq tr expected)
      (:wat::test::assert-eq pr expected)
      (:wat::kernel::println
        (:wat::string::concat
          (:wat::edn::write tr)
          (:wat::string::concat " " (:wat::edn::write pr)))))))
