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
  (:wat::vec::length
    (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:d7::Hit/k (:wat::core::Option/expect (:wat::map::get p "?fact") "?fact")))
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
               [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
               [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f}
                 (:wat::kernel::assertion-failed! :message "compile")])
     staged (:wat::core::match (:wat::rete::insert-all session (:d7::facts))
               [:wat::rete::InsertOutcome.Inserted {:session __s} __s]
               [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c}
                 (:wat::kernel::assertion-failed! :message "insert")])
     native (:wat::core::match (:wat::rete::fire-rules staged)
               [:wat::rete::FireOutcome.Fired {:value __f} __f]
               [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r}
                 (:wat::kernel::assertion-failed! :message "fire ceiling")]
               [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
                 (:wat::kernel::assertion-failed! :message "fire cap")])
     oracle (:wat::core::match (:wat::rete::fire-rules$oracle staged)
               [:wat::rete::FireOutcome.Fired {:value __f} __f]
               [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r}
                 (:wat::kernel::assertion-failed! :message "fire ceiling")]
               [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
                 (:wat::kernel::assertion-failed! :message "fire cap")])]
    (:wat::string::concat
      (:wat::string::concat "native=" (:wat::i64::to-string (:d7::hits native)))
      (:wat::string::concat " oracle=" (:wat::i64::to-string (:d7::hits oracle))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:d7::run)))
