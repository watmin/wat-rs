;; rules-corpus 02 — GATES, UNLOCKS, AND "I DON'T KNOW" AS A QUERYABLE RESULT
;;
;; Corpus 01 proved the mechanism (a fact per node; position as a join). This one is about
;; the SHAPE OF THE REASONING, which is the actual subject.
;;
;; ─── WHAT THIS IS NOT ────────────────────────────────────────────────────────
;; It is NOT a transliteration of `fix.wat`'s if/else chain into rules. Those conditionals
;; are an artifact of walking a child vector left-to-right; they are not the knowledge.
;; Encoding them faithfully would produce a walk wearing rules.
;;
;; ─── THE SHAPE: A CHAIN OF GATES, EACH UNLOCKING THE NEXT ────────────────────
;; "What does `:wat::core::i64::to-string` become?" is not one decision. It is:
;;
;;   G1  what CONCEPT does this prefix name?        (`String` and `string` are one concept)
;;   G2  what STYLE is that concept written in?     (`/` slash, or `::` colons)
;;   G3  is the concept written CONSISTENTLY?       (two styles for one concept ⇒ blocked)
;;   G4  have we RULED where the concept relocates? (a decision, not derivable)
;;   G5  only now: emit the target
;;
;; A member cannot reach G5 without passing G3 and G4. Forward chaining IS the gating —
;; there is no control flow to get the ordering wrong, because a rule that lacks its input
;; fact simply does not fire.
;;
;; ─── AND THE PART THAT MATTERS MOST: TYPED UNKNOWNS ──────────────────────────
;; "I don't know" is NOT one bucket. Each gate that fails to open produces its OWN named
;; unknown, and the name IS the missing ruling:
;;
;;   Inconsistent [concept]   — one concept, two spellings; unify before targeting
;;   NoRuling     [concept]   — spelling is consistent, but nobody has said where it goes
;;
;; Querying those two is how the migration reports what it cannot do — instead of guessing,
;; and instead of dying on whichever file happens to contain the case first.
;;
;; ─── ★ WHAT THIS FILE FOUND — AN ENGINE DEFECT, NOW CLOSED ──────────────────
;; RESOLVED 2026-08-13 in ff581b6f. Every row below now reads its predicted value:
;;   Concept 5 · StyleSeen 4 · Settled 2 · TargetNS 1 · Target 2 · Inconsistent 1 · NoRuling 1
;; Kept in full because the diagnosis is the teaching, and because these rows are now the
;; standing regression test for the stratifier's positive-dependency rule.
;;
;; ─── THE DEFECT, AS FOUND (history) ─────────────────────────────────────────
;; Every gate below derives correctly: Concept 5, StyleSeen 4, Settled 2, Inconsistent 1,
;; NoRuling 1 — all exactly as predicted. `Target` reads 0 where it must read 2.
;;
;; The cause is NOT this design. `Settled` is derived THROUGH a negation (stratum 2), and a
;; post-negation fact does not join with a base fact in a downstream rule. Reproduced
;; minimally, both arms in one file:
;;   wat-scripts/scratch-pad/probe-strat2-derived-join-base.wat
;;   -> control (no negation) Out = 1 ; subject (negation) Out = 0, same join, same data.
;;
;; This BLOCKS the gate/unlock shape in general — "derive a gate by negation, then join the
;; gate against a ruling table" is precisely the pattern — so it is the thing to fix before
;; the migration rules are built out. The `Target` row is left RED on purpose: it is a live
;; acceptance test for that fix, not a bug in the reasoning.  [CLOSED — see above.]

;; ─── BASE FACTS — mechanical, asserted by a reader pass. NO reasoning here. ───
;; The prefix/base SPLIT is mechanical (cut at the last separator). Giving the rules
;; pre-split facts is deliberate: string surgery is not reasoning, and the fence would
;; refuse most of it anyway. Feed rules facts at the granularity of the DECISION.
(:wat::core::defrecord :m::Member
  [id     <- :wat::core::i64
   prefix <- :wat::core::String     ;; "String" | "string" | "i64" | "Vector" …
   base   <- :wat::core::String     ;; "concat" | "length" | "to-string" …
   style  <- :wat::core::String])   ;; "slash" (Prefix/base) | "colons" (prefix::base)

