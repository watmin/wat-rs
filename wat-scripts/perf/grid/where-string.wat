;; wat-scripts/perf/grid/where-string.wat — THE `where`-CLAUSE EXPRESSIVITY CORPUS,
;; STRING-VERB family, wat side.
;;
;; Sibling of where-shapes.wat / where-boolean.wat (read where-shapes.wat's header first — same
;; verdict shape, same four rules, same harness; where-boolean.wat for the helper-fn pair shape this
;; file mirrors). This pair asks: over the FULL String verb surface — the per-Type `String/` family
;; (grep `wat/` for `:wat::core::String/`: `concat`, `starts-with?`, `ends-with?`, `contains?`,
;; `empty?` — five verbs, all pure∧deterministic per `src/rete/purity.rs`'s hand-managed metadata
;; map) plus the `:wat::core::string::*` namespace it sits beside (whitelisted for `where` by an
;; entire-namespace prefix match, same file) — does wat's evaluator agree with Clara/Clojure's string
;; ops on every shape a `where` predicate can build from them?
;;
;; ── HOW IT RUNS ───────────────────────────────────────────────────────────────────────────────
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-string.wat   > /tmp/ours
;;     clojure -Sdeps '…'  -M  wat-scripts/perf/grid/where-string.clj > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty  ⇒  every row agrees
;;
;; `check-where-shapes.sh where-string` is that, wrapped.
;;
;; ── THE FACT STREAM — FIVE EQUAL-SIZED STRING CATEGORIES OVER `i mod 5`, CRT-CLEAN AGAINST
;;    `i mod 2` and `i mod 8` ──────────────────────────────────────────────────────────────────
;;
;; `items` is 400 = 5 * 2 * 8 * 5 (lcm(5,2,8) = 40, and 400 = 10 * 40), so every category below
;; partitions the range EXACTLY across every row — no remainder term to fudge, and every row's
;; expected count is exact, not approximate, and reproducible by CRT over `i mod 40` the same way
;; where-boolean.wat does it over `i mod 210`.
;;
;; `r = i mod 5` selects the STRING CONTENT (`n`), deliberately spanning the edge content the brief
;; calls out by name:
;;   r=0 → n = ""            — THE EMPTY STRING. Every predicate below must handle it without
;;                              special-casing (Rust's `str::starts_with`/`contains` on `""` are
;;                              total; this row is the check that wat's dispatch doesn't panic or
;;                              diverge on the boundary).
;;   r=1 → n = "cat"         — the needle at position 0, and nothing else — length 3.
;;   r=2 → n = "zzcatzz"     — the needle in the MIDDLE, with trailing content after it too, so
;;                              `contains?` and `ends-with?` disagree on this row — length 7.
;;   r=3 → n = "ねこcat"      — UNICODE. Two BMP hiragana chars (each 3 UTF-8 bytes, 0 surrogate
;;                              pairs — wat's `char` is BMP-only per `src/intrinsic/char.rs`, so this
;;                              stays inside what both engines can represent as a single "char") in
;;                              front of the needle — length 5 CHARS (not bytes: 6+3=9 UTF-8 bytes),
;;                              directly exercising `string::length`'s "unicode scalar count, not
;;                              byte count" contract against Clojure's `count` (UTF-16 code units,
;;                              which coincide with scalar count for BMP-only content).
;;   r=4 → n = "DOG"         — no needle at all, and UPPERCASE — the case-sensitivity edge for the
;;                              to-lowercase row (10) below.
;;
;; `i mod 2` (`is-even`) independently selects two more bound fields:
;;   `tag`    = "ca" (even) | "xy" (odd)      — row 6's test-time-built argument.
;;   `padded` = "  cat  " (even) | "  dog  " (odd) — row 12's whitespace-trim edge.
;;
;; `i mod 8` selects `minlen`, a PER-FACT bound i64 threshold (0..7) — never a hidden constant —
;; for the length-vs-bound-var row (5) and the composed row (8).
;;
;; ── THE FOUR RULES (same as where-shapes.wat / where-boolean.wat; restated for this family) ────
;; 1. THE SHARED CONDITION BINDS EVERY FIELD (?k ?n ?tag ?minlen ?padded), identical in every rule.
;; 2. EVERY ROW MUST DISCRIMINATE A PROPER SUBSET — 0 < n < 400 — checked against the comment AND
;;    against this program's own `n=` (rule 2's witness).
;; 3. SEED FROM A FORMULA OVER `i`, never a table — `r`/`is-even`/`minlen` above, nothing hand-kept.
;; 4. MIRROR THE OPERATION: wat's `:wat::core::String/contains?` mirrors as Clojure's
;;    `clojure.string/includes?` (there is no Clojure `contains?` for substrings — `contains?` there
;;    tests collection/map membership, an entirely different operation, so USING the same name would
;;    be the fudge; `includes?` is the substring test and is the faithful mirror of the OPERATION).
;;    `String/empty?` mirrors as `(zero? (count s))`, never `clojure.string/blank?` — `blank?` also
;;    treats whitespace-only strings as empty, which `str::is_empty()` does not; that would be a
;;    silent semantics change, not a translation.
;;
;; ── WHY ROW 11 IS THE HEADLINE, alongside where-boolean's row 15 ────────────────────────────────
;;
;; Row 11 guards `(string::subs ?n 0 3)` — which RAISES on r=0 (`n=""`, length 0 < end 3) — behind
;; `(i64::>= (string::length ?n) 3)` in the same `and`. If wat's `and` did not short-circuit
;; left-to-right, this row would not print a wrong count on the 80 r=0 facts — it would ABORT the
;; whole process with a MalformedForm/index-out-of-range error the first time it reached one. A
;; clean `n=80` is therefore the same class of proof as where-boolean's row 15: direct behavioural
;; evidence for `eval_and`'s short-circuit, this time through a String verb's own arity/range fence
;; rather than an arithmetic one.

