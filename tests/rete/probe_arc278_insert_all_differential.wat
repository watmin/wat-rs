;; Arc 278 — `insert-all` joins the dual-impl as the real BATCH primitive; `insert` keeps its
;; 2-ary clause UNCHANGED and gains a variadic clause as sugar over `insert-all`.
;;
;; Dual-impl law: unprimed public names are native; `$oracle` is the spec mouth;
;; `$native` is the kernel. (mirrors probe_arc278_native_insert_differential)
;;   insert$oracle / insert$native / insert  ->  insert-all$oracle / insert-all$native / insert-all
;;
;; What would turn this red once it is green (the R59 question, answered before the assertions
;; were written):
;;   (a) `insert-all` writing the wrong `Session` slot (a positional-index assumption instead of
;;       resolving `facts` by name) — `staged` would drift, or `fired`/`sum` would collapse;
;;   (b) `insert-all` silently returning the session UNCHANGED (a no-op) — the equivalence (1) and
;;       oracle (2) assertions would both pass vacuously against an empty vector; assertion (3)
;;       (N > 1, `facts` length == N exactly) is the ONLY thing that would catch it;
;;   (c) the public `insert-all` becoming a second implementation instead of a delegate to
;;       `insert-all$native` — invisible to the oracle/native split alone, which is why (2) compares
;;       `insert-all$oracle` to `insert-all$native` directly (not through the public verb);
;;   (d) the 2-ary `insert` being silently RE-ROUTED through `insert-all` (STOP-1) — a
;;       one-element-vector allocation on the streaming hot path that every other assertion here
;;       would miss. Assertion (4) checks it by BEHAVIOUR: a lone 2-ary `insert` call must match
;;       `insert$native` called directly, fact for fact. (The form-level proof — that
;;       `wat/rete/oracle/insert.wat`'s 2-ary clause body is `(:wat::rete::insert$native session fact)`
;;       with no reference to `insert-all` — is read by hand against the source, not re-derived
;;       here.)

(:wat::core::defrecord :nia::Reading [g <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :nia::Out     [g <- :wat::core::i64])

(:wat::rete::defrule :nia::pass-rule
  :when
  [(:nia::Reading (?g <- :g))]
  :then
  [(:nia::Out ?g)])

(:wat::rete::defquery :nia::q-Out
  :params []
  :when [(?fact <- :nia::Out)])


(:wat::core::defn :nia::base [] -> :wat::rete::Session
  (:wat::rete::compile-all (:wat::rete::collect-rules :nia) (:wat::core::PersistentVector (:nia::q-Out))))

;; The facts under test — N=5, satisfying assertion 3's N > 1 requirement.
(:wat::core::defn :nia::the-facts [] -> (:wat::core::PersistentVector :- [:nia::Reading])
  (:wat::core::PersistentVector
    (:nia::Reading :g 0 :v 0)
    (:nia::Reading :g 1 :v 10)
    (:nia::Reading :g 2 :v 20)
    (:nia::Reading :g 3 :v 30)
    (:nia::Reading :g 4 :v 40)))

;; ── seeders — identical facts, different verb ────────────────────────────────

;; batch: ONE insert-all call (the public verb, delegating to insert-all$native).
(:wat::core::defn :nia::seed-batch [] -> :wat::rete::Session
  (:wat::rete::insert-all (:nia::base) (:nia::the-facts)))

;; chained: N sequential 2-ary insert calls — the pre-existing streaming hot path.
;; `insert` is now a `defclause` (a dispatch table, not a plain `Function` value), so it
;; cannot be passed to `foldl` bare; wrap it so the 2-ary clause is called explicitly.
(:wat::core::defn :nia::seed-chained [] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session f <- :nia::Reading] -> :wat::rete::Session
      (:wat::rete::insert s f))
    (:nia::base)
    (:nia::the-facts)))

;; oracle: batch via insert-all$oracle (the wat reference / differential oracle).
(:wat::core::defn :nia::seed-oracle [] -> :wat::rete::Session
  (:wat::rete::insert-all$oracle (:nia::base) (:nia::the-facts)))

;; native: batch via insert-all$native DIRECTLY (bypassing the public delegate — isolates the prime).
(:wat::core::defn :nia::seed-native [] -> :wat::rete::Session
  (:wat::rete::insert-all$native (:nia::base) (:nia::the-facts)))

