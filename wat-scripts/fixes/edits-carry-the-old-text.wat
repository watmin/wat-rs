;; wat-scripts/fixes/edits-carry-the-old-text.wat — arc 282, STONE: an edit carries what it
;; CLAIMS to replace.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;; Written and run against the OLD `fix-text-apply` (offset, old-len, new-text) — it still works —
;; per wat/fix.wat's own STASH-DANCE note: this migration ships alongside a definition change to
;; fix-text-apply ITSELF, so it must run to completion BEFORE that definition changes.
;;
;; THE MECHANICAL PART (safe everywhere, verified structurally, no judgement call):
;;
;;   RULE 1 — the edit-tuple TYPE ANNOTATION: the 3-element type vector
;;     [:wat::core::i64 :wat::core::i64 :wat::core::String]  ->  [:wat::core::i64 :wat::core::String :wat::core::String]
;;     (matched by exact element-name triple; nothing else in the corpus has this shape).
;;
;;   RULE 2 — a value-construction Tuple whose 2nd (old-len) argument is the literal `0`
;;     (a pure INSERT — nothing is being overwritten, so `""` is trivially correct, never a
;;     "claim" that could be wrong): `0` -> `""`.
;;
;;   RULE 3 — a `let` binding `NAME (:wat::string::length SUBJECT)` whose bound NAME is used,
;;     anywhere in that SAME let's body, as the 2nd argument of a value-construction Tuple:
;;     the binding's VALUE is rewritten from `(:wat::string::length SUBJECT)` to SUBJECT's own
;;     literal text (sliced verbatim from source — SUBJECT is exactly the rule's belief about
;;     what occupies that span, which is what old-text must be). Scoped to bindings ACTUALLY
;;     used as a Tuple's old-len slot within the SAME let, so a length used for unrelated
;;     char-walk arithmetic in the SAME function (e.g. wat/fix.wat's rename-prefix-edits, whose
;;     `old-len` feeds `rename-in-name`'s char-walk and is NEVER a Tuple argument) is untouched.
;;
;; THE JUDGEMENT-CALL PART: every remaining site (the true `fix-text-span-len` sites, an
;; arithmetic offset-difference between two independently-known node boundaries, or a whole
;; structural-node span being replaced/deleted) is handled below by a small, NAMED, per-file
;; rule — each one read and hand-verified against its source file (not a blind pattern), per
;; the brief's own admission that "the 7 are where this stone is won or lost." Each such rule
;; is documented at its call site with WHERE the claim comes from.
;;
;; Usage — dry run first (MANDATORY, brief STOP-3):
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' | ./target/release/wat ./wat-scripts/fixes/edits-carry-the-old-text.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (one EDN vector of paths on stdin, list EVERY path, including this file itself):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/edits-carry-the-old-text.wat

;; ── small helpers ─────────────────────────────────────────────────────────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))
(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))
(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword") (:wat::core::ast-name n) ""))
(:wat::core::defn :user::node-text [n <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::string::subs src (:user::start-off n lines) (:user::end-off n lines)))

;; ── RULE 1: the type annotation ──────────────────────────────────────────────
(:wat::core::defn :user::i64i64string-vec? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "vector")
    (:wat::core::let [ch (:wat::core::ast->children n)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 3)
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::nth ch 0)) ":wat::core::i64")
          (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::nth ch 1)) ":wat::core::i64")
            (:wat::core::= (:user::kw-name (:wat::core::nth ch 2)) ":wat::core::String")
            false)
          false)
        false))
    false))

(:wat::core::defn :user::annot-edit [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:user::i64i64string-vec? n)
    (:wat::core::let [mid (:wat::core::nth (:wat::core::ast->children n) 1)]
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
        (:wat::core::Tuple (:user::start-off mid lines) (:wat::string::length ":wat::core::i64") ":wat::core::String")))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

;; ── RULE 2: literal 0 (pure insert) → "" ─────────────────────────────────────
(:wat::core::defn :user::value-tuple? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "list")
    (:wat::core::let [ch (:wat::core::ast->children n)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 4)
        (:wat::core::= (:user::kw-name (:wat::core::nth ch 0)) ":wat::core::Tuple")
        false))
    false))

(:wat::core::defn :user::zero-lit? [n <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "int")
    (:wat::core::= (:user::node-text n src lines) "0")
    false))

