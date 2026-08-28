;; wat-scripts/fixes/to-faithful-clojure-rete.wat — the faithful-Clojure conversion as PURE rete.
;;
;; wat rewrites wat with its OWN rule engine. rete is always pure: the RULES DEDUCE
;; classification facts; this DRIVE queries them out and ACTIONS them (the transform + the
;; I/O), OUTSIDE rete. A :then never transforms a value — the byte-for-byte reproduction of
;; :wat::fix::fix-text is achieved by the deduce-then-action split, not by an engine change.
;;
;; PIPELINE (per file):
;;   read → parse → position-aware walk emitting :fix::Node facts (kind/name/offset/len/post-arrow)
;;   → collect-rules :fix → compile → insert every Node → fire-rules
;;   → QUERY OUT :fix::HeadConv / :fix::ArrowConv / :fix::TypeConv (the pure deductions)
;;   → for each, build a span edit (the TRANSFORM lives HERE):
;;       HeadConv → (ast-name (keyword/to-symbol (keyword-node name)))   e.g. wat.core/defrecord
;;       ArrowConv → ":-"
;;       TypeConv → (write-forms (keyword/to-type-form (keyword-node name)))  e.g. wat.type/String
;;   → sort edits right-to-left (descending offset; spans are disjoint) → fix-text-apply → write.
;;
;; Reuses fix.wat's walk primitives (structural?, arrow?, fix-text-offset-of, fix-text-apply).
;; NOTE: source.wat contains no `if`, so the golden's `strip-if` (redundant `-> :T` on an if)
;; is not exercised here; the three conv rules reproduce the source.wat golden byte-identical.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/source.wat"]\n' | cargo wat ./wat-scripts/fixes/to-faithful-clojure-rete.wat
;;
;; Dry-run on a /tmp copy first (MANDATORY):
;;   cp wat/source.wat /tmp/pilot.wat
;;   printf '["/tmp/pilot.wat"]\n' | cargo wat ./wat-scripts/fixes/to-faithful-clojure-rete.wat
;;   diff <(fix-text output) /tmp/pilot.wat   # must be byte-identical

;; ── fact model (mirrors tests/rete/probe_arc300_2_fix_defrule.wat) ───────────
(:wat::core::defrecord :fix::Node
  [kind       <- :wat::core::String
   name       <- :wat::core::String
   offset     <- :wat::core::i64
   len        <- :wat::core::i64
   post-arrow <- :wat::core::bool])

(:wat::core::defrecord :fix::HeadConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64
   name   <- :wat::core::String])

;; arc 282: carries `name` too — the arrow's OWN text ("<-" or "->"), the old-text claim.
(:wat::core::defrecord :fix::ArrowConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64
   name   <- :wat::core::String])

(:wat::core::defrecord :fix::TypeConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64
   name   <- :wat::core::String])

;; ── pure string predicates (used in :where guards) ──────────────────────────
(:wat::core::defn :fix::head-keyword-str?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::string::contains? name "::"))

(:wat::core::defn :fix::type-shaped-keyword-str?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::if (:wat::string::contains? name "<")
                    (:wat::string::contains? name ">")
                    false)
    true
    (:wat::core::if (:wat::string::contains? name "(")
      (:wat::string::contains? name ")")
      false)))

;; ── the rules: each :then is PURE (bindings only, no transform) ──────────────
(:wat::rete::defrule :fix::head-keyword->conv
  :when
  [(:fix::Node
     (?offset     <- :offset)
     (?len        <- :len)
     (?kind       <- :kind)
     (?name       <- :name)
     (?post-arrow <- :post-arrow)
     (:wat::rete::string::= ?kind "keyword"))
   (:wat::rete::where (:fix::head-keyword-str? ?name))
   (:wat::rete::where (:wat::rete::core::not ?post-arrow))
   (:wat::rete::where (:wat::rete::core::not (:fix::type-shaped-keyword-str? ?name)))]
  :then
  [(:fix::HeadConv ?offset ?len ?name)])

