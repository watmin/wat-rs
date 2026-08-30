;; wat-scripts/perf/grid/node-share.wat — GRID AXIS A8: node-sharing / rule-count, IN WAT.
;;
;; The subtree-reuse axis (docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md;
;; CLARA-TRANSLATIONS.md §A8): MANY rules that all share the SAME leading join-prefix, stressed at
;; growing rule-count N. wat's compiler dedups a shared prefix structurally —
;; `find-or-mint-alpha` (wat/rete.wat:422+, dedup key "alpha:<write-forms cond>") collapses the
;; identical leading condition into ONE AlphaNode, and `find-or-mint-hash-join` (wat/rete.wat:483+,
;; dedup key "hashjoin:<parent-id>:<cond-text>") collapses the identical join onto ONE beta subtree
;; (only ProductionNodes are per-rule, wat/rete.wat:781). So N rules sharing [A]⋈[B] compile to one
;; shared alpha + one shared root-join fanning out to N per-rule continuations.
;;
;; CAVEAT (CLARA-TRANSLATIONS.md §A8, load-bearing): this is a SPEED / node-count axis, NOT an
;; accuracy axis. Both engines are proven by their own compilers' dedup to collapse the shared
;; prefix into one subtree with an otherwise-identical logical network, so the derived-fact SET is
;; identical by construction regardless of rule count — there is no scenario where sharing changes
;; semantics. The :derived vector is therefore a SINGLE derived-set sanity check (there is no
;; per-size accuracy gate); the real measurement is fire-cost as rule-count N grows with a fixed
;; shared join-prefix depth.
;;
;; Shape (mirrors the A8 benchmark form — N rules with a common leading join-prefix, differentiated
;; by a per-rule trailing predicate so types stay FIXED while N is a pure runtime rule-count dial,
;; the same "unquote a bare i64 literal into a predicate" trick min-finding.wat uses for its gate):
;;   A(k), B(k)                              — M seed facts each, k in [0, items).
;;   r_i (i in [0, N)):  Out(k) :- A(k) AND B(k) AND (i == k mod N)
;; The leading [A (?k)]⋈[B (?k)] join-prefix is byte-identical across all N rules (→ shared alpha +
;; shared hash-join); each rule diverges only at its trailing (:wat::rete::where ...) TestNode,
;; carrying the per-rule literal i. Since every k in [0, items) satisfies EXACTLY one rule
;; (i == k mod N), the union of all N rules' output is {Out(k) : k in [0, items)} — one Out per k,
;; independent of N. That N-invariance is exactly why the derived set is a one-time sanity check.
;;
;; size = [rules items]. `rules` (N) is the swept dial (the node-sharing / activation-cost driver);
;; `items` (M) is the fixed shared-prefix fan-in. There is NO type ceiling here (unlike strat-neg's
;; static S0..S9) — rules are differentiated by a runtime literal, not by type, so N is unbounded.
;;
;; Fires the NATIVE production verb `:wat::rete::fire-rules` (compile + seed are un-timed setup) —
;; the differential-tested fast path, NOT the wat oracle `fire-rules-spec`.
;;
;; :derived is the FULL SORTED derived Out set, each fact canonicalized as its i64 key k, so it
;; compares byte-for-byte against Clara's rendering of the identical workload (gen-node-share.sh).
;;
;; Usage (stdin = an i64 vector [rules items]; stdout = one #grid/Result EDN line):
;;   echo '[50 200]' | cargo wat ./wat-scripts/perf/grid/node-share.wat
;;   => #grid/Result {:axis "node-share" :size [50 200] :derived [...] :native-ns N}

(:wat::core::defrecord :nsh::A   [k <- :wat::core::i64])   ;; shared leading condition
(:wat::core::defrecord :nsh::B   [k <- :wat::core::i64])   ;; shared join partner (prefix = A ⋈ B)
(:wat::core::defrecord :nsh::Out [k <- :wat::core::i64])   ;; per-rule production (single output type)

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64])

(:wat::rete::defquery :nsh::q-Out
  :params []
  :when [(?fact <- :nsh::Out)])