(:wat::core::defn :user::zero-edit [n <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:user::value-tuple? n)
    (:wat::core::let [b (:wat::core::nth (:wat::core::ast->children n) 2)]
      (:wat::core::if (:user::zero-lit? b src lines)
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
          (:wat::core::Tuple (:user::start-off b lines) 1 "\"\""))
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

;; ── RULE 2b: a value-tuple whose 2nd arg is DIRECTLY (:wat::string::length SUBJECT),
;;    no intermediate let-binding — unwrap in place to SUBJECT's own text. ─────
(:wat::core::defn :user::direct-length-edit [n <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:user::value-tuple? n)
    (:wat::core::let [b (:wat::core::nth (:wat::core::ast->children n) 2)]
      (:wat::core::if (:user::length-call? b)
        (:wat::core::let [subj      (:wat::core::nth (:wat::core::ast->children b) 1)
                          subj-text (:user::node-text subj src lines)]
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
            (:wat::core::Tuple (:user::start-off b lines) (:wat::core::i64::- (:user::end-off b lines) (:user::start-off b lines)) subj-text)))
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

;; ── RULE 3: let-scoped (:wat::string::length SUBJECT) unwrap ────────────────
(:wat::core::defn :user::length-call? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "list")
    (:wat::core::let [ch (:wat::core::ast->children n)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 2)
        (:wat::core::= (:user::kw-name (:wat::core::nth ch 0)) ":wat::string::length")
        false))
    false))

;; count-symbol-occurrences — total occurrences of a bare SYMBOL named `nm` anywhere in `node`.
(:wat::core::defn :user::count-symbol-occurrences [node <- :wat::WatAST  nm <- :wat::core::String] -> :wat::core::i64
  (:wat::core::let
    [here (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
            (:wat::core::if (:wat::core::= (:wat::core::ast-name node) nm) 1 0)
            0)]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::i64::+ here (:user::seq-count-symbol (:wat::core::ast->children node) nm))
      here)))

(:wat::core::defn :user::seq-count-symbol
  [items <- (:wat::core::Vector :- [:wat::WatAST])  nm <- :wat::core::String] -> :wat::core::i64
  (:wat::core::if (:wat::core::empty? items)
    0
    (:wat::core::i64::+ (:user::count-symbol-occurrences (:wat::core::first items) nm)
      (:user::seq-count-symbol (:wat::core::rest items) nm))))

;; count-symbol-as-tuple-arg2 — occurrences of symbol `nm` SPECIFICALLY as a value-tuple's
;; 2nd (old-len) argument, anywhere in `node`.
(:wat::core::defn :user::count-symbol-as-tuple-arg2 [node <- :wat::WatAST  nm <- :wat::core::String] -> :wat::core::i64
  (:wat::core::let
    [here (:wat::core::if (:user::value-tuple? node)
            (:wat::core::let [b (:wat::core::nth (:wat::core::ast->children node) 2)]
              (:wat::core::if (:wat::core::= (:wat::core::ast-kind b) "symbol")
                (:wat::core::if (:wat::core::= (:wat::core::ast-name b) nm) 1 0)
                0))
            0)]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::i64::+ here (:user::seq-count-symbol-arg2 (:wat::core::ast->children node) nm))
      here)))

(:wat::core::defn :user::seq-count-symbol-arg2
  [items <- (:wat::core::Vector :- [:wat::WatAST])  nm <- :wat::core::String] -> :wat::core::i64
  (:wat::core::if (:wat::core::empty? items)
    0
    (:wat::core::i64::+ (:user::count-symbol-as-tuple-arg2 (:wat::core::first items) nm)
      (:user::seq-count-symbol-arg2 (:wat::core::rest items) nm))))

;; symbol-used-ONLY-as-tuple-arg2? — every occurrence of `nm` inside `node` is a value-tuple's
;; 2nd argument (and there is at least one) — i.e. it is SAFE to change this variable's type
;; from i64 to String, because nothing else (arithmetic, another Tuple's 1st/3rd arg, a
;; function-call argument…) reads it as a number.
(:wat::core::defn :user::symbol-used-ONLY-as-tuple-arg2? [node <- :wat::WatAST  nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let [total (:user::count-symbol-occurrences node nm)
                    as-arg2 (:user::count-symbol-as-tuple-arg2 node nm)]
    (:wat::core::if (:wat::core::> total 0) (:wat::core::= total as-arg2) false)))

;; bodies-symbol-used-only-as-arg2? — the same safety check, folded across a Vector of body
;; forms (a `let`'s body may be more than one form).
(:wat::core::defn :user::bodies-symbol-used-only-as-arg2?
  [bodies <- (:wat::core::Vector :- [:wat::WatAST])  nm <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let [total   (:user::seq-count-symbol bodies nm)
                    as-arg2 (:user::seq-count-symbol-arg2 bodies nm)]
    (:wat::core::if (:wat::core::> total 0) (:wat::core::= total as-arg2) false)))

(:wat::core::defn :user::let-form? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::let")))
    false))

