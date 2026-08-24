;; wat/grep.wat — wat-grep's vocabulary: the fact base it inserts, per file, and the ONE query.
;;
;; DESIGN: docs/arc/2026/06/278-rules-engine/DESIGN-STONE-wat-grep-is-a-feature.md — the contract,
;; the file's shape, and why each name is what it is. THE CONTRACT, verbatim:
;;
;;   wat-grep DECLARES   Node · Named · Span      the fact base it inserts, per file
;;                       Match · Capture          what a rule asserts
;;                       q-match                  the one query — never written by a user
;;   the USER DECLARES   rules over those, asserting Match
;;   wat-grep OWNS       compile · the lease · the loop · the reset · the query · the print
;;
;; This is a MOVE of proven code from
;; wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat (shipped numbers: `5d650b807`,
;; wat/fix.wat Node=4316 Span=4316, neg-consumer 435, probe_do_splice 33). The migration is
;; `:fx::` → `:wat::grep::`, `:fx::Acc` becomes `:wat::grep::Facts`, and the four inline
;; `Option/expect` + `HashMap/get` unwraps of an `ast-span`/`ast-end-span` HashMap collapse into
;; one door, `extent-of` — after this file, no other site anywhere unwraps a span.
;;
;; ─── THE ONE DESIGN DECISION (carried from corpus-03, unchanged) ─────────────
;; Node identity is assigned by a PRE-ORDER walk with a threaded counter. Pre-order is chosen
;; because it makes `parent` always ALREADY ASSIGNED when a child is numbered — a post-order walk
;; would force a second pass to back-fill parents.
;;
;; ─── AND THE GUARD (corpus 01's, now load-bearing for real) ──────────────────
;; `ast-name` is PARTIAL — it raises on any node that is not Symbol/Keyword/StringLit. The guard
;; is structural instead: a `Named` fact is emitted ONLY for a nameable kind, so an unnameable
;; node simply HAS NO NAME FACT and every downstream rule that joins `Named` cannot see it. The
;; absence IS the guard.
;;
;; ★ NON-VACUITY: `Node` must exceed `Named` on any real file. `Span` is emitted UNCONDITIONALLY
;; beside `Node` (unlike `Named`) because `ast-span`/`ast-end-span` are TOTAL — Span == Node is
;; the non-vacuity control for that half.

;; ── the fact base wat-grep inserts, one set per file ────────────────────────────────

(:wat::core::defrecord :wat::grep::Node
  [id     <- :wat::core::i64
   parent <- :wat::core::i64
   index  <- :wat::core::i64
   kind   <- :wat::core::String])

;; ONLY for a nameable kind — the absence IS the guard.
(:wat::core::defrecord :wat::grep::Named
  [id   <- :wat::core::i64
   name <- :wat::core::String])

;; EVERY node — Span == Node is the non-vacuity control. Flat, not nested, and NOT
;; :wat::core::Span: `ast-span` returns keyword->i64 so it cannot carry :file; the file is a
;; property of the RUN, not of a node; and a rule binds FIELDS, not sub-records.
;; ⚠ CROSS-REFERENCE: this field list (line/col/end-line/end-col) must stay in lockstep with
;; :wat::grep::Extent's four fields below — nothing pins them together, so a rename of one must
;; be made in both by hand.
(:wat::core::defrecord :wat::grep::Span
  [id       <- :wat::core::i64
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

;; ── what a rule asserts ─────────────────────────────────────────────────────────────

(:wat::core::defrecord :wat::grep::Capture
  [name  <- :wat::core::String
   value <- :wat::core::String])

;; FLAT — no nested Extent — because a rule binds FIELDS, not sub-records.
(:wat::core::defrecord :wat::grep::Match
  [file     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64
   rule     <- :wat::core::String
   captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])])

;; ── the in-process coordinate, and THE ONE DOOR ─────────────────────────────────────

(:wat::core::defrecord :wat::grep::Extent
  [line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

;; extent-of — the ONLY site that unwraps an ast-span/ast-end-span HashMap. Mirrors
;; wat/fix.wat's fix-text-offset-of shape. After this fn exists, nothing else anywhere unwraps a
;; span — that is what the name promises.
(:wat::core::defn :wat::grep::extent-of
  [node <- :wat::WatAST]
  -> :wat::grep::Extent
  (:wat::core::let [sp (:wat::core::ast-span node)
                    ep (:wat::core::ast-end-span node)]
    (:wat::grep::Extent
      :line     (:wat::core::Option/expect (:wat::core::HashMap/get sp :line) "extent-of: :line")
      :col      (:wat::core::Option/expect (:wat::core::HashMap/get sp :col)  "extent-of: :col")
      :end-line (:wat::core::Option/expect (:wat::core::HashMap/get ep :line) "extent-of: :end-line")
      :end-col  (:wat::core::Option/expect (:wat::core::HashMap/get ep :col)  "extent-of: :end-col"))))

;; ── source → facts ──────────────────────────────────────────────────────────────────

(:wat::core::defrecord :wat::grep::Facts
  [nodes <- (:wat::core::PersistentVector :- [:wat::grep::Node])
   named <- (:wat::core::PersistentVector :- [:wat::grep::Named])
   spans <- (:wat::core::PersistentVector :- [:wat::grep::Span])])

;; ── internal walk plumbing (not part of the wat-grep contract; the walk's threading) ────
;; Moved verbatim from corpus-03's :fx::Acc / :fx::ChildAcc, renamed.

(:wat::core::defrecord :wat::grep::Acc
  [next-id <- :wat::core::i64
   nodes   <- (:wat::core::PersistentVector :- [:wat::grep::Node])
   named   <- (:wat::core::PersistentVector :- [:wat::grep::Named])
   spans   <- (:wat::core::PersistentVector :- [:wat::grep::Span])])

;; per-level child accumulator: the walk's Acc plus this level's running index
(:wat::core::defrecord :wat::grep::ChildAcc
  [acc <- :wat::grep::Acc
   idx <- :wat::core::i64])

;; nameable? — the TOTAL guard in front of the partial `ast-name`.
(:wat::core::defn :wat::grep::nameable?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::if (:wat::core::= k "symbol") true
      (:wat::core::if (:wat::core::= k "keyword") true
        (:wat::core::= k "string")))))

;; structural? — does this node HAVE children to descend into?
(:wat::core::defn :wat::grep::structural?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::contains?
      (:wat::core::HashSet :wat::type::Infer "list" "vector" "map" "set") k)))

