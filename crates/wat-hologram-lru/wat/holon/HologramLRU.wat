;; :wat::holon::HologramLRU — bounded coordinate-cell store with
;; cosine readout. Composes :wat::holon::Hologram (substrate) and
;; :wat::lru::LocalCache (wat-lru). When the LRU evicts a key, the
;; matching Hologram cell entry is dropped.
;;
;; Arc 074 slice 2. The bounded sibling of slice 1's :wat::holon::
;; Hologram. Lab umbrella 059 slice 1's L1/L2 cache lands on this.
;;
;; Surface mirrors Hologram's where it overlaps; differences:
;;
;;   Hologram::new       (d :i64) -> Hologram
;;   HologramLRU/make    (d :i64) (cap :i64) -> HologramLRU
;;
;;   Hologram/put        store pos key val -> ()
;;   HologramLRU/put     store pos key val -> ()       ;; ALSO updates LRU + drops evicted
;;
;;   Hologram/get        store pos probe filter -> Option<HolonAST>
;;   HologramLRU/get     store pos probe filter -> Option<HolonAST>  ;; ALSO bumps LRU on hit
;;
;;   {Hologram,HologramLRU}/coincident-get / present-get / len / dim — same shape

(:wat::core::struct :wat::holon::HologramLRU
  (hologram :wat::holon::Hologram)
  (lru :wat::lru::LocalCache<wat::holon::HolonAST,i64>))

;; ─── Construction ────────────────────────────────────────────────
;;
;; Takes the encoding `d` (sets up the inner Hologram's sqrt(d) cells)
;; and a `cap` (the LRU's global capacity bound — when exceeded, the
;; least-recently-used entry is evicted). The user picks `cap` based
;; on memory budget; a reasonable starting point is 100 × sqrt(d) for
;; ~100 entries per cell on average.
(:wat::core::define
  (:wat::holon::HologramLRU/make
    (d :i64)
    (cap :i64)
    -> :wat::holon::HologramLRU)
  (:wat::holon::HologramLRU/new
    (:wat::holon::Hologram/new d)
    (:wat::lru::LocalCache::new cap)))

;; ─── put — insert + LRU bookkeeping ──────────────────────────────
;;
;; 1. Compute cell idx for pos.
;; 2. Insert (key, val) into the inner Hologram cell.
;; 3. Push key→idx onto the LRU.
;; 4. If the LRU evicted an entry, drop it from the Hologram cell.
;;
;; Step 3's LocalCache::put returns Option<(K, V)> after the wat-lru
;; eviction-aware change — we use that to clean up step 4.
(:wat::core::define
  (:wat::holon::HologramLRU/put
    (store :wat::holon::HologramLRU)
    (pos :f64)
    (key :wat::holon::HolonAST)
    (val :wat::holon::HolonAST)
    -> :())
  (:wat::core::let*
    (((h :wat::holon::Hologram) (:wat::holon::HologramLRU/hologram store))
     ((lru :wat::lru::LocalCache<wat::holon::HolonAST,i64>)
      (:wat::holon::HologramLRU/lru store))
     ((idx :i64) (:wat::holon::Hologram/pos-to-idx h pos))
     ((_ :()) (:wat::holon::Hologram/put h pos key val))
     ((evicted :Option<(wat::holon::HolonAST,i64)>)
      (:wat::lru::LocalCache::put lru key idx)))
    (:wat::core::match evicted -> :()
      ((Some pair)
        (:wat::core::let*
          (((evicted-key :wat::holon::HolonAST) (:wat::core::first pair))
           ((evicted-idx :i64) (:wat::core::second pair))
           ((_ :Option<wat::holon::HolonAST>)
            (:wat::holon::Hologram/remove-at-index h evicted-idx evicted-key)))
          ()))
      (:None ()))))

;; ─── get — find-best + filter + LRU bump on hit ──────────────────
;;
;; Same cosine readout as Hologram/get, with the addition that on a
;; passing-filter hit, the matched key is touched in the LRU (moving
;; it to MRU). Cold entries fade; hot entries stay.
(:wat::core::define
  (:wat::holon::HologramLRU/get
    (store :wat::holon::HologramLRU)
    (pos :f64)
    (probe :wat::holon::HolonAST)
    (filter :fn(f64)->bool)
    -> :Option<wat::holon::HolonAST>)
  (:wat::core::let*
    (((h :wat::holon::Hologram) (:wat::holon::HologramLRU/hologram store))
     ((lru :wat::lru::LocalCache<wat::holon::HolonAST,i64>)
      (:wat::holon::HologramLRU/lru store)))
    (:wat::core::match
      (:wat::holon::Hologram/find-best h pos probe)
      -> :Option<wat::holon::HolonAST>
      ((Some triple)
        (:wat::core::let*
          (((matched-key :wat::holon::HolonAST) (:wat::core::first triple))
           ((val :wat::holon::HolonAST) (:wat::core::second triple))
           ((cos :f64) (:wat::core::third triple)))
          (:wat::core::if (filter cos) -> :Option<wat::holon::HolonAST>
            ;; Hit: bump the matched key to MRU (LocalCache::put
            ;; updates LRU order on existing keys), then return Some.
            (:wat::core::let*
              (((idx :i64) (:wat::holon::Hologram/pos-to-idx h pos))
               ((_ :Option<(wat::holon::HolonAST,i64)>)
                (:wat::lru::LocalCache::put lru matched-key idx)))
              (Some val))
            ;; Miss: filter rejected; don't bump.
            :None)))
      (:None :None))))

;; ─── coincident-get — strict variant (mirrors Hologram's) ────────
(:wat::core::define
  (:wat::holon::HologramLRU/coincident-get
    (store :wat::holon::HologramLRU)
    (pos :f64)
    (probe :wat::holon::HolonAST)
    -> :Option<wat::holon::HolonAST>)
  (:wat::holon::HologramLRU/get store pos probe
    (:wat::holon::filter-coincident
      (:wat::holon::Hologram/dim
        (:wat::holon::HologramLRU/hologram store)))))

;; ─── present-get — looser variant ────────────────────────────────
(:wat::core::define
  (:wat::holon::HologramLRU/present-get
    (store :wat::holon::HologramLRU)
    (pos :f64)
    (probe :wat::holon::HolonAST)
    -> :Option<wat::holon::HolonAST>)
  (:wat::holon::HologramLRU/get store pos probe
    (:wat::holon::filter-present
      (:wat::holon::Hologram/dim
        (:wat::holon::HologramLRU/hologram store)))))

;; ─── len — total entries across all cells ────────────────────────
(:wat::core::define
  (:wat::holon::HologramLRU/len
    (store :wat::holon::HologramLRU)
    -> :i64)
  (:wat::holon::Hologram/len
    (:wat::holon::HologramLRU/hologram store)))

;; ─── dim — encoding dimension this store was built against ───────
(:wat::core::define
  (:wat::holon::HologramLRU/dim
    (store :wat::holon::HologramLRU)
    -> :i64)
  (:wat::holon::Hologram/dim
    (:wat::holon::HologramLRU/hologram store)))
