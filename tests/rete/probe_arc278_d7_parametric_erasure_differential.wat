;; D7 — NATIVE vs ORACLE over a TYPE-ERASURE SEAM in the seed pass.
;;
;; THE PROPERTY THIS GATES — not the fixture, the property:
;;
;;   > A single runtime CLASS whose instances DIFFER in packability
;;   > (`pack_i64_row`, session.rs) must derive exactly what the oracle derives.
;;
;; `pack_i64_row` tests RUNTIME values; `build_alpha_index` files every alpha
;; node under ONE erased `pat.type_head`. So any construct that lets one class
;; hold both an all-i64 instance and a non-all-i64 one puts BOTH seed writers —
;; `alpha_activate_fact`'s push and the occupancy batch's whole-entry replace —
;; onto the SAME `aid`. Before class-uniform batching the replace discarded the
;; push and `d_alpha` was left indexing DIFFERENT elements: measured
;; `native=2 oracle=3` on 2026-09-02, a derived fact lost with no diagnostic.
;;
;; PARAMETRIC RECORDS ARE THE CONSTRUCTOR. `(:d7g::Box :- [T] [k <- i64 v <- :T])`
;; erases `T`, so `Box[i64]` and `Box[String]` are ONE class `d7g::Box`. There is
;; no non-generic route today (no `Any` supertype that `i64` and `String` share),
;; which is why the seam went unnoticed — and exactly why the gate must drive the
;; generic, not a hand-picked pair. Two distinct erasures are driven below
;; (a String filler and a RECORD filler) plus both interleavings and a 4-fact
;; alternation, because the batch runs AFTER the fact loop and order must not
;; matter.
;;
;; ⛔ NOT A COUNT — THE KEY SET. Each entry returns the SORTED derived keys. A
;; count alone cannot see the aliasing half of the defect: `d_alpha[aid]` kept
;; the pushed slot indices, which after the replace named a different element,
;; so a wrong-but-same-size answer is exactly what this mechanism produces.
;;
;; ⛔ NOT `leaf_occ`. That differential builds `predicted` from the same
;; packability predicate that decides batch membership, so it compared writer 2
;; to writer 2 and read `extra=[] missing=[]` while the fact was dropping.

(:wat::core::defrecord :d7g::Box :- [T] [k <- :wat::core::i64  v <- :T])
;; A RECORD-valued filler: a second, independent way for one `Box` instance to
;; fail `pack_i64_row` (Aggregate, not `Value::i64`) — so the gate is not pinned
;; to `String`.
(:wat::core::defrecord :d7g::Tag [n <- :wat::core::i64])
;; A NON-parametric, uniformly-packable class living in the SAME session. It must
;; keep the occupancy batch while its neighbour loses it: a cure that narrowed
;; batching to nothing would satisfy every equality assertion above and silently
;; delete the fast path.
(:wat::core::defrecord :d7g::Plain [k <- :wat::core::i64])

(:wat::core::defrecord :d7g::Hit      [k <- :wat::core::i64])
(:wat::core::defrecord :d7g::PlainHit [k <- :wat::core::i64])
(:wat::core::defrecord :d7g::Pair     [k <- :wat::core::i64])

(:wat::rete::defrule :d7g::r-box
  :when  [(:d7g::Box (?k <- :k) (?v <- :v))]
  :then  [(:d7g::Hit ?k)])

(:wat::rete::defrule :d7g::r-plain
  :when  [(:d7g::Plain (?k <- :k))]
  :then  [(:d7g::PlainHit ?k)])

;; The JOIN arm. `Box`'s alpha delta is consumed here as SLOT INDICES into
;; `wm.alpha[aid]` — the consumer that turns the aliasing into a wrong binding
;; rather than merely a missing one. It also gives `d7g::Box` a SECOND leaf aid,
;; so the class-uniform decision is exercised across more than one node.
(:wat::rete::defrule :d7g::r-pair
  :when  [(:d7g::Box (?k <- :k) (?v <- :v))
          (:d7g::Plain (?k <- :k))]
  :then  [(:d7g::Pair ?k)])