(:wat::core::defn :wst::items [] -> :wat::core::i64 400)   ;; 5*2*8*5 — CRT-clean, both sides

(:wat::core::defn :wst::row-count [] -> :wat::core::i64 12)

;; k mod 5    → n's category (see header)
;; k mod 2    → tag/padded's category
;; k mod 8    → minlen, a PER-FACT bound threshold (row 5's "not just a constant")
(:wat::core::defrecord :wst::Req
  [k      <- :wat::core::i64
   n      <- :wat::core::String
   tag    <- :wat::core::String
   minlen <- :wat::core::i64
   padded <- :wat::core::String])

(:wat::core::defrecord :wst::Hit [k <- :wat::core::i64])

;; row 9's user-defined pure fn: String -> bool, itself built from TWO String verbs across two
;; namespaces (`String/contains?` and `string::length`) composed with `and`. feline?(s) :=
;; contains(s,"cat") AND length(s) > 3. True for r=2 ("zzcatzz", len 7) and r=3 ("ねこcat", len 5);
;; false for r=1 ("cat", len 3 — contains but NOT longer than 3) and r=0/r=4 (no "cat" at all).
(:wat::rete::core::defn :wst::feline? [s <- :wat::core::String] -> :wat::core::bool
  (:wat::rete::core::and
    (:wat::rete::string::contains? s "cat")
    (:wat::rete::i64::> (:wat::rete::string::length s) 3)))

;; THE SHARED LEADING CONDITION, quoted once and reused by every row — only `where-c` varies.
(:wat::core::defn :wst::conds [] -> :wat::WatAST
  (:wat::core::quasiquote
    (:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded))))

(:wat::core::defn :wst::ins [] -> :wat::WatAST
  (:wat::core::quasiquote (:wst::Hit ?k)))

;; ROW 1 — String/starts-with?. Hit :- Req(…) AND (starts-with? ?n "cat").
;; True only for r=1 ("cat" itself) — r=2 has "zz" first, r=3 has "ねこ" first, r=4 is "DOG",
;; r=0 is empty (nothing starts a non-empty prefix). One category of five ⇒ 80/400.
(:wat::rete::defrule :wst::starts-with
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where (:wat::rete::string::starts-with? ?n "cat"))]
  :then
  [(:wst::Hit ?k)])

;; ROW 2 — String/ends-with?. True for r=1 ("cat" ends "cat") and r=3 ("ねこcat" ends "cat"), but
;; NOT r=2 ("zzcatzz" ends "zz") — the row that tells starts/ends/contains apart. Two of five
;; categories ⇒ 160/400.
(:wat::rete::defrule :wst::ends-with
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where (:wat::rete::string::ends-with? ?n "cat"))]
  :then
  [(:wst::Hit ?k)])

;; ROW 3 — String/contains?. True for r=1, r=2, AND r=3 (the needle anywhere) — three of five
;; categories ⇒ 240/400. Compare against row 2: r=2 flips from false (ends-with) to true
;; (contains) — that flip is the whole reason r=2 has trailing "zz" instead of being a plain
;; "zzcat" suffix-match.
(:wat::rete::defrule :wst::contains
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where (:wat::rete::string::contains? ?n "cat"))]
  :then
  [(:wst::Hit ?k)])

;; ROW 4 — String/empty?. True ONLY for r=0 (the empty-string category itself) ⇒ 80/400. THE direct
;; empty-string witness: every other row's r=0 facts flow through starts/ends/contains as legitimate
;; "no match" cases, but this row asserts the boundary is reachable and exact, not merely never hit.
(:wat::rete::defrule :wst::empty
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where (:wat::rete::string::empty? ?n))]
  :then
  [(:wst::Hit ?k)])