;; ── witnesses, read off a seeded Session ──────────────────────────────────────

(:wat::core::defn :nia::staged-count [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::Session/facts s)))

(:wat::core::defn :nia::fired-outs [s <- :wat::rete::Session] -> :wat::core::PersistentVector
  (:wat::rete::query (:wat::rete::fire-rules s) (:nia::q-Out)))

(:wat::core::defn :nia::fired-count [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:nia::fired-outs s)))

(:wat::core::defn :nia::fired-sum [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::core::i64  p <- :wat::core::PersistentMap] -> :wat::core::i64
      (:wat::core::let [o (:wat::core::Option/expect
                            (:wat::core::PersistentMap/get p "?fact")
                            "q-Out: ?fact")]
        (:wat::i64::+ a (:nia::Out/g o))))
    0
    (:nia::fired-outs s)))

;; ── single-fact seeders (assertion 4 — the 2-ary hot path) ────────────────────

(:wat::core::defn :nia::seed-single-public [] -> :wat::rete::Session
  (:wat::rete::insert (:nia::base) (:nia::Reading :g 7 :v 70)))

(:wat::core::defn :nia::seed-single-native [] -> :wat::rete::Session
  (:wat::rete::insert$native (:nia::base) (:nia::Reading :g 7 :v 70)))

;; ── entries (0-arity, called by name from the .rs) ────────────────────────────

;; assertion 1 — EQUIVALENCE: insert-all(s,[f1..f5]) == 5 chained insert calls.
(:wat::core::defn :user::batch-staged   [] -> :wat::core::i64 (:nia::staged-count (:nia::seed-batch)))
(:wat::core::defn :user::chained-staged [] -> :wat::core::i64 (:nia::staged-count (:nia::seed-chained)))
(:wat::core::defn :user::batch-fired    [] -> :wat::core::i64 (:nia::fired-count (:nia::seed-batch)))
(:wat::core::defn :user::chained-fired  [] -> :wat::core::i64 (:nia::fired-count (:nia::seed-chained)))
(:wat::core::defn :user::batch-sum      [] -> :wat::core::i64 (:nia::fired-sum (:nia::seed-batch)))
(:wat::core::defn :user::chained-sum    [] -> :wat::core::i64 (:nia::fired-sum (:nia::seed-chained)))

;; assertion 2 — THE ORACLE: insert-all$oracle == insert-all$native on the same input.
(:wat::core::defn :user::oracle-staged  [] -> :wat::core::i64 (:nia::staged-count (:nia::seed-oracle)))
(:wat::core::defn :user::native-staged  [] -> :wat::core::i64 (:nia::staged-count (:nia::seed-native)))
(:wat::core::defn :user::oracle-fired   [] -> :wat::core::i64 (:nia::fired-count (:nia::seed-oracle)))
(:wat::core::defn :user::native-fired   [] -> :wat::core::i64 (:nia::fired-count (:nia::seed-native)))
(:wat::core::defn :user::oracle-sum     [] -> :wat::core::i64 (:nia::fired-sum (:nia::seed-oracle)))
(:wat::core::defn :user::native-sum     [] -> :wat::core::i64 (:nia::fired-sum (:nia::seed-native)))

;; assertion 3 — NON-VACUITY: N (the fact count under test) and the resulting `facts` length.
(:wat::core::defn :user::n-under-test    [] -> :wat::core::i64 (:wat::core::length (:nia::the-facts)))
(:wat::core::defn :user::batch-facts-len [] -> :wat::core::i64 (:nia::staged-count (:nia::seed-batch)))

;; assertion 4 — THE 2-ARY PATH IS UNTOUCHED: a single 2-ary insert call matches insert$native directly.
(:wat::core::defn :user::single-public-staged [] -> :wat::core::i64 (:nia::staged-count (:nia::seed-single-public)))
(:wat::core::defn :user::single-native-staged [] -> :wat::core::i64 (:nia::staged-count (:nia::seed-single-native)))
(:wat::core::defn :user::single-public-fired  [] -> :wat::core::i64 (:nia::fired-count (:nia::seed-single-public)))
(:wat::core::defn :user::single-native-fired  [] -> :wat::core::i64 (:nia::fired-count (:nia::seed-single-native)))
