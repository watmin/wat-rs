# EXPECTATIONS — STONE D

| # | what | check | expected |
|---|---|---|---|
| 1 | coverage unchanged | union of shard slices | every one of the ~770 files checked exactly once |
| 2 | names preserved | `grep` the 17 citing files | all still true, zero edits needed |
| 3 | invariant holds | one shard isolated × 4.4 | < 69% of its kill |
| 4 | a broken file reds ONE shard | mutation 1 | one shard FAIL, naming the file |
| 5 | vacuity guarded | mutation 2 | RED |
| 6 | all three profiles moved | read `.config/nextest.toml` | default / ci / slow consistent |
| 7 | the floor | `.floor/latest/clean.log` Summary | **0 failed** |
| 8 | ⭐ wall-clock | floor before vs after | **before ~1026 s**; after reported, not predicted |
| 9 | clippy | `cargo clippy --release --all-targets` | clean |

Runtime: the edit is ~30 min; two floors dominate. ⚠ Row 8's "before" is one run on a box that
had other agents on it earlier the same day — hand the after-number down with its contamination.