;; A RULING is data, not a branch. This is the decision table L6 argues for: nothing in the
;; spelling of `:wat::core::string::length` says it becomes `wat.string/length` while
;; `:wat::core::i64::to-string` becomes `wat.core.i64/to-string`. Somebody decided.
(:wat::core::defrecord :m::Ruling
  [concept <- :wat::core::String
   target  <- :wat::core::String])

;; ─── DERIVED: the gates ──────────────────────────────────────────────────────
(:wat::core::defrecord :m::Concept      [id <- :wat::core::i64  concept <- :wat::core::String  style <- :wat::core::String])
(:wat::core::defrecord :m::StyleSeen    [concept <- :wat::core::String  style <- :wat::core::String])
(:wat::core::defrecord :m::Inconsistent [concept <- :wat::core::String])
(:wat::core::defrecord :m::Settled      [concept <- :wat::core::String])
(:wat::core::defrecord :m::Target       [id <- :wat::core::i64  ns <- :wat::core::String  base <- :wat::core::String])
(:wat::core::defrecord :m::NoRuling     [concept <- :wat::core::String])

;; G1 — CONCEPT. `String` and `string` are the same concept; case is a spelling accident.
;; The RHS computes (rete ops only), so the normalisation is part of the derivation.
(:wat::rete::defrule :m::concept-of
  :when [(:m::Member (?id <- :id) (?p <- :prefix) (?s <- :style))]
  :then [(:m::Concept :id ?id
                      :concept (:wat::rete::string::to-lowercase ?p)
                      :style ?s)])

;; G2 — STYLE SEEN. Project each occurrence down to (concept, style). Two members of the
;; same concept in the same style collapse to one fact; two DIFFERENT styles do not.
(:wat::rete::defrule :m::style-seen
  :when [(:m::Concept (?c <- :concept) (?s <- :style))]
  :then [(:m::StyleSeen :concept ?c :style ?s)])

;; G3 — INCONSISTENT. One concept carrying two distinct styles. This is a SELF-JOIN on the
;; derived fact — the shape that finds a contradiction inside a set, which no left-to-right
;; walk can see because the two witnesses may be a thousand lines and two files apart.
(:wat::rete::defrule :m::inconsistent
  :when [(:m::StyleSeen (?c <- :concept) (?s1 <- :style))
         (:m::StyleSeen (?c <- :concept) (?s2 <- :style))
         (:wat::rete::where (:wat::rete::core::not (:wat::rete::string::= ?s1 ?s2)))]
  :then [(:m::Inconsistent :concept ?c)])

;; G3' — SETTLED. The negation is the gate: a concept is settled only if nothing derived it
;; inconsistent. Stratified negation means this fires only after G3 has run to fixpoint —
;; the ordering is a property of the rule set, not something a programmer sequenced.
(:wat::rete::defrule :m::settled
  :when [(:m::StyleSeen (?c <- :concept))
         (:wat::rete::not (:m::Inconsistent (?c <- :concept)))]
  :then [(:m::Settled :concept ?c)])

;; G4/G5 — TARGET. Requires BOTH unlocks: the concept is settled AND a ruling exists.
;; Neither is checked by an `if`; both are joins, so a missing one simply does not fire.
(:wat::rete::defrule :m::target
  :when [(:m::Concept (?id <- :id) (?c <- :concept))
         (:m::Settled (?c <- :concept))
         (:m::Ruling  (?c <- :concept) (?t <- :target))
         (:m::Member  (?id <- :id) (?b <- :base))]
  :then [(:m::Target :id ?id :ns ?t :base ?b)])

;; BISECT probe: same rule minus the Member re-join, to locate why `target` is dark.
(:wat::core::defrecord :m::TargetNS [concept <- :wat::core::String  ns <- :wat::core::String])
(:wat::rete::defrule :m::target-ns
  :when [(:m::Settled (?c <- :concept))
         (:m::Ruling  (?c <- :concept) (?t <- :target))]
  :then [(:m::TargetNS :concept ?c :ns ?t)])