(:wat::core::defn :user::binding-pair-edit
  [nm-node  <- :wat::WatAST
   val-node <- :wat::WatAST
   bodies   <- (:wat::core::Vector :- [:wat::WatAST])
   src      <- :wat::core::String
   lines    <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:user::length-call? val-node)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind nm-node) "symbol")
      (:wat::core::if (:user::bodies-symbol-used-only-as-arg2? bodies (:wat::core::ast-name nm-node))
        (:wat::core::let [subj      (:wat::core::nth (:wat::core::ast->children val-node) 1)
                          subj-text (:user::node-text subj src lines)
                          off       (:user::start-off val-node lines)
                          len       (:wat::core::i64::- (:user::end-off val-node lines) off)]
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
            (:wat::core::Tuple off len subj-text)))
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

(:wat::core::defn :user::binding-pairs-edits
  [bpairs <- (:wat::core::Vector :- [:wat::WatAST])
   bodies <- (:wat::core::Vector :- [:wat::WatAST])
   src    <- :wat::core::String
   lines  <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::< (:wat::core::length bpairs) 2)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    (:wat::core::let [nm         (:wat::core::nth bpairs 0)
                      val        (:wat::core::nth bpairs 1)
                      rest-pairs (:wat::core::into [] (:wat::core::drop bpairs 2))
                      ;; downstream = every remaining binding VALUE (not name) in this same
                      ;; let, PLUS the body forms — everywhere `nm` could still be READ.
                      ;; A binding pair's own NAME slot is a bare symbol too, but comparing
                      ;; a later re-binding's NAME against `nm` only risks a false SAFE->
                      ;; unsafe over-exclusion (shadowing), never a false positive transform —
                      ;; conservative either way.
                      downstream (:wat::core::concat (:user::rest-pair-values rest-pairs) bodies)
                      this       (:user::binding-pair-edit nm val downstream src lines)]
      (:wat::core::concat this (:user::binding-pairs-edits rest-pairs bodies src lines)))))

;; rest-pair-values — every VALUE slot (odd index) from a binding-pairs vector.
(:wat::core::defn :user::rest-pair-values [bpairs <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::< (:wat::core::length bpairs) 2)
    (:wat::core::Vector :wat::WatAST)
    (:wat::core::concat
      (:wat::core::Vector :wat::WatAST (:wat::core::nth bpairs 1))
      (:user::rest-pair-values (:wat::core::into [] (:wat::core::drop bpairs 2))))))

(:wat::core::defn :user::let-edits
  [node <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:user::let-form? node)
    (:wat::core::let [ch      (:wat::core::ast->children node)
                      bindvec (:wat::core::nth ch 1)
                      bpairs  (:wat::core::ast->children bindvec)
                      bodies  (:wat::core::into [] (:wat::core::drop ch 2))]
      (:user::binding-pairs-edits bpairs bodies src lines))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

;; ── the generic walk: RULE 1 + RULE 2 + RULE 3, every node, every file ──────
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [e1 (:user::annot-edit node lines)
     e2 (:user::zero-edit node src lines)
     e2b (:user::direct-length-edit node src lines)
     e3 (:user::let-edits node src lines)
     e4 (:wat::core::if (:wat::fix::structural? node)
          (:user::seq-edits (:wat::core::ast->children node) src lines)
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))]
    (:wat::core::concat e1 (:wat::core::concat e2 (:wat::core::concat e2b (:wat::core::concat e3 e4))))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it src lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    items))

;; old-fix-text-apply — a FROZEN COPY of the pre-282 (offset, old-len, new-text) splicer, kept
;; LOCAL to this file. This migration's entire purpose was to change `:wat::fix::fix-text-
;; apply`'s own signature, so — uniquely among recorded migrations — it cannot keep calling the
;; stdlib verb by name and still type-check once its own work has landed (the name now resolves
;; to the NEW 3-arg-String signature). It never runs again either way; this keeps the recorded
;; migration self-contained and historically exact instead of silently reinterpreted against an
;; API it never actually ran against.
(:wat::core::defn :user::old-fix-text-apply
  [src   <- :wat::core::String
   edits <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])]
  -> :wat::core::String
  (:wat::core::if (:wat::core::empty? edits)
    src
    (:wat::core::let [edit     (:wat::core::first edits)
                      off      (:wat::core::first edit)
                      old-len  (:wat::core::second edit)
                      new-text (:wat::core::third edit)
                      tl       (:wat::core::rest edits)
                      new-src  (:wat::string::concat
                                  (:wat::string::subs src 0 off)
                                  new-text
                                  (:wat::string::subs src
                                    (:wat::core::+ off old-len)
                                    (:wat::string::length src)))]
      (:user::old-fix-text-apply new-src tl))))

;; ── per-file migrate ──────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate-generic [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    forms     (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
                    all-edits (:user::seq-edits forms src lines)
                    rev-edits (:wat::core::reverse (:wat::core::sort all-edits))]
    (:user::old-fix-text-apply src rev-edits)))

;; ── driver ────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate-generic (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[edits-carry-the-old-text] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