(:wat::rete::defrule :fix::arrow->conv
  :when
  [(:fix::Node
     (?offset <- :offset)
     (?len    <- :len)
     (?kind   <- :kind)
     (?name   <- :name)
     (:wat::rete::string::= ?kind "symbol"))
   (:wat::rete::where (:wat::rete::core::or
                        (:wat::rete::string::= ?name "<-")
                        (:wat::rete::string::= ?name "->")))]
  :then
  [(:fix::ArrowConv ?offset ?len ?name)])

(:wat::rete::defrule :fix::type-keyword->conv
  :when
  [(:fix::Node
     (?offset     <- :offset)
     (?len        <- :len)
     (?kind       <- :kind)
     (?name       <- :name)
     (?post-arrow <- :post-arrow)
     (:wat::rete::string::= ?kind "keyword"))
   (:wat::rete::where (:wat::rete::core::or
                        ?post-arrow
                        (:fix::type-shaped-keyword-str? ?name)))]
  :then
  [(:fix::TypeConv ?offset ?len ?name)])

(:wat::rete::defquery :fix::q-HeadConv
  :params []
  :when [(:fix::HeadConv (?offset <- :offset) (?len <- :len) (?name <- :name))])

(:wat::rete::defquery :fix::q-ArrowConv
  :params []
  :when [(:fix::ArrowConv (?offset <- :offset) (?len <- :len) (?name <- :name))])

(:wat::rete::defquery :fix::q-TypeConv
  :params []
  :when [(:fix::TypeConv (?offset <- :offset) (?len <- :len) (?name <- :name))])


;; ── the walk: emit a :fix::Node per keyword/symbol leaf (position-aware) ─────
;; Mirrors :wat::fix::fix-text-seq-edits: threads prev-arrow? across siblings, recurses
;; structural nodes (resetting prev-arrow? to false, exactly as fix-text-struct-edits does).
(:wat::core::defn :fix::collect-nodes-node
  [node        <- :wat::WatAST
   prev-arrow? <- :wat::core::bool
   lines       <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:fix::Node])
  (:wat::core::if (:wat::fix::structural? node)
    (:fix::collect-nodes-seq (:wat::core::ast->children node) false lines)
    (:wat::core::let [kind (:wat::core::ast-kind node)]
      (:wat::core::if (:wat::core::or (:wat::core::= kind "keyword")
                                      (:wat::core::= kind "symbol"))
        (:wat::core::let [name (:wat::core::ast-name node)
                          off  (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                          len  (:wat::string::length name)]
          (:wat::core::Vector :fix::Node (:fix::Node :kind kind :name name :offset off :len len :post-arrow prev-arrow?)))
        (:wat::core::Vector :fix::Node)))))

(:wat::core::defn :fix::collect-nodes-seq
  [items       <- (:wat::core::Vector :- [:wat::WatAST])
   prev-arrow? <- :wat::core::bool
   lines       <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:fix::Node])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :fix::Node)
    (:wat::core::let [h  (:wat::core::first items)
                      tl (:wat::core::rest items)]
      (:wat::core::concat
        (:fix::collect-nodes-node h prev-arrow? lines)
        (:fix::collect-nodes-seq tl (:wat::fix::arrow? h) lines)))))

;; ── stage the facts: fold insert over the Node vector ────────────────────────
(:wat::core::defn :fix::insert-nodes
  [session <- :wat::rete::Session
   nodes   <- (:wat::core::Vector :- [:fix::Node])]
  -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  n <- :fix::Node] -> :wat::rete::Session
      (:wat::rete::insert s n))
    session
    nodes))