;; ★ THE TYPED UNKNOWN — settled spelling, but no ruling. This is not an error path; it is a
;; RESULT. Querying it answers "what do we still not know how to translate, and why".
(:wat::rete::defrule :m::no-ruling
  :when [(:m::Settled (?c <- :concept))
         (:wat::rete::not (:m::Ruling (?c <- :concept)))]
  :then [(:m::NoRuling :concept ?c)])

(:wat::rete::defquery :m::q-Concept
  :params []
  :when [(?fact <- :m::Concept)])


(:wat::rete::defquery :m::q-StyleSeen
  :params []
  :when [(?fact <- :m::StyleSeen)])


(:wat::rete::defquery :m::q-Settled
  :params []
  :when [(?fact <- :m::Settled)])


(:wat::rete::defquery :m::q-TargetNS
  :params []
  :when [(?fact <- :m::TargetNS)])


(:wat::rete::defquery :m::q-Target
  :params []
  :when [(?fact <- :m::Target)])


(:wat::rete::defquery :m::q-Inconsistent
  :params []
  :when [(?fact <- :m::Inconsistent)])


(:wat::rete::defquery :m::q-NoRuling
  :params []
  :when [(?fact <- :m::NoRuling)])


;; ─── the seed — REAL spellings, measured from wat/ this session ───────────────
;;   String/  2 members  ·  string::  19 members   <- ONE concept, TWO styles  => Inconsistent
;;   i64::   13 members                            <- consistent; ruled        => Target
;;   Vector/  4 members                            <- consistent; NOT ruled    => NoRuling
(:wat::core::defn :m::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert-all s
    (:wat::core::PersistentVector
      (:m::Member :id 1 :prefix "String" :base "concat"    :style "slash")
      (:m::Member :id 2 :prefix "string" :base "length"    :style "colons")
      (:m::Member :id 3 :prefix "i64"    :base "to-string" :style "colons")
      (:m::Member :id 4 :prefix "i64"    :base "+"         :style "colons")
      (:m::Member :id 5 :prefix "Vector" :base "get"       :style "slash"))))

;; The rulings we HAVE made. `string` is deliberately absent-by-blocking and `vector`
;; deliberately absent-by-omission — two different unknowns, and the point is that the
;; engine distinguishes them.
(:wat::core::defn :m::seed-rulings [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert-all s
    (:wat::core::PersistentVector
      (:m::Ruling :concept "i64" :target "wat.core.i64"))))

(:wat::core::defn :m::show [label <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat label (:wat::core::str n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::core::PersistentVector
             (:m::concept-of) (:m::style-seen) (:m::inconsistent)
             (:m::settled) (:m::target) (:m::target-ns) (:m::no-ruling))
     fired (:wat::rete::fire-rules
             (:m::seed-rulings (:m::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:m::q-Concept) (:m::q-StyleSeen) (:m::q-Settled) (:m::q-TargetNS) (:m::q-Target) (:m::q-Inconsistent) (:m::q-NoRuling))))))]
    (:wat::core::do
      ;; non-vacuity: 0 concepts ⇒ the seed never landed and every row below is meaningless
      (:m::show "Concept      (want 5; 0 => seed dead, all below vacuous): "
        (:wat::core::length (:wat::rete::query fired (:m::q-Concept))))
      (:m::show "StyleSeen    (want 4 = string/slash,string/colons,i64,vector): "
        (:wat::core::length (:wat::rete::query fired (:m::q-StyleSeen))))
      (:m::show "Settled      (want 2 = i64, vector):                      "
        (:wat::core::length (:wat::rete::query fired (:m::q-Settled))))
      ;; the two answers
      (:m::show "TargetNS     (bisect, want 1; GREEN since #94):              "
        (:wat::core::length (:wat::rete::query fired (:m::q-TargetNS))))
      (:m::show "Target       (want 2; GREEN since ff581b6f closed #94):      "
        (:wat::core::length (:wat::rete::query fired (:m::q-Target))))
      ;; ★ the two DISTINCT unknowns — this is the deliverable
      (:m::show "Inconsistent (want 1 = 'string': String/ vs string::):    "
        (:wat::core::length (:wat::rete::query fired (:m::q-Inconsistent))))
      (:m::show "NoRuling     (want 1 = 'vector': settled, undecided):     "
        (:wat::core::length (:wat::rete::query fired (:m::q-NoRuling)))))))
