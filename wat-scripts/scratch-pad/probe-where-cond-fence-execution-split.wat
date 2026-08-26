;; probe-where-cond-fence-execution-split.wat — A `where` FORM THAT PASSES THE FENCE AND DIES ON FIRE.
;;
;; THE FINDING (where-control corpus family, 2026-08-01). `:wat::core::cond` in a `where` clause:
;;
;;   - `--check` passes it CLEAN.
;;   - `:wat::rete::compile`'s purity fence passes it CLEAN (classify_expr has a clause-aware `cond`
;;     arm that structurally approves it).
;;   - and then `fire-rules` dies on the FIRST FIRE with `#wat.runtime/UnknownFunction`.
;;
;; THE ROOT, as reported and reproduced here: `cond` is a `defmacro` (`wat/core.wat`), not a runtime
;; primitive like `if`/`let`/`match` (which dispatch directly in `eval_inner`). A `where` expression is
;; captured as DATA by `quasiquote` and evaluated later by `eval-test`, which never macro-expands. So
;; the fence reasons about a form the evaluator cannot execute.
;;
;; ── WHY THIS IS A HIDDEN-FAILURE CLASS, not a capability boundary ─────────────────────────────
;;
;; A capability boundary is honest: the checker refuses the form, at the site, before you ship. This
;; is the opposite shape — every static gate says YES and the failure arrives at runtime, on the first
;; fire, in whatever process happened to run the rule. That is the exact silhouette arc 278's law
;; forbids (R55 `REVOLVTIONE, NVLLA LARVA`), and it is the R57 lesson recurring: a law is completed by
;; USE, not by declaration — the fence was declared complete and a real consumer walked into a gap.
;;
;; It is also `NON MVRVS SED VITIVM` (R24) inverted. There, a "wall" turned out to be a flaw. Here a
;; "pass" turns out to be one: the fence's YES is not a wall that held, it is a wall with a hole.
;;
;; The specific danger for #49a: a compiled-`where` executor built by consulting the PURITY FENCE for
;; its accepted language would inherit this gap exactly — it would believe `cond` is in the language.
;;
;; ── WHAT THIS PROBE IS ────────────────────────────────────────────────────────────────────────
;;
;; A RED gate. It is EXPECTED TO DIE, loudly, with UnknownFunction naming `:wat::core::cond`. It goes
;; GREEN (i.e. it stops dying) when the split is closed — whichever way it is closed:
;;   (a) the fence learns to REJECT macro heads in a `where`, at compile time, located; or
;;   (b) the `where` capture macro-expands, so `cond` genuinely works.
;; Either is a fix. Both make this probe stop raising. Do not "fix" it by deleting the probe.
;;
;; Run:  ./target/release/wat wat-scripts/scratch-pad/probe-where-cond-fence-execution-split.wat
;; Expect: a raise. `--check` on this file passes, which is half the point.

(:wat::core::defrecord :pcf::Req [k <- :wat::core::i64])
(:wat::core::defrecord :pcf::Hit [k <- :wat::core::i64])

(:wat::rete::defquery :pcf::q-Hit
  :params []
  :when [(?fact <- :pcf::Hit)])


;; The control: the SAME branching logic spelled with `if`, a runtime primitive. This one works, which
;; is what proves the wall is `cond`'s MACRO-NESS and not the branching.
(:wat::core::defn :pcf::rule-if [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:pcf::Req (?k <- :k)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::rete::core::if (:wat::core::= 0 (:wat::i64::- ?k (:wat::i64::* (:wat::i64::/ ?k 2) 2)))
                                  true
                                  false)))
                    ins     (:wat::core::quasiquote (:pcf::Hit ?k))]
    (:wat::rete::Rule :name "if-control"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; THE SUBJECT: identical logic, spelled with `cond`. Passes --check. Passes the purity fence.
;; Dies on the first fire.
(:wat::core::defn :pcf::rule-cond [] -> :wat::rete::Rule
  (:wat::core::let [conds   (:wat::core::quasiquote (:pcf::Req (?k <- :k)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::rete::core::cond
                                  ((:wat::core::= 0 (:wat::i64::- ?k (:wat::i64::* (:wat::i64::/ ?k 2) 2))) true)
                                  (:else false))))
                    ins     (:wat::core::quasiquote (:pcf::Hit ?k))]
    (:wat::rete::Rule :name "cond-subject"
      :lhs (:wat::core::PersistentVector conds where-c)
      :rhs (:wat::core::PersistentVector ins))))

(:wat::core::defn :pcf::seed [session <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::vector::conj acc (:pcf::Req :k i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 10))))

(:wat::core::defn :pcf::derived [fired <- :wat::rete::Session] -> :wat::core::i64
  (:wat::vec::length
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:pcf::Hit/k f)))
        (:wat::rete::query fired (:pcf::q-Hit))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    ;; 1. The `if` control fires and derives — so the branching logic is fine.
    [ctl     (:wat::rete::fire-rules
               (:pcf::seed (:wat::rete::compile-all (:wat::core::PersistentVector (:pcf::rule-if)) (:wat::core::PersistentVector (:pcf::q-Hit)))))
     _ok     (:wat::kernel::println
               (:wat::core::String/concat "if-control derived n=" (:wat::i64::to-string (:pcf::derived ctl))))

     ;; 2. COMPILE the cond rule. The purity fence runs HERE and passes — this line does not raise,
     ;;    which is precisely the defect: every static gate has now said yes.
     compiled (:wat::rete::compile-all (:wat::core::PersistentVector (:pcf::rule-cond)) (:wat::core::PersistentVector (:pcf::q-Hit)))
     _fence   (:wat::kernel::println "cond-subject PASSED compile + the purity fence")

     ;; 3. FIRE. This raises #wat.runtime/UnknownFunction on :wat::core::cond.
     _fire    (:wat::rete::fire-rules (:pcf::seed compiled))]
    (:wat::kernel::println "UNEXPECTED: cond fired without raising — the split may be CLOSED; re-read this probe's header")))