;; ROW 5 — string::length vs a BOUND i64 VAR (?minlen), not a constant. length(n) is fixed per
;; category: r=0→0, r=1→3, r=2→7, r=3→5, r=4→3 (chars, not bytes, for r=3). minlen = k mod 8.
;; By CRT over k mod 40 (categories r=k mod 5, thresholds m=k mod 8 vary independently), matches
;; per 40 = (m<0 count for L=0) + (m<3 for L=3, twice) + (m<7 for L=7) + (m<5 for L=5)
;;         =         0          +      3     +    3    +     7        +     5         = 18/40
;; ⇒ 180/400 over the 10 cycles in [0,400).
(:wat::rete::defrule :wst::length-bound
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where (:wat::rete::i64::> (:wat::rete::string::length ?n) ?minlen))]
  :then
  [(:wst::Hit ?k)])

;; ROW 6 — the ARGUMENT is built AT TEST TIME, not a literal: needle = (String/concat ?tag "t"),
;; so the compiler must know the second arg to `contains?` can be a runtime value, not a constant
;; to fold. tag = "ca" (even k) or "xy" (odd k) ⇒ needle = "cat" (even) or "xyt" (odd, never found).
;; By CRT over k mod 10 (parity × r): needle="cat" matches r=1,r=2,r=3 when k even; needle="xyt"
;; never matches anything. Of the 5 even/odd × 5 r combos per 10, "cat" hits at (even,r=1),
;; (even,r=2), (even,r=3) = 3 of 10 ⇒ 120/400.
(:wat::rete::defrule :wst::dynamic-arg
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where
                                 (:wat::rete::string::contains? ?n (:wat::rete::string::concat ?tag "t")))]
  :then
  [(:wst::Hit ?k)])

;; ROW 7 — a String verb INSIDE a boolean composition: (contains "cat") AND (NOT (starts-with
;; "cat")). Contains ⇒ {r1,r2,r3}; excluding starts-with's {r1} leaves {r2,r3} ⇒ 160/400.
(:wat::rete::defrule :wst::compose-bool
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where
                                 (:wat::rete::core::and
                                   (:wat::rete::string::contains? ?n "cat")
                                   (:wat::rete::core::not (:wat::rete::string::starts-with? ?n "cat"))))]
  :then
  [(:wst::Hit ?k)])

;; ROW 8 — a String verb feeding an i64 comparison, itself composed with `and`: (contains "cat")
;; AND (?minlen > 3). Contains ⇒ {r1,r2,r3} (3 of 5 r-values); minlen>3 ⇒ m in {4,5,6,7} (4 of 8).
;; Independent (CRT, k mod 40) ⇒ 3*4 = 12/40 ⇒ 120/400.
(:wat::rete::defrule :wst::compose-i64
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where
                                 (:wat::rete::core::and
                                   (:wat::rete::string::contains? ?n "cat")
                                   (:wat::rete::i64::> ?minlen 3)))]
  :then
  [(:wst::Hit ?k)])

;; ROW 9 — the user-defined pure fn (String -> bool). Hit :- Req(…) AND (feline? ?n).
;; See :wst::feline? above: true for r=2, r=3 ⇒ 160/400.
(:wat::rete::defrule :wst::userfn
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where (:wst::feline? ?n))]
  :then
  [(:wst::Hit ?k)])

;; ROW 10 — a String verb feeding ANOTHER String verb's argument (value composition, not boolean):
;; (starts-with? (to-lowercase ?n) "dog"). Only r=4 ("DOG") lowercases to "dog" and matches — this
;; is the case-sensitivity edge: `String/starts-with?` is case-sensitive, so without the
;; `to-lowercase` wrapper this row would derive the EMPTY set (STOP-2 territory), and that gap is
;; exactly what the row is for. One of five categories ⇒ 80/400.
(:wat::rete::defrule :wst::lowercase-chain
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where
                                 (:wat::rete::string::starts-with? (:wat::rete::string::to-lowercase ?n) "dog"))]
  :then
  [(:wst::Hit ?k)])

;; ROW 11 — SHORT-CIRCUIT-SENSITIVE (see header). `(string::subs ?n 0 3)` raises when
;; `length(?n) < 3` (r=0, the empty string, is exactly that fact). Guarding it behind
;; `(i64::>= (string::length ?n) 3)` in the same `and` means a non-short-circuiting `and` would not
;; derive a wrong set — it would ABORT the process on the first r=0 fact. Of the categories that
;; clear the guard (r=1,2,3,4 — all length >= 3), only r=1's first three chars are literally "cat";
;; r=2's are "zzc", r=3's are "ねこc" (char-indexed, not byte-indexed — a 2-BMP-char unicode prefix
;; still only consumes 2 of the 3 requested chars), r=4's are "DOG" (case-sensitive, no match).
;; One of five categories ⇒ 80/400.
(:wat::rete::defrule :wst::shortcircuit-subs
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where
                                 (:wat::rete::core::and
                                   (:wat::rete::i64::>= (:wat::rete::string::length ?n) 3)
                                   (:wat::rete::string::starts-with? (:wat::rete::string::subs ?n 0 3 :undefined "") "cat")))]
  :then
  [(:wst::Hit ?k)])

