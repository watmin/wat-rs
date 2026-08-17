;; Fixture BESIDE probe_arc278_P6_delta_asymmetric_join.rs — the record types for all three
;; asymmetric-arrival join scenarios (chain / triple-cascade / xyz). The RULES are constructed at
;; runtime in the .rs (parameterized by N inputs), so only the record types live here.
;;
;; NO :user::main — a world-under-test needs none (freeze does not require it; cf. startup_bare).

;; chain: R1 A→B, R2 B⋈A→C  (right-before-left arrival)
(:wat::core::defrecord :chain::A [k <- :wat::core::i64])
(:wat::core::defrecord :chain::B [k <- :wat::core::i64])
(:wat::core::defrecord :chain::C [k <- :wat::core::i64])
(:wat::rete::defquery :chain::q-A :params [] :when [(?fact <- :chain::A)])
(:wat::rete::defquery :chain::q-B :params [] :when [(?fact <- :chain::B)])
(:wat::rete::defquery :chain::q-C :params [] :when [(?fact <- :chain::C)])

;; triple cascade: R1 A→B, R2 B⋈A→C (derived⋈input), R3 C⋈B→D (derived⋈derived)
(:wat::core::defrecord :tri::A [k <- :wat::core::i64])
(:wat::core::defrecord :tri::B [k <- :wat::core::i64])
(:wat::core::defrecord :tri::C [k <- :wat::core::i64])
(:wat::core::defrecord :tri::D [k <- :wat::core::i64])
(:wat::rete::defquery :tri::q-A :params [] :when [(?fact <- :tri::A)])
(:wat::rete::defquery :tri::q-B :params [] :when [(?fact <- :tri::B)])
(:wat::rete::defquery :tri::q-C :params [] :when [(?fact <- :tri::C)])
(:wat::rete::defquery :tri::q-D :params [] :when [(?fact <- :tri::D)])

;; xyz: R1 X⋈Y→Z  (left-before-right arrival)
(:wat::core::defrecord :xyz::X [k <- :wat::core::i64])
(:wat::core::defrecord :xyz::Y [k <- :wat::core::i64])
(:wat::core::defrecord :xyz::Z [k <- :wat::core::i64])
(:wat::rete::defquery :xyz::q-X :params [] :when [(?fact <- :xyz::X)])
(:wat::rete::defquery :xyz::q-Y :params [] :when [(?fact <- :xyz::Y)])
(:wat::rete::defquery :xyz::q-Z :params [] :when [(?fact <- :xyz::Z)])