;; build-rule i n — the i-th rule of the N-rule set:
;;   Out(k) :- A(k) AND B(k) AND (i == k mod N)
;; The leading [A (?k)] and [B (?k)] conditions are BYTE-IDENTICAL across every i (no i splices into
;; them) → they compile to the one shared AlphaNode + one shared hash-join (the point of this axis).
;; The trailing (:wat::rete::where ...) is a per-rule beta TestNode (compile-condition's where-branch,
;; wat/rete.wat:547+) whose eval-test does a genuine eval_inner — full expressions are fine there.
;; `i` and `n` splice in as bare i64 LITERALS via unquote (proven — min-finding.wat unquotes its
;; threshold the same way; deep-cascade embeds `(= ?l (unquote prev))`). There is no native i64 mod
;; (only + - * /), so `k mod n` is written inline as the truncating-division idiom `k - (k/n)*n`
;; (k >= 0, n > 0 at every grid size → exact), the same shape strat-neg.wat's even test uses.
;; Arc 278 law A: a `where` predicate admits ONLY `:wat::rete::` ops. `=` is a pure rename
;; (`:wat::rete::core::i64::=`, OpClass::Alias). `- * /` are OpClass::Fallback — 4-ary
;; (operand operand :undefined fallback). Fallback -1 at all three arithmetic sites: k>=0, n>0 by
;; construction here, so every honest quotient/product/remainder along this chain is >= 0 — -1 can
;; never be produced by a real (non-overflow, non-zero-divisor) evaluation, so if the undefined
;; point were ever reached the token simply fails every rule's `where` (i, the compared literal, is
;; always in [0, n) — never -1) rather than silently landing on a plausible wrong match.
(:wat::core::defn :nsh::build-rule [i <- :wat::core::i64  n <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [a-c     (:wat::core::quasiquote (:nsh::A (?k <- :k)))
                    b-c     (:wat::core::quasiquote (:nsh::B (?k <- :k)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::rete::core::i64::= (:wat::core::unquote i)
                                  (:wat::rete::core::i64::- ?k
                                    (:wat::rete::core::i64::* (:wat::rete::core::i64::/ ?k (:wat::core::unquote n) :undefined -1) (:wat::core::unquote n) :undefined -1)
                                    :undefined -1))))
                    ins     (:wat::core::quasiquote (:nsh::Out ?k))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string i)
      :lhs (:wat::core::PersistentVector a-c b-c where-c)
      :rhs (:wat::core::PersistentVector ins))))

;; build-rules n — the N-rule set [r0 .. r(n-1)], folding build-rule over (range 0 n). Every rule
;; shares the leading [A]⋈[B] join-prefix; only the trailing literal differs (mirrors strat-neg's
;; build-rules fold shape).
(:wat::core::defn :nsh::build-rules [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  i <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:nsh::build-rule i n)))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 n)))

;; seed session items — stage A(i) AND B(i) for i in [0, items), threading the staging session, so
;; the shared [A]⋈[B] join yields exactly one token per k (fan-in = items).
;; Staged with the BATCH verb — one `insert-all` (native, one rebuild) rather than `insert` x 2N.
;; The fact vector is heterogeneous (A and B), which `(PersistentVector :- [Record])` carries fine.
(:wat::core::defn :nsh::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::PersistentVector/conj (:wat::core::PersistentVector/conj acc (:nsh::A i)) (:nsh::B i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]). DESIGN-STONE-into-pv-
;; from-vector.md: `into` now has a native ((PersistentVector :- [T]), (Vector :- [T])) clause backed by one
;; `PersistentVector/concat` call — retiring the N-interpreted-closure-invocation conj-fold.
(:wat::core::defn :nsh::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; derived-vector fired — every derived Out fact's key k, sorted ascending. THE (one-time) sanity
;; witness: the full derived set. Expected = [0 1 .. items-1], independent of rule-count N.
(:wat::core::defn :nsh::derived-vector
  [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:nsh::vec->pvec
    (:wat::core::sort
      (:wat::core::into (:wat::core::Vector :wat::core::i64)
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:nsh::Out/k f)))
          (:wat::rete::query fired (:nsh::q-Out)))))))

;; ns-between t0 t1 — nanoseconds between two Instants (cf. min-finding.wat).
(:wat::core::defn :nsh::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    rules-n (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [rules items]")
                    items   (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [rules items]")
                    rules   (:nsh::build-rules rules-n)
                    staged  (:nsh::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:nsh::q-Out))) items)
                    ;; time the NATIVE production verb only (compile + seed are un-timed setup)
                    n0      (:wat::time::now)
                    fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    n1      (:wat::time::now)
                    derived (:nsh::derived-vector fired)
                    nat-ns  (:nsh::ns-between n0 n1)
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::core::match (:wat::rete::fire-rules$oracle staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "node-share" :size (:wat::core::PersistentVector rules-n items) :derived derived :native-ns nat-ns :oracle-derived (:nsh::derived-vector ofired) :oracle-ns (:nsh::ns-between o0 o1)))))
