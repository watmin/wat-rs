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
;;
;; This covers every position the brief names (after `<-`/`->`; a `:-` type bracket; a `:->`
;; fn-type bracket; a `typealias` body; `extend-type`'s child/target; `derive`'s type args; a
;; record/struct/newtype field type; an enum variant's payload; a bound `[T :< X]`) — (A)/(B)/(C)
;; from the grammar alone, (D)/(E) added once the dry-run diff showed the bare, unwrapped corpus
;; shape (A)/(B)/(C) could not see. A keyword used as DATA never sits adjacent to one of these
;; markers, inside one of these two form heads, or as a typealias's last child (a map key's
;; parent is a Map; a `type-of` argument's preceding sibling is a call head, never one of the
;; markers or `extend-type`/`derive`/`typealias`), so the exclusion the brief asks for ("Not when
;; it is a keyword VALUE") still falls out of the same checks rather than needing a separate one.
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
(:wat::core::defn :t2wt::target-names [] -> (:wat::core::HashSet :- [:wat::core::String])
  (:wat::core::HashSet :- [:wat::core::String]
    ":wat::core::i64" ":wat::core::f64" ":wat::core::u8" ":wat::core::bigint"
    ":wat::core::rational" ":wat::core::char" ":wat::core::String" ":wat::core::bool"
    ":wat::core::keyword" ":wat::core::nil" ":wat::core::Value" ":wat::core::Never"
    ":wat::core::Fn" ":wat::core::Record" ":wat::core::Struct" ":wat::core::Vector"
    ":wat::core::HashMap" ":wat::core::HashSet" ":wat::core::List" ":wat::core::Tuple"
    ":wat::core::PersistentVector" ":wat::core::PersistentMap" ":wat::core::Bytes"
    ":wat::WatAST"))

(:wat::core::defn :t2wt::target-name? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::contains? (:t2wt::target-names) name))

;; a binder/return/bound/fn-type marker: the keyword right after one of these is a type.
;; `:-` is IN this set too (not just the head-of-bracket marker rule A checks for) — the
;; kwargs-style binder/return annotation spells the SAME "has type" relation as `<-`/`->`
;; with `:-` instead (measured: `tests/resolve/probe_arc251_stone4_annotation_arrow.wat`'s
;; `[x :- :wat::core::i64] :- :wat::core::i64`, `tests/types/probe_arc255_66_position.wat`'s
;; `[a :- :wat::core::i64 b :- wat.type/i64]`). Safe alongside rule (A)/(B): inside the
;; `(Head :- [args])` idiom, the token right after `:-` is always the args VECTOR, never a
;; bare keyword, so this addition is inert there and only activates for the kwargs shape.
(:wat::core::defn :t2wt::marker-symbol? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= name "<-") true
    (:wat::core::if (:wat::core::= name "->") true
      (:wat::core::if (:wat::core::= name ":->") true
        (:wat::core::if (:wat::core::= name ":<") true
          (:wat::core::= name ":-"))))))

;; item-name — the ast-name of a symbol/keyword sibling; "" for anything else (never a marker,
;; never a target — an empty string can't collide with a real name).
(:wat::core::defn :t2wt::item-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [k (:wat::core::ast-kind n)]
    (:wat::core::if (:wat::core::or (:wat::core::= k "keyword") (:wat::core::= k "symbol"))
      (:wat::core::ast-name n)
      "")))

;; conv-edit — the one-element edit Vector converting a genuine target-keyword leaf.
(:wat::core::defn :t2wt::conv-edit
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let [off      (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                    old-name (:wat::core::ast-name node)
                    new-text (:wat::core::write-forms (:wat::keyword::to-type-form node))]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple off old-name new-text))))

;; (D) known BARE-type-taking form heads — a keyword sitting directly after these, with no
;; marker and no `:-` wrapper, is still a type: `(extend-type Child Target)`, `(derive Child
;; Parent)` both take a bare type in EVERY non-head position (measured: `wat/class.wat`'s 20+
;; `(extend-type :wat::core::i64 :wat::core::Equatable)`-shaped leaf-Equatable/Orderable rows —
;; a bare, unwrapped, marker-free monomorphic type is real corpus shape, not a hypothetical).
(:wat::core::defn :t2wt::all-children-are-types-form? [head-name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= head-name ":wat::core::extend-type") true
    (:wat::core::= head-name ":wat::core::derive")))

;; (E) `typealias` — ONLY its last child (the body) is a type; `Name` (the child right after
;; the head) is the name being DECLARED, never a type, so it must NOT ride rule (D)'s uniform
;; eligibility. Measured: `(typealias :my::Coord :wat::core::i64)` — a bare, unwrapped body is
;; real corpus shape (10+ hits), not hypothetical.
(:wat::core::defn :t2wt::only-last-child-is-type-form? [head-name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::= head-name ":wat::core::typealias"))

;; node-edits — one node's contribution: recurse if structural (this node becomes an
;; args-vector for ITS OWN children iff `prev-name` — OUR OWN preceding sibling in the
;; sequence that holds us — is exactly `:-`); else, a leaf keyword converts iff it is a
;; genuine target AND (A) next sibling is `:-`, OR (B) `in-args?`, OR (C) `prev-name` is a
;; marker, OR (D)/(E) `last-eligible?` (the enclosing form is `typealias` and this IS its last
;; child, or the enclosing form is `extend-type`/`derive` — which `in-args?` already covers,
;; since the whole child sequence is flagged, so (D) needs no separate leaf-side check).
(:wat::core::defn :t2wt::node-edits
  [node          <- :wat::WatAST
   in-args?      <- :wat::core::bool
   last-eligible? <- :wat::core::bool
   prev-name     <- :wat::core::String
   next-name     <- :wat::core::String
   lines         <- (:wat::core::Vector :- [:wat::core::String])
   src           <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch          (:wat::core::ast->children node)
                      head-name   (:wat::core::if (:wat::core::empty? ch) "" (:t2wt::item-name (:wat::core::first ch)))
                      child-in-args? (:wat::core::if (:wat::core::= prev-name ":-")
                                       true
                                       (:t2wt::all-children-are-types-form? head-name))
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
                  (:wat::fix::empty-edits)))))
          ;; reader-synthesized (span text ≠ ast-name) — never convert.
          (:wat::fix::empty-edits))
        (:wat::fix::empty-edits))
      (:wat::fix::empty-edits))))

;; walk-seq — left-to-right over one sibling sequence. `in-args?` is a property of THIS WHOLE
;; sequence (constant across it, decided by our caller before descending); `only-last?` marks
;; that ONLY the final item in this sequence gets the (E) eligibility; `prev-name` threads
;; left-to-right within it, starting at "" (no marker before the first item).
(:wat::core::defn :t2wt::walk-seq
  [items      <- (:wat::core::Vector :- [:wat::WatAST])
   in-args?   <- :wat::core::bool
   only-last? <- :wat::core::bool
   prev-name  <- :wat::core::String
   lines      <- (:wat::core::Vector :- [:wat::core::String])
   src        <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
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
  [src <- :wat::core::String]
  -> :wat::core::String
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
              (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                               b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                -> :wat::core::bool
                (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
              edits)]
    (:wat::fix::fix-text-apply src sorted)))

;; ══ DRIVE — read → convert → write, per path ═════════════════════════════════════════════════
(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:t2wt::convert (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[types-to-wat-type] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
