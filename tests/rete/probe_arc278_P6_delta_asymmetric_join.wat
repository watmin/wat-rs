;; Fixture BESIDE probe_arc278_P6_delta_asymmetric_join.rs — the record types for all three
;; asymmetric-arrival join scenarios (chain / triple-cascade / xyz). The RULES are constructed at
;; runtime in the .rs (parameterized by N inputs), so only the record types live here.
;;
;; NO :user::main — a world-under-test needs none (freeze does not require it; cf. startup_bare).

;; chain: R1 A→B, R2 B⋈A→C  (right-before-left arrival)
(:wat::core::defrecord :chain::A [k <- :wat::core::i64])
(:wat::core::defrecord :chain::B [k <- :wat::core::i64])
(:wat::core::defrecord :chain::C [k <- :wat::core::i64])

;; triple cascade: R1 A→B, R2 B⋈A→C (derived⋈input), R3 C⋈B→D (derived⋈derived)
(:wat::core::defrecord :tri::A [k <- :wat::core::i64])
(:wat::core::defrecord :tri::B [k <- :wat::core::i64])
(:wat::core::defrecord :tri::C [k <- :wat::core::i64])
(:wat::core::defrecord :tri::D [k <- :wat::core::i64])

;; xyz: R1 X⋈Y→Z  (left-before-right arrival)
(:wat::core::defrecord :xyz::X [k <- :wat::core::i64])
(:wat::core::defrecord :xyz::Y [k <- :wat::core::i64])
(:wat::core::defrecord :xyz::Z [k <- :wat::core::i64])
