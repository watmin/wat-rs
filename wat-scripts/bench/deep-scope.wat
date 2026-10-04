;; wat-scripts/bench/deep-scope.wat -- deep lexical nesting, to measure
;; Environment::lookup's PARENT WALK in isolation (docs/arc/2026/10/name-resolution/BRIEF-2.md,
;; row P2). Neither conj-build.wat nor call-heavy.wat nests scopes deeply (R2's named gap, part
;; 1 SCORE.md) -- every lookup there is a 0-or-1-hop hit. This file nests 64 separate `let`
;; forms, each binding EXACTLY ONE new name, so there are 64 distinct `Environment` objects
;; chained parent-to-child -- looking up the OUTERMOST name (`s0`) from the innermost scope
;; must walk all 64 links; looking up the INNERMOST name (`s63`) is a 0-hop hit, the control.
;;
;; A closure (`step`) created inside the innermost scope closes over the whole 64-deep chain;
;; `foldl` (a Rust-implemented builtin iterator, not wat-level recursion) applies it once per
;; element of `(range 0 1000000)`, so the closure's body -- and both lookups -- run 1,000,000
;; times. Prints one checksum.
;;
;; Usage (no stdin; stdout = an EDN :perf::DeepScopeResult map):
;;   target/release/wat ./wat-scripts/bench/deep-scope.wat

(:wat::core::defrecord :perf::DeepScopeResult
  [checksum <- :wat::core::i64
   ns       <- :wat::core::i64])

(:wat::core::defn :perf::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [s0 1]
    (:wat::core::let [s1 (:wat::i64::+ s0 1)]
      (:wat::core::let [s2 (:wat::i64::+ s1 1)]
        (:wat::core::let [s3 (:wat::i64::+ s2 1)]
          (:wat::core::let [s4 (:wat::i64::+ s3 1)]
            (:wat::core::let [s5 (:wat::i64::+ s4 1)]
              (:wat::core::let [s6 (:wat::i64::+ s5 1)]
                (:wat::core::let [s7 (:wat::i64::+ s6 1)]
                  (:wat::core::let [s8 (:wat::i64::+ s7 1)]
                    (:wat::core::let [s9 (:wat::i64::+ s8 1)]
                      (:wat::core::let [s10 (:wat::i64::+ s9 1)]
                        (:wat::core::let [s11 (:wat::i64::+ s10 1)]
                          (:wat::core::let [s12 (:wat::i64::+ s11 1)]
                            (:wat::core::let [s13 (:wat::i64::+ s12 1)]
                              (:wat::core::let [s14 (:wat::i64::+ s13 1)]
                                (:wat::core::let [s15 (:wat::i64::+ s14 1)]
                                  (:wat::core::let [s16 (:wat::i64::+ s15 1)]
                                    (:wat::core::let [s17 (:wat::i64::+ s16 1)]
                                      (:wat::core::let [s18 (:wat::i64::+ s17 1)]
                                        (:wat::core::let [s19 (:wat::i64::+ s18 1)]
                                          (:wat::core::let [s20 (:wat::i64::+ s19 1)]
                                            (:wat::core::let [s21 (:wat::i64::+ s20 1)]
                                              (:wat::core::let [s22 (:wat::i64::+ s21 1)]
                                                (:wat::core::let [s23 (:wat::i64::+ s22 1)]
                                                  (:wat::core::let [s24 (:wat::i64::+ s23 1)]
                                                    (:wat::core::let [s25 (:wat::i64::+ s24 1)]
                                                      (:wat::core::let [s26 (:wat::i64::+ s25 1)]
                                                        (:wat::core::let [s27 (:wat::i64::+ s26 1)]
                                                          (:wat::core::let [s28 (:wat::i64::+ s27 1)]
                                                            (:wat::core::let [s29 (:wat::i64::+ s28 1)]
                                                              (:wat::core::let [s30 (:wat::i64::+ s29 1)]
                                                                (:wat::core::let [s31 (:wat::i64::+ s30 1)]
                                                                  (:wat::core::let [s32 (:wat::i64::+ s31 1)]
                                                                    (:wat::core::let [s33 (:wat::i64::+ s32 1)]
                                                                      (:wat::core::let [s34 (:wat::i64::+ s33 1)]
                                                                        (:wat::core::let [s35 (:wat::i64::+ s34 1)]
                                                                          (:wat::core::let [s36 (:wat::i64::+ s35 1)]
                                                                            (:wat::core::let [s37 (:wat::i64::+ s36 1)]
                                                                              (:wat::core::let [s38 (:wat::i64::+ s37 1)]
                                                                                (:wat::core::let [s39 (:wat::i64::+ s38 1)]
                                                                                  (:wat::core::let [s40 (:wat::i64::+ s39 1)]
                                                                                    (:wat::core::let [s41 (:wat::i64::+ s40 1)]
                                                                                      (:wat::core::let [s42 (:wat::i64::+ s41 1)]
                                                                                        (:wat::core::let [s43 (:wat::i64::+ s42 1)]
                                                                                          (:wat::core::let [s44 (:wat::i64::+ s43 1)]
                                                                                            (:wat::core::let [s45 (:wat::i64::+ s44 1)]
                                                                                              (:wat::core::let [s46 (:wat::i64::+ s45 1)]
                                                                                                (:wat::core::let [s47 (:wat::i64::+ s46 1)]
                                                                                                  (:wat::core::let [s48 (:wat::i64::+ s47 1)]
                                                                                                    (:wat::core::let [s49 (:wat::i64::+ s48 1)]
                                                                                                      (:wat::core::let [s50 (:wat::i64::+ s49 1)]
                                                                                                        (:wat::core::let [s51 (:wat::i64::+ s50 1)]
                                                                                                          (:wat::core::let [s52 (:wat::i64::+ s51 1)]
                                                                                                            (:wat::core::let [s53 (:wat::i64::+ s52 1)]
                                                                                                              (:wat::core::let [s54 (:wat::i64::+ s53 1)]
                                                                                                                (:wat::core::let [s55 (:wat::i64::+ s54 1)]
                                                                                                                  (:wat::core::let [s56 (:wat::i64::+ s55 1)]
                                                                                                                    (:wat::core::let [s57 (:wat::i64::+ s56 1)]
                                                                                                                      (:wat::core::let [s58 (:wat::i64::+ s57 1)]
                                                                                                                        (:wat::core::let [s59 (:wat::i64::+ s58 1)]
                                                                                                                          (:wat::core::let [s60 (:wat::i64::+ s59 1)]
                                                                                                                            (:wat::core::let [s61 (:wat::i64::+ s60 1)]
                                                                                                                              (:wat::core::let [s62 (:wat::i64::+ s61 1)]
                                                                                                                                (:wat::core::let [s63 (:wat::i64::+ s62 1)]
                                                                                                                                  (:wat::core::let [step (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
                                                                                                                                    (:wat::i64::+ acc (:wat::i64::+ s0 s63)))
                                                                                                                                    t0     (:wat::time::now)
                                                                                                                                    total  (:wat::core::foldl step 0 (:wat::core::range 0 1000000))
                                                                                                                                    t1     (:wat::time::now)
                                                                                                                                    ns     (:perf::ns-between t0 t1)]
                                                                                                                                    (:wat::kernel::println (:perf::DeepScopeResult :checksum total :ns ns)))
)))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))