;; ROW 12 — string::trim feeding `=`, over the WHITESPACE edge. padded = "  cat  " (even k) or
;; "  dog  " (odd k); trimming and comparing to the literal "cat" selects exactly the even half.
;; Half of the stream ⇒ 200/400.
(:wat::rete::defrule :wst::trim-eq
  :when
  [(:wst::Req (?k <- :k) (?n <- :n) (?tag <- :tag) (?minlen <- :minlen) (?padded <- :padded)) (:wat::rete::where (:wat::rete::string::= (:wat::rete::string::trim ?padded) "cat"))]
  :then
  [(:wst::Hit ?k)])

(:wat::rete::defquery :wst::q-Hit
  :params []
  :when [(?fact <- :wst::Hit)])


;; build-rules — THE ROW DISPATCH. An unknown row is a located failure, never a silent fallback.
(:wat::core::defn :wst::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1)  (:wst::starts-with))
      ((:wat::core::= row 2)  (:wst::ends-with))
      ((:wat::core::= row 3)  (:wst::contains))
      ((:wat::core::= row 4)  (:wst::empty))
      ((:wat::core::= row 5)  (:wst::length-bound))
      ((:wat::core::= row 6)  (:wst::dynamic-arg))
      ((:wat::core::= row 7)  (:wst::compose-bool))
      ((:wat::core::= row 8)  (:wst::compose-i64))
      ((:wat::core::= row 9)  (:wst::userfn))
      ((:wat::core::= row 10) (:wst::lowercase-chain))
      ((:wat::core::= row 11) (:wst::shortcircuit-subs))
      ((:wat::core::= row 12) (:wst::trim-eq))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::string::concat "where-string: unknown row " (:wat::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed — stage Req(i) for i in [0, items) via the BATCH verb (one rebuild). Every field is a
;; FORMULA over i, independently computable on the Clara side so nothing rots as a hand-kept table.
(:wat::core::defn :wst::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::let [r       (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 5) 5))
                          is-even (:wat::core::= 0 (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 2) 2)))
                          nm      (:wat::core::cond
                                    ((:wat::core::= r 0) "")
                                    ((:wat::core::= r 1) "cat")
                                    ((:wat::core::= r 2) "zzcatzz")
                                    ((:wat::core::= r 3) "ねこcat")
                                    (:else "DOG"))
                          tg      (:wat::core::if is-even "ca" "xy")
                          ml      (:wat::i64::- i (:wat::i64::* (:wat::i64::/ i 8) 8))
                          pd      (:wat::core::if is-even "  cat  " "  dog  ")]
          (:wat::vector::conj acc
            (:wst::Req :k i :n nm :tag tg :minlen ml :padded pd))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; derived-ints fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wst::derived-ints
  [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:wst::Hit/k f)))
        (:wat::rete::query fired (:wst::q-Hit))))))

;; render-ints — " 3 13 23 …". A plain space-joined rendering, NOT the EDN printer — see
;; where-shapes.wat's identical helper for why this must not be `:wat::edn::write`.
(:wat::core::defn :wst::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc
        (:wat::string::concat " " (:wat::i64::to-string x))))
    ""
    v))

;; run-row row -> the corpus line for ONE shape, in its OWN session.
;; rule-display-name — TOTAL derivation of the printed row label from a Rule/name that may
;; now carry this file's namespace prefix (e.g. "NS::arith") after the namespacing wall.
;; `string::split` on "::" always returns >= 1 segment (the whole string, unsplit, when
;; "::" is absent); folding with SEED = full while always overwriting the accumulator
;; with the current segment lands on the LAST segment without ever calling a partial
;; verb (`first`/`nth`/`Option/expect`) — the seed also makes the no-"::" case return
;; the input UNCHANGED, and even an impossible empty split falls back to the seed
;; instead of raising.
(:wat::core::defn :wst::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::string::split full "::")))

(:wat::core::defn :wst::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wst::build-rules row)
                    rule    (:wat::core::first rules)
                    staged  (:wst::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wst::q-Hit))) (:wst::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wst::derived-ints fired)
                    n       (:wat::vec::length derived)]
    (:wat::string::concat
      (:wat::string::concat
        (:wat::string::concat "row " (:wat::i64::to-string row))
        (:wat::string::concat " " (:wst::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string n))
        (:wat::string::concat " ->" (:wst::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wst::run-row row)))
    nil
    (:wat::core::range 1 (:wat::i64::+ (:wst::row-count) 1))))