(:wat::rete::defquery :d7g::q-hit   :params [] :when [(?fact <- :d7g::Hit)])
(:wat::rete::defquery :d7g::q-plain :params [] :when [(?fact <- :d7g::PlainHit)])
(:wat::rete::defquery :d7g::q-pair  :params [] :when [(?fact <- :d7g::Pair)])

;; ── reporting ────────────────────────────────────────────────────────────────

(:wat::core::defn :d7g::render
  [ks <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String
                     n   <- :wat::core::i64] -> :wat::core::String
      (:wat::core::String/concat
        acc
        (:wat::core::String/concat (:wat::core::i64::to-string n) ",")))
    ""
    (:wat::core::sort ks)))

(:wat::core::defn :d7g::hit-keys [s <- :wat::rete::Session] -> :wat::core::String
  (:d7g::render
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:d7g::Hit/k (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "?fact")))
        (:wat::rete::query s (:d7g::q-hit))))))

(:wat::core::defn :d7g::plain-keys [s <- :wat::rete::Session] -> :wat::core::String
  (:d7g::render
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:d7g::PlainHit/k (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "?fact")))
        (:wat::rete::query s (:d7g::q-plain))))))

(:wat::core::defn :d7g::pair-keys [s <- :wat::rete::Session] -> :wat::core::String
  (:d7g::render
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:d7g::Pair/k (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "?fact")))
        (:wat::rete::query s (:d7g::q-pair))))))

;; ── the driver ───────────────────────────────────────────────────────────────
;;
;; One shape for every workload: stage the SAME session once, fire it through
;; `fire-rules` and through `fire-rules$oracle`, and report all three derived key
;; sets from each. Both engines read the identical staged session, so a
;; difference is the engine and nothing else.

(:wat::core::defn :d7g::as-record [r <- :wat::core::Record] -> :wat::core::Record r)

(:wat::core::defn :d7g::report
  [facts <- (:wat::core::PersistentVector :- [:wat::core::Record])] -> :wat::core::String
  (:wat::core::let
    [session (:wat::core::match (:wat::rete::compile-all
               (:wat::core::PersistentVector (:d7g::r-box) (:d7g::r-plain) (:d7g::r-pair))
               (:wat::core::PersistentVector (:d7g::q-hit) (:d7g::q-plain) (:d7g::q-pair)))
               ((:wat::rete::CompileOutcome::Compiled __s) __s)
               ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
                 (:wat::kernel::assertion-failed! "compile" :wat::core::None :wat::core::None)))
     staged (:wat::core::match (:wat::rete::insert-all session facts)
               ((:wat::rete::InsertOutcome::Inserted __s) __s)
               ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c)
                 (:wat::kernel::assertion-failed! "insert" :wat::core::None :wat::core::None)))
     native (:wat::core::match (:wat::rete::fire-rules staged)
               ((:wat::rete::FireOutcome::Fired __f) __f)
               ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
                 (:wat::kernel::assertion-failed! "fire ceiling" :wat::core::None :wat::core::None))
               ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
                 (:wat::kernel::assertion-failed! "fire cap" :wat::core::None :wat::core::None)))
     oracle (:wat::core::match (:wat::rete::fire-rules$oracle staged)
               ((:wat::rete::FireOutcome::Fired __f) __f)
               ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r)
                 (:wat::kernel::assertion-failed! "oracle ceiling" :wat::core::None :wat::core::None))
               ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
                 (:wat::kernel::assertion-failed! "oracle cap" :wat::core::None :wat::core::None)))]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat
          (:wat::core::String/concat "hitN=" (:d7g::hit-keys native))
          (:wat::core::String/concat " hitO=" (:d7g::hit-keys oracle)))
        (:wat::core::String/concat
          (:wat::core::String/concat " plainN=" (:d7g::plain-keys native))
          (:wat::core::String/concat " plainO=" (:d7g::plain-keys oracle))))
      (:wat::core::String/concat
        (:wat::core::String/concat " pairN=" (:d7g::pair-keys native))
        (:wat::core::String/concat " pairO=" (:d7g::pair-keys oracle))))))

;; A `PersistentVector`'s element type is INVARIANT and inferred from its first
;; element, so each `Box` INSTANTIATION must be upcast to `Record` before the two
;; can share one bag — that upcast is all `:d7g::as-record` does.