;; ── query-out + action: turn each pure conv fact into a span edit (the TRANSFORM) ──
;; HeadConv → (ast-name (keyword/to-symbol (keyword-node name))) — the ::-keyword becomes a symbol.
(:wat::core::defn :fix::head-edits
  [convs <- :wat::core::PersistentVector
   acc   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [a  <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     hc <- :wat::core::PersistentMap]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::let [old-name (:wat::core::Option/expect (:wat::map::get hc "?name") "q-HeadConv: ?name")]
        ;; old-text = ?name directly (arc 282) — NEVER ?len; see wat-scripts/fixes/to-
        ;; faithful-clojure-net.wat's sibling comment for the full reasoning.
        (:wat::core::concat a
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
            (:wat::core::Tuple
              (:wat::core::Option/expect (:wat::map::get hc "?offset") "q-HeadConv: ?offset")
              old-name
              (:wat::core::ast-name (:wat::keyword::to-symbol (:wat::core::keyword-node old-name))))))))
    acc
    convs))

;; ArrowConv → ":-" (the annotation-arrow becomes the faithful bind marker).
(:wat::core::defn :fix::arrow-edits
  [convs <- :wat::core::PersistentVector
   acc   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [a  <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     ac <- :wat::core::PersistentMap]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      ;; old-text = ?name — see q-HeadConv's sibling comment above (arc 282).
      (:wat::core::concat a
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
          (:wat::core::Tuple
            (:wat::core::Option/expect (:wat::map::get ac "?offset") "q-ArrowConv: ?offset")
            (:wat::core::Option/expect (:wat::map::get ac "?name") "q-ArrowConv: ?name")
            ":-"))))
    acc
    convs))

;; TypeConv → (write-forms (keyword/to-type-form (keyword-node name))) — the type-keyword becomes a type form.
(:wat::core::defn :fix::type-edits
  [convs <- :wat::core::PersistentVector
   acc   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [a  <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     tc <- :wat::core::PersistentMap]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::let [old-name (:wat::core::Option/expect (:wat::map::get tc "?name") "q-TypeConv: ?name")]
        ;; old-text = ?name directly (arc 282) — NEVER ?len.
        (:wat::core::concat a
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
            (:wat::core::Tuple
              (:wat::core::Option/expect (:wat::map::get tc "?offset") "q-TypeConv: ?offset")
              old-name
              (:wat::core::write-forms (:wat::keyword::to-type-form (:wat::core::keyword-node old-name))))))))
    acc
    convs))

;; ── convert: the full deduce-then-action pipeline for one source string ──────
(:wat::core::defn :fix::convert
  [src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines   (:wat::string::split src "\n")
                    tree    (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms   (:wat::core::ast->children tree)
                    ;; walk → :fix::Node facts (kind/name/offset/len/post-arrow)
                    nodes   (:fix::collect-nodes-seq forms false lines)
                    ;; PURE rete: deduce the classification facts
                    rules   (:wat::rete::collect-rules :fix)
                    session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
                    staged  (:fix::insert-nodes session nodes)
                    fired   (:wat::rete::fire-rules staged)
                    ;; query out + action (the transform lives here, outside rete)
                    ;; query-by-type-string (colon-free FQDN) is the checked-body idiom — the bare
                    ;; type-name constructor form `query` wants doesn't type-check in a defn body.
                    empty-e (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
                    e1      (:fix::head-edits  (:wat::rete::query fired (:fix::q-HeadConv))  empty-e)
                    e2      (:fix::arrow-edits (:wat::rete::query fired (:fix::q-ArrowConv)) e1)
                    e3      (:fix::type-edits  (:wat::rete::query fired (:fix::q-TypeConv))  e2)
                    ;; sort right-to-left (descending offset; spans disjoint) so splicing is stable
                    sorted  (:wat::core::sort
                              (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                                               b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                                -> :wat::core::bool
                                (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
                              e3)]
    (:wat::fix::fix-text-apply src sorted)))

;; ── drive: read → convert → write, per path (mirrors to-faithful-clojure.wat) ─
(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:fix::convert (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[to-faithful-clojure-rete] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
