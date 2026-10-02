;; wat-scripts/fixes/types-to-wat-type.wat — arc 255, STONE 255.67 (cutover 2 of 7).
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-255.67-cutover-2-types.md
;; RULING: docs/arc/2026/06/255-builtin-registry/FINDING-the-shape-of-a-declared-signature.md
;;         § "`wat.type/` is closed" (2026-09-27) — exactly 24 names.
;;
;; Converts a keyword `:wat::core::<one of the 24 hard primitives>` (or `:wat::WatAST`) to the
;; symbol `wat.type/<name>` (`wat.type/AST` for WatAST) **only when it sits in a TYPE POSITION**:
;;
;;   the 24: i64 f64 u8 bigint rational char String bool keyword nil Value Never Fn Record
;;           Struct Vector HashMap HashSet List Tuple PersistentVector PersistentMap Bytes AST
;;
;; Everything else — function/form heads, `Option`/`Result`/`Span`/…, the slash-verbs
;; (`Vector/get`), a keyword used as DATA (a map key, `type-of`'s argument, a quoted example
;; compared at runtime) — is untouched. Exact-string match on the WHOLE keyword token is what
;; keeps this precise.
;;
;; ══ THE POSITION RULE, reduced to five LOCAL, non-recursive checks ═══════════════════════════
;;
;; Every type expression in this dialect is one of: a bare keyword, a parametric form
;; `(Head :- [args])`, or a fn-type bracket `[A B :-> R]` — and these nest arbitrarily. Because
;; `(Head :- [args])` means the SAME thing (constructor and type both dispatch through the same
;; door, stone 255.66) wherever it appears, a keyword's type-ness is decidable from its
;; IMMEDIATE parent/sibling relationship alone — no ancestor walk, no fixpoint, ONE left-to-right
;; pass over each sibling sequence (mirrors `wat/fix.wat`'s own `fix-seq`, which threads
;; `prev-arrow?` the same way for the full faithful-Clojure conversion — this migration threads
;; a NARROWER version scoped to just the 24 names):
;;
;;   (A) HEAD-OF-BRACKET  — the keyword is immediately followed (next sibling) by the symbol
;;       `:-`.  `(Vector :- […])` — Vector converts, in ANY position (stone 1's ruling).
;;   (B) ARGS-VECTOR MEMBER — the keyword sits directly inside a sequence that is ITSELF
;;       immediately preceded (in ITS OWN parent) by `:-`. Recurses "for free": a nested
;;       `(HashMap :- […])` inside an args vector independently satisfies (A) for its own head
;;       and (B) for its own args, at whatever depth, because each `(Head :- [args])` list makes
;;       this call fresh — no state threaded across levels for it.
;;   (C) POST-MARKER — the keyword immediately follows (previous sibling) one of the
;;       binder/return/bound/fn-type markers `<- -> :-> :<`.
;;   (D) BARE CHILD OF `extend-type` / `derive` — measured against the corpus (not
;;       hypothesized): `wat/class.wat` alone has 20+ `(extend-type :wat::core::i64
;;       :wat::core::Equatable)`-shaped rows — a monomorphic type with NO wrapping bracket and
;;       NO preceding marker, sitting bare right after the form head. Every non-head child of
;;       one of these two heads is a type (the brief's "extend-type's child and target",
;;       "derive's type arguments").
;;   (E) LAST CHILD OF `typealias` — ALSO measured (10+ corpus hits): `(typealias :my::Coord
;;       :wat::core::i64)`, a bare body with no wrapping bracket. Unlike (D), typealias's
;;       child right after the head is the NAME being declared (never a type) — only the LAST
;;       child (the body) is eligible.
;;   (F) NATURE-VALUE — added stone 255.79 (4a), a follow-up to stone 2's recorded residue
;;       ("`:nature :wat::core::Struct` — a type position the rules missed", 79 sites at the
;;       time). The keyword immediately follows (previous sibling) the bare keyword `:nature`
;;       in a `defsurface`'s `:nature <type>` clause: `(defsurface :probe::Store :nature
;;       :wat::core::Struct …)` (measured corpus-wide at this stone's draw: 89 sites, 55
;;       `:nature :wat::core::Struct` + 34 `:nature :wat::core::Record`). `:nature` itself has
;;       no wrapping bracket and no `<-`/`->`/`:->`/`:<` before its value, so (C) alone cannot
;;       see it — this rule reads the same way, by previous-sibling identity, over the one
;;       extra keyword `:nature`. A `:nature` value outside the 24 (`:wat::kernel::Peer`, a
;;       service surface) is already excluded by `target-name?`, so this rule adds no false
;;       positive: it only ever fires on a genuine target keyword.
;;
;; This covers every position the brief names (after `<-`/`->`; a `:-` type bracket; a `:->`
;; fn-type bracket; a `typealias` body; `extend-type`'s child/target; `derive`'s type args; a
;; record/struct/newtype field type; an enum variant's payload; a bound `[T :< X]`; a
;; `defsurface`'s `:nature` value) — (A)/(B)/(C) from the grammar alone, (D)/(E)/(F) added once
;; the dry-run diff (and, for (F), the stone 2 residue table) showed the bare, unwrapped corpus
;; shape (A)/(B)/(C) could not see. A keyword used as DATA never sits adjacent to one of these
;; markers, inside one of these two form heads, right after `:nature`, or as a typealias's last
;; child (a map key's parent is a Map; a `type-of` argument's preceding sibling is a call head,
;; never one of the markers or `extend-type`/`derive`/`typealias`/`:nature`), so the exclusion
;; the brief asks for ("Not when it is a keyword VALUE") still falls out of the same checks
;; rather than needing a separate one.
;;
;; ══ WHY A PLAIN WALK, NOT wat/grep.wat's RETE FACT BASE ══════════════════════════════════════
;; A first attempt modeled this on `to-faithful-clojure-net.wat`'s offset-identity `:fix::Node`
;; facts + self-joins (parent/child-idx equality) — proven for THAT file's unconditional
;; conversion, where every `Node` fact type is a LEAF. Extending it here to also fact-ify every
;; STRUCTURAL node (needed for the args-vector-root join) blew up: `wat/core.wat` (2,329 lines)
;; ran for 4+ minutes with the network pegged at 100% CPU while `cache.wat` (440 lines) and
;; `class.wat` (93 lines) each finished in under a second — non-linear, not merely slow. Killed
;; before it finished; not diagnosed further (not this stone's tool to fix), and not reused.
;; Every fact this migration needs is a LOCAL sibling/parent relationship a single left-to-right
;; pass already has in hand while walking — the same shape `fix.wat`'s own `fix-seq`/
;; `fix-text-leaf-edits` use, at O(n) — so there was never a reason to route through rete here.
;;
;; The replacement text is produced by the ONE canonical door, `:wat::keyword::to-type-form`
;; (special-cases `:wat::WatAST` → `wat.type/AST`; every other `wat.type/` tail is the identity,
;; arc 251.2) — not a hand-duplicated 24-row table — so this codemod can never drift from what
;; the checker itself calls "the same head".
;;
;; Genuineness guard (`fix.wat`'s `source-matches-name?`, span text == ast-name) skips a
;; reader-synthesized keyword leaf (arc 251.8d-i class B) so a desugared sigil is never mistaken
;; for one of the 24.
;;
;; Usage — dry-run on a /tmp copy first, diff, THEN apply (list EVERY path):
;;   printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/types-to-wat-type.wat
;;
;; Idempotent: a converted file has no keyword left matching the 24 target strings, so a re-run
;; finds nothing to convert.

;; the 24 exact FQDN spellings (WatAST included) this codemod may touch.
(:wat::core::defn :t2wt::target-names [] -> (wat.type/HashSet :- [wat.type/String])
  (wat.type/HashSet :- [wat.type/String]
    ":wat::core::i64" ":wat::core::f64" ":wat::core::u8" ":wat::core::bigint"
    ":wat::core::rational" ":wat::core::char" ":wat::core::String" ":wat::core::bool"
    ":wat::core::keyword" ":wat::core::nil" ":wat::core::Value" ":wat::core::Never"
    ":wat::core::Fn" ":wat::core::Record" ":wat::core::Struct" ":wat::core::Vector"
    ":wat::core::HashMap" ":wat::core::HashSet" ":wat::core::List" ":wat::core::Tuple"
    ":wat::core::PersistentVector" ":wat::core::PersistentMap" ":wat::core::Bytes"
    ":wat::WatAST"))

(:wat::core::defn :t2wt::target-name? [name <- wat.type/String] -> wat.type/bool
  (:wat::core::contains? (:t2wt::target-names) name))

;; a binder/return/bound/fn-type marker: the keyword right after one of these is a type.
;; `:-` is IN this set too (not just the head-of-bracket marker rule A checks for) — the
;; kwargs-style binder/return annotation spells the SAME "has type" relation as `<-`/`->`
;; with `:-` instead (measured: `tests/resolve/probe_arc251_stone4_annotation_arrow.wat`'s
;; `[x :- :wat::core::i64] :- :wat::core::i64`, `tests/types/probe_arc255_66_position.wat`'s
;; `[a :- :wat::core::i64 b :- wat.type/i64]`). Safe alongside rule (A)/(B): inside the
;; `(Head :- [args])` idiom, the token right after `:-` is always the args VECTOR, never a
;; bare keyword, so this addition is inert there and only activates for the kwargs shape.
(:wat::core::defn :t2wt::marker-symbol? [name <- wat.type/String] -> wat.type/bool
  (:wat::core::if (:wat::core::= name "<-") true
    (:wat::core::if (:wat::core::= name "->") true
      (:wat::core::if (:wat::core::= name ":->") true
        (:wat::core::if (:wat::core::= name ":<") true
          (:wat::core::= name ":-"))))))

;; item-name — the ast-name of a symbol/keyword sibling; "" for anything else (never a marker,
;; never a target — an empty string can't collide with a real name).
(:wat::core::defn :t2wt::item-name [n <- wat.type/AST] -> wat.type/String
  (:wat::core::let [k (:wat::core::ast-kind n)]
    (:wat::core::if (:wat::core::or (:wat::core::= k "keyword") (:wat::core::= k "symbol"))
      (:wat::core::ast-name n)
      "")))

;; conv-edit — the one-element edit Vector converting a genuine target-keyword leaf.
(:wat::core::defn :t2wt::conv-edit
  [node  <- wat.type/AST
   lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let [off      (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                    old-name (:wat::core::ast-name node)
                    new-text (:wat::core::write-forms (:wat::keyword::to-type-form node))]
    (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] off old-name new-text))))

;; (D) known BARE-type-taking form heads — a keyword sitting directly after these, with no
;; marker and no `:-` wrapper, is still a type: `(extend-type Child Target)`, `(derive Child
;; Parent)` both take a bare type in EVERY non-head position (measured: `wat/class.wat`'s 20+
;; `(extend-type :wat::core::i64 :wat::core::Equatable)`-shaped leaf-Equatable/Orderable rows —
;; a bare, unwrapped, marker-free monomorphic type is real corpus shape, not a hypothetical).
;; Stone 255.81 — widened to four more ALL-bare-children heads, each measured the same way (a
;; `wat --check`-clean corpus program naming a real type, not a hypothesized shape):
;; `subtype?`'s two args (`(subtype? Child Parent)`), and the three single-arg reflection verbs
;; `is-type?`/`type-of`/`signature-of-defn` (their one arg IS the type being asked about).
;; Also `recordtype`/`aggregatetype`: `(recordtype :name [:- binder] Parent [fields])` — the
;; PARENT is a bare type, sitting among otherwise-never-target-name children (`:name` is a
;; user name, the optional binder's vars are type-PARAMETER names, `[fields]` is itself
;; structural and walks its OWN field-type positions through the existing marker rules) —
;; so flagging the WHOLE sequence in-args is inert on every position except the genuine
;; target-name one. Measured: `tests/types/probe_arc237_sB1_recordtype.wat`,
;; `tests/types/probe_arc293_decl_a_aggregatetype.wat`, and `src/types.rs`'s own
;; `expand_then_register` fixtures (8+ `(recordtype … :wat::core::Record […])` rows).
(:wat::core::defn :t2wt::all-children-are-types-form? [head-name <- wat.type/String] -> wat.type/bool
  (:wat::core::if (:wat::core::= head-name ":wat::core::extend-type") true
    (:wat::core::if (:wat::core::= head-name ":wat::core::derive") true
      (:wat::core::if (:wat::core::= head-name ":wat::core::subtype?") true
        (:wat::core::if (:wat::core::= head-name ":wat::runtime::is-type?") true
          (:wat::core::if (:wat::core::= head-name ":wat::runtime::type-of") true
            (:wat::core::if (:wat::core::= head-name ":wat::runtime::signature-of-defn") true
              (:wat::core::if (:wat::core::= head-name ":wat::core::recordtype") true
                (:wat::core::= head-name ":wat::core::aggregatetype")))))))))

;; (E) `typealias` — ONLY its last child (the body) is a type; `Name` (the child right after
;; the head) is the name being DECLARED, never a type, so it must NOT ride rule (D)'s uniform
;; eligibility. Measured: `(typealias :my::Coord :wat::core::i64)` — a bare, unwrapped body is
;; real corpus shape (10+ hits), not hypothetical.
;; Stone 255.81 — widened to four more LAST-child-only heads, each the SAME shape (a leading
;; non-type argument, then the type): `newtype`'s inner type (`(newtype Name InnerType)` — the
;; same NAME-then-TYPE shape as typealias, measured in `tests/types/probe_arc255_55_newtype.wat`
;; and 14 siblings); `conforms?`/`ann-form`'s second arg (`(conforms? value Type)`,
;; `(ann-form value Type)` — a VALUE then the type it's checked/annotated against); and
;; `:wat::edn::validate`'s second arg (`(validate value Type)`, same shape).
(:wat::core::defn :t2wt::only-last-child-is-type-form? [head-name <- wat.type/String] -> wat.type/bool
  (:wat::core::if (:wat::core::= head-name ":wat::core::typealias") true
    (:wat::core::if (:wat::core::= head-name ":wat::core::newtype") true
      (:wat::core::if (:wat::core::= head-name ":wat::core::conforms?") true
        (:wat::core::if (:wat::core::= head-name ":wat::core::ann-form") true
          (:wat::core::if (:wat::core::= head-name ":wat::edn::validate") true
            ;; (H) `typeunion`'s last child is a VECTOR of member types
            ;; (`(typeunion :name [T1 T2 …])`), not a bare type itself — this predicate still
            ;; marks the VECTOR eligible; `node-edits`' structural branch (below) is what
            ;; then treats a last-eligible VECTOR's own children as in-args, the one place
            ;; this rule's shape differs from the other four (a bare leaf, not a container).
            (:wat::core::= head-name ":wat::core::typeunion")))))))

;; (F) NATURE-VALUE — stone 255.79's added rule. A keyword whose immediately preceding sibling
;; is the bare keyword `:nature` (a `defsurface`'s `:nature <type>` clause) is a type position,
;; exactly as `:- :->` etc. are for rule (C) — but `:nature` is not one of those markers, so it
;; needs its own check. `target-name?` already excludes any `:nature` value outside the 24
;; (`:wat::kernel::Peer`), so this predicate only needs to ask "was my previous sibling exactly
;; `:nature`" — it can never fire on a bare data keyword, since a data keyword's previous
;; sibling is never the literal token `:nature`.
(:wat::core::defn :t2wt::nature-value? [name <- wat.type/String] -> wat.type/bool
  (:wat::core::= name ":nature"))

;; (G) FN-BRACKET ARGS — stone 255.81 (cutover 4b), a residue rule (B)-(F) do not see.
;; `[A1 A2 … :-> R]` is the fn-type bracket (`[arg… :-> ret]`, arc 251.4c) — the RETURN `R`
;; already converts via rule (C) (`R`'s previous sibling is literally `:->`, already in
;; `marker-symbol?`'s set), but an ARG before the arrow (`A1`, `A2`, …) has NO preceding
;; marker (the arrow comes AFTER it, not before) and is not itself head-of-a-`:-`-bracket — so
;; rules (A)-(F) all miss it. Measured, not hypothesized: `wat/bracket.wat`/`wat/gen.wat`/
;; `wat/seq.wat` each still carry a bare `:wat::core::i64` as a multi-arg bracket's FIRST
;; argument, confirmed by the idempotent codemod's own 0-change dry run finding nothing there —
;; the existing five rules structurally cannot reach it. A bracket Vector's children are a
;; type position in their OWN right (same relation `(Head :- [args])`'s args-vector members
;; already get via rule (B)) the MOMENT any child is literally `:->` — so this is evaluated once
;; per structural node, over ITS OWN children, not threaded from a parent/sibling relationship.
(:wat::core::defn :t2wt::seq-is-fn-bracket? [ch <- (wat.type/Vector :- [wat.type/AST])] -> wat.type/bool
  (:wat::core::if (:wat::core::empty? ch) false
    (:wat::core::if (:wat::core::= (:t2wt::item-name (:wat::core::first ch)) ":->") true
      (:t2wt::seq-is-fn-bracket? (:wat::core::rest ch)))))

;; node-edits — one node's contribution: recurse if structural (this node becomes an
;; args-vector for ITS OWN children iff `prev-name` — OUR OWN preceding sibling in the
;; sequence that holds us — is exactly `:-`); else, a leaf keyword converts iff it is a
;; genuine target AND (A) next sibling is `:-`, OR (B) `in-args?`, OR (C) `prev-name` is a
;; marker, OR (D)/(E) `last-eligible?` (the enclosing form is `typealias` and this IS its last
;; child, or the enclosing form is `extend-type`/`derive` — which `in-args?` already covers,
;; since the whole child sequence is flagged, so (D) needs no separate leaf-side check), OR
;; (F) `prev-name` is exactly `:nature`.
(:wat::core::defn :t2wt::node-edits
  [node          <- wat.type/AST
   in-args?      <- wat.type/bool
   last-eligible? <- wat.type/bool
   prev-name     <- wat.type/String
   next-name     <- wat.type/String
   lines         <- (wat.type/Vector :- [wat.type/String])
   src           <- wat.type/String]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch          (:wat::core::ast->children node)
                      head-name   (:wat::core::if (:wat::core::empty? ch) "" (:t2wt::item-name (:wat::core::first ch)))
                      child-in-args? (:wat::core::if (:wat::core::= prev-name ":-")
                                       true
                                       (:wat::core::if (:t2wt::all-children-are-types-form? head-name)
                                         true
                                         (:wat::core::if
                                           ;; (G) — this node's OWN children are a fn-bracket
                                           ;; sequence `[A… :-> R]` iff ANY of them is literally
                                           ;; `:->`; every non-arrow child (arg or ret) is then
                                           ;; a type position. Checked over ch itself (not
                                           ;; prev-name, not head-name) — the one rule keyed on
                                           ;; the SEQUENCE's own contents rather than its
                                           ;; relation to a parent/sibling.
                                           (:t2wt::seq-is-fn-bracket? ch)
                                           true
                                           ;; (H) — THIS node arrived already marked
                                           ;; `last-eligible?` (rule E said so — e.g. it is
                                           ;; `typeunion`'s last child) AND it is itself a
                                           ;; Vector (a member-type LIST, not a bare type) —
                                           ;; so ITS children are the types, not it.
                                           (:wat::core::and last-eligible?
                                             (:wat::core::= (:wat::core::ast-kind node) "vector")))))
                      child-only-last? (:t2wt::only-last-child-is-type-form? head-name)]
      (:t2wt::walk-seq ch child-in-args? child-only-last? "" lines src))
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::if (:t2wt::target-name? (:wat::core::ast-name node))
        (:wat::core::if (:wat::fix::source-matches-name? node lines src)
          (:wat::core::if (:wat::core::= next-name ":-")
            (:t2wt::conv-edit node lines)
            (:wat::core::if in-args?
              (:t2wt::conv-edit node lines)
              (:wat::core::if (:t2wt::marker-symbol? prev-name)
                (:t2wt::conv-edit node lines)
                (:wat::core::if last-eligible?
                  (:t2wt::conv-edit node lines)
                  (:wat::core::if (:t2wt::nature-value? prev-name)
                    (:t2wt::conv-edit node lines)
                    (:wat::fix::empty-edits))))))
          ;; reader-synthesized (span text ≠ ast-name) — never convert.
          (:wat::fix::empty-edits))
        (:wat::fix::empty-edits))
      (:wat::fix::empty-edits))))

;; walk-seq — left-to-right over one sibling sequence. `in-args?` is a property of THIS WHOLE
;; sequence (constant across it, decided by our caller before descending); `only-last?` marks
;; that ONLY the final item in this sequence gets the (E) eligibility; `prev-name` threads
;; left-to-right within it, starting at "" (no marker before the first item).
(:wat::core::defn :t2wt::walk-seq
  [items      <- (wat.type/Vector :- [wat.type/AST])
   in-args?   <- wat.type/bool
   only-last? <- wat.type/bool
   prev-name  <- wat.type/String
   lines      <- (wat.type/Vector :- [wat.type/String])
   src        <- wat.type/String]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::fix::empty-edits)
    (:wat::core::let [h            (:wat::core::first items)
                      tl           (:wat::core::rest items)
                      is-last?     (:wat::core::empty? tl)
                      next-name    (:wat::core::if is-last?
                                     ""
                                     (:t2wt::item-name (:wat::core::first tl)))
                      last-eligible? (:wat::core::if only-last? is-last? false)
                      h-edits      (:t2wt::node-edits h in-args? last-eligible? prev-name next-name lines src)
                      h-name       (:t2wt::item-name h)]
      (:wat::core::concat h-edits
        (:t2wt::walk-seq tl in-args? only-last? h-name lines src)))))

;; convert — one file's source text, fully converted.
(:wat::core::defn :t2wt::convert
  [src <- wat.type/String]
  -> wat.type/String
  (:wat::core::let
    [lines  (:wat::string::split src "\n")
     tree   (:wat::core::match (:wat::core::read-string src)
              [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
              [:wat::core::ReadOutcome.Malformed {:cause __cause}
                (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
     forms  (:wat::core::ast->children tree)
     ;; top-level forms are never in-args and have no preceding sibling.
     edits  (:t2wt::walk-seq forms false false "" lines src)
     sorted (:wat::core::sort
              (:wat::core::fn [a <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])
                               b <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
                -> wat.type/bool
                (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
              edits)]
    (:wat::fix::fix-text-apply src sorted)))

;; ══ DRIVE — read → convert → write, per path ═════════════════════════════════════════════════
(:wat::core::defn :user::apply-each
  [paths <- (wat.type/Vector :- [wat.type/String])] -> wat.type/nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:t2wt::convert (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[types-to-wat-type] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