;; walk — assign this node an id, emit its facts, then descend. Pre-order, so `parent` is always
;; already numbered when a child is reached.
(:wat::core::defn :wat::grep::walk
  [acc    <- :wat::grep::Acc
   node   <- :wat::WatAST
   parent <- :wat::core::i64
   index  <- :wat::core::i64]
  -> :wat::grep::Acc
  (:wat::core::let
    [id    (:wat::grep::Acc/next-id acc)
     kind  (:wat::core::ast-kind node)
     nodes (:wat::core::PersistentVector/conj (:wat::grep::Acc/nodes acc)
             (:wat::grep::Node :id id :parent parent :index index :kind kind))
     ;; THE GUARD: no name fact for an unnameable node. `ast-name` is never reached for one.
     named (:wat::core::if (:wat::grep::nameable? node)
             (:wat::core::PersistentVector/conj (:wat::grep::Acc/named acc)
               (:wat::grep::Named :id id :name (:wat::core::ast-name node)))
             (:wat::grep::Acc/named acc))
     ;; NO GUARD: extent-of is total (ast-span / ast-end-span are total). Every node gets a Span.
     ex    (:wat::grep::extent-of node)
     spans (:wat::core::PersistentVector/conj (:wat::grep::Acc/spans acc)
             (:wat::grep::Span :id id
                        :line     (:wat::grep::Extent/line ex)
                        :col      (:wat::grep::Extent/col ex)
                        :end-line (:wat::grep::Extent/end-line ex)
                        :end-col  (:wat::grep::Extent/end-col ex)))
     acc'  (:wat::grep::Acc :next-id (:wat::core::i64::+ id 1) :nodes nodes :named named :spans spans)]
    (:wat::core::if (:wat::grep::structural? node)
      (:wat::grep::ChildAcc/acc
        (:wat::core::foldl
          (:wat::core::fn [ca <- :wat::grep::ChildAcc  child <- :wat::WatAST] -> :wat::grep::ChildAcc
            (:wat::grep::ChildAcc
              :acc (:wat::grep::walk (:wat::grep::ChildAcc/acc ca) child id (:wat::grep::ChildAcc/idx ca))
              :idx (:wat::core::i64::+ (:wat::grep::ChildAcc/idx ca) 1)))
          (:wat::grep::ChildAcc :acc acc' :idx 0)
          (:wat::core::ast->children node)))
      acc')))

(:wat::core::defn :wat::grep::empty-acc [] -> :wat::grep::Acc
  (:wat::grep::Acc :next-id 1
            :nodes (:wat::core::PersistentVector)
            :named (:wat::core::PersistentVector)
            :spans (:wat::core::PersistentVector)))

;; facts-of — every top-level form of one source string, walked into one fact base.
;;
;; `read-string` returns a FACED OUTCOME, not a bare vector — the no-hidden-failures law. A
;; string that will not parse is a RESULT the extractor carries, never a crash: Malformed yields
;; an EMPTY fact base, which every downstream rule reads as "nothing to say about this file".
(:wat::core::defn :wat::grep::facts-of
  [src <- :wat::core::String]
  -> :wat::grep::Facts
  (:wat::core::let
    [acc (:wat::core::match (:wat::core::read-string src)
           ((:wat::core::ReadOutcome::Forms forms)
             (:wat::grep::ChildAcc/acc
               (:wat::core::foldl
                 (:wat::core::fn [ca <- :wat::grep::ChildAcc  form <- :wat::WatAST] -> :wat::grep::ChildAcc
                   (:wat::grep::ChildAcc
                     :acc (:wat::grep::walk (:wat::grep::ChildAcc/acc ca) form 0 (:wat::grep::ChildAcc/idx ca))
                     :idx (:wat::core::i64::+ (:wat::grep::ChildAcc/idx ca) 1)))
                 (:wat::grep::ChildAcc :acc (:wat::grep::empty-acc) :idx 0)
                 (:wat::core::ast->children forms))))
           ((:wat::core::ReadOutcome::Malformed __cause) (:wat::grep::empty-acc)))]
    (:wat::grep::Facts
      :nodes (:wat::grep::Acc/nodes acc)
      :named (:wat::grep::Acc/named acc)
      :spans (:wat::grep::Acc/spans acc))))

;; ── the ONE query — never written by a user; wat-grep owns exactly one query so the printer
;; is TOTAL, rendering exactly one type it fully knows. ───────────────────────────────────────

(:wat::rete::defquery :wat::grep::q-match
  :params []
  :when [(?fact <- :wat::grep::Match)])