;; ── the workloads ────────────────────────────────────────────────────────────

;; 1 — the D7 shape: a packable instance FIRST, then the erased one.
(:wat::core::defn :user::mixed-i64-first [] -> :wat::core::String
  (:d7g::report
    (:wat::core::PersistentVector
      (:d7g::as-record (:d7g::Box :k 0 :v 100))
      (:d7g::as-record (:d7g::Box :k 1 :v "not-an-i64"))
      (:d7g::as-record (:d7g::Box :k 2 :v 200)))))

;; 2 — the same class, the erased instance FIRST. The batch runs after the fact
;; loop, so order must not change the answer; it did not change the DEFECT
;; either, and a cure that only handled one order would pass 1 and fail here.
(:wat::core::defn :user::mixed-erased-first [] -> :wat::core::String
  (:d7g::report
    (:wat::core::PersistentVector
      (:d7g::as-record (:d7g::Box :k 0 :v "not-an-i64"))
      (:d7g::as-record (:d7g::Box :k 1 :v 100))
      (:d7g::as-record (:d7g::Box :k 2 :v 200)))))

;; 3 — alternating, four facts: two packable, two erased.
(:wat::core::defn :user::mixed-alternating [] -> :wat::core::String
  (:d7g::report
    (:wat::core::PersistentVector
      (:d7g::as-record (:d7g::Box :k 0 :v 100))
      (:d7g::as-record (:d7g::Box :k 1 :v "a"))
      (:d7g::as-record (:d7g::Box :k 2 :v 200))
      (:d7g::as-record (:d7g::Box :k 3 :v "b")))))

;; 4 — A DIFFERENT ERASURE. The unpackable filler is a RECORD, not a String, so
;; the gate is about "one class, mixed packability" and not about one type pair.
(:wat::core::defn :user::mixed-record-filler [] -> :wat::core::String
  (:d7g::report
    (:wat::core::PersistentVector
      (:d7g::as-record (:d7g::Box :k 0 :v 100))
      (:d7g::as-record (:d7g::Box :k 1 :v (:d7g::Tag :n 7)))
      (:d7g::as-record (:d7g::Box :k 2 :v 200)))))

;; 5 — CONTROL, uniformly PACKABLE: every `Box` holds an i64, so the class keeps
;; the occupancy batch. This is the arm a batching-narrowing cure must not break.
(:wat::core::defn :user::uniform-packable [] -> :wat::core::String
  (:d7g::report
    (:wat::core::PersistentVector
      (:d7g::as-record (:d7g::Box :k 0 :v 100))
      (:d7g::as-record (:d7g::Box :k 1 :v 150))
      (:d7g::as-record (:d7g::Box :k 2 :v 200)))))

;; 6 — CONTROL, uniformly UNPACKABLE: no `Box` packs, so the class was never in
;; the batch and only writer 1 ever ran. This arm was already correct before the
;; cure; it is here so a regression on the all-activate path names itself.
(:wat::core::defn :user::uniform-unpackable [] -> :wat::core::String
  (:d7g::report
    (:wat::core::PersistentVector
      (:d7g::as-record (:d7g::Box :k 0 :v "a"))
      (:d7g::as-record (:d7g::Box :k 1 :v "b"))
      (:d7g::as-record (:d7g::Box :k 2 :v "c")))))

;; 7 — ★ THE MIXED CLASS BESIDE A UNIFORM ONE, in one session. `d7g::Box` loses
;; the batch; `d7g::Plain` must keep it, and the join must pair every `Box` with
;; its `Plain` — including the erased `Box`, whose element is the one the replace
;; used to discard.
(:wat::core::defn :user::mixed-beside-uniform [] -> :wat::core::String
  (:d7g::report
    (:wat::core::PersistentVector
      (:d7g::as-record (:d7g::Box :k 0 :v 100))
      (:d7g::as-record (:d7g::Box :k 1 :v "not-an-i64"))
      (:d7g::as-record (:d7g::Box :k 2 :v 200))
      (:d7g::as-record (:d7g::Plain :k 0))
      (:d7g::as-record (:d7g::Plain :k 1))
      (:d7g::as-record (:d7g::Plain :k 2)))))
