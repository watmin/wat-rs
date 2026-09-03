;; D7 drive — one `aid` receives BOTH writer 1 (push, fire/delta.rs:100) and
;; writer 2 (replace, fire/pass/alpha.rs:130) in one seed pass. TRIGGER FOUND,
;; 2026-09-02. This is a LIVE correctness defect, not a latent shape.
;;
;; THE MECHANISM. `pack_i64_row` (session.rs:309) tests RUNTIME values — every
;; field must be `Value::i64`. The key in `leaf_aids` / `class_ids` is the fact's
;; ERASED class FQDN. A PARAMETRIC record
;;   (defrecord :Box :- [T] [k <- i64  v <- :T])
;; therefore gives ONE class whose instances differ in packability: `Box[i64]`
;; packs and joins the batch, `Box[String]` does not and falls to
;; `alpha_activate_fact` (writer 1). Both reach the SAME `aid`, because
;; `build_alpha_index` files an alpha under one type head and `candidates_into`
;; keys on that same erased class. Writer 2's `wm.alpha.insert(aid, els)`
;; replaces the whole `Arc<Vec<Element>>` and DISCARDS what writer 1 pushed.
;;
;; A PersistentVector's element type is invariant and inferred from its first
;; element, so the two instantiations must each be upcast to `Record` first —
;; that is all `:d7::as-record` does. Struct-nature facts cannot be smuggled in
;; the same way: `:d7s::S` is refused at the `Record` wall (checked).
;;
;; OBSERVED: native=2 oracle=3. The `Box[String]` fact's Hit is silently lost;
;; `fire-rules$oracle` derives all three. Ordering does not matter (the batch
;; runs after the fact loop): [i64,String] -> 1, [String,i64] -> 1,
;; [i64,String,String,i64] -> 2 of 4.
;;
;; ⛔ THE ARMED DIFFERENTIAL IS BLIND TO THIS. `record_seed_leaf_vs_alpha`
;; (delta.rs:118-170) builds `predicted` by SKIPPING any fact whose
;; `i64_by_fact[i]` is `None` — the very filter that decides batch membership —
;; so it re-derives writer 2's own output. Measured on this program with
;; `with_leaf_occ_diff` armed: predicted=2 actual=2 extra=[] missing=[].
;; `extra` cannot be non-empty for this mechanism.
;;
;; See also `d7-pack-width-controls.wat` for the angles that do NOT collide.

(:wat::core::defrecord :d7::Box :- [T] [k <- :wat::core::i64  v <- :T])
(:wat::core::defrecord :d7::Hit [k <- :wat::core::i64])

(:wat::rete::defrule :d7::r
  :when  [(:d7::Box (?k <- :k) (?v <- :v))]
  :then  [(:d7::Hit ?k)])

(:wat::rete::defquery :d7::q :params [] :when [(?fact <- :d7::Hit)])

(:wat::core::defn :d7::hits [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::Vector/length
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:d7::Hit/k (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "?fact")))
        (:wat::rete::query s (:d7::q))))))


;; The fact bag, typed `(PersistentVector :- [Record])`. A PersistentVector's
;; element type is INVARIANT and inferred from its first element, so the two Box
;; INSTANTIATIONS (`Box[i64]`, `Box[String]`) must each be UPCAST to `Record`
;; first — that upcast is the only thing `:d7::as-record` does.
(:wat::core::defn :d7::as-record [r <- :wat::core::Record] -> :wat::core::Record r)

(:wat::core::defn :d7::facts [] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::PersistentVector
    (:d7::as-record (:d7::Box :k 0 :v 100))
    (:d7::as-record (:d7::Box :k 1 :v "not-an-i64"))
    (:d7::as-record (:d7::Box :k 2 :v 200))))

(:wat::core::defn :d7::run [] -> :wat::core::String
  (:wat::core::let
    [session (:wat::core::match (:wat::rete::compile-all
               (:wat::core::PersistentVector (:d7::r))
               (:wat::core::PersistentVector (:d7::q)))
               ((:wat::rete::CompileOutcome::Compiled __s) __s)
               ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
                 (:wat::kernel::assertion-failed! "compile" :wat::core::None :wat::core::None)))
     staged (:wat::core::match (:wat::rete::insert-all session (:d7::facts))
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
                 (:wat::kernel::assertion-failed! "fire ceiling" :wat::core::None :wat::core::None))
               ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
                 (:wat::kernel::assertion-failed! "fire cap" :wat::core::None :wat::core::None)))]
    (:wat::core::String/concat
      (:wat::core::String/concat "native=" (:wat::core::i64::to-string (:d7::hits native)))
      (:wat::core::String/concat " oracle=" (:wat::core::i64::to-string (:d7::hits oracle))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:d7::run)))
