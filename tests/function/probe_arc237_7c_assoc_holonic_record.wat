;; tests/function/probe_arc237_7c_assoc_holonic_record.wat
;; Arc 237 Stone 237.7c — assoc on a holonic defrecord.
;; Loaded via startup_from_file by the #[ignore]'d sibling probe.
;; Currently RED (alias is HashMap-only); un-ignored + GREEN after Stone 237.7c ships.

(:wat::holon::defrecord :my::HolonicVoltage [value <- :wat::core::i64])
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:my::HolonicVoltage/value
    (:wat::core::assoc (:my::HolonicVoltage :value 10) :value 42)))
