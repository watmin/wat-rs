# BRIEF — A1: the left-index latch

Cure the defect **and** land its gate, in one strike. **The floor must be GREEN when you are done.**

## Read in order

1. **`DESIGN.md` beside this file** — it pins the contract decision (structural, not a `first_keying`
   patch) and records the false citation two wards made. Read it first.
2. **`src/rete/kernel/session.rs:224-262`** — `JoinRightIndex` + `RightIndexWriter`. **This is the
   shape to copy.** Private `buckets`/`indexed_n`, one door (`push`), no accessor handing out `&mut`.
   Its header states the rule the left side needs.
3. **`src/rete/kernel/session.rs:208`** — `JoinLeftIndex`, today a bare
   `type = HashMap<i64, JoinKeyMap<Token>>`. This is what becomes a real type.
4. **`src/rete/kernel/fire/pass/hash_join.rs:118-124`** (the latch), **`:155-162`** (the catch-up
   gate), **`:265-280`** (the ONLY bulk builder), **`:429`** (the silent `None`).
5. **`src/rete/kernel/fire/mod.rs:782-792`** — `FilterJoinIdx`, two fields, no `left_idx`; and
   **`:802`** where it writes the memo. Note its doc describes D2 — the defect it was built to cure.
6. **`src/rete/kernel/fire/pass/mod.rs:107`** and **`filter_after_join.rs:75`** — the second writer
   and its only caller.
7. **`../vigilia-2026-09-05/probes/probe_vig_left_idx_latch.{rs,wat}`** — the probe. It reproduces at
   HEAD `8bca0f7fe`, verified by the orchestrator: `native=[OutW=1,…] oracle=[OutW=2,…]`.

## Implementation sketch

```rust
// session.rs — the left side gets the shape the right side already has
pub(crate) struct JoinLeftIndex {
    buckets: HashMap<i64, JoinKeyMap<Token>>,   // PRIVATE
    keys: HashMap<i64, Vec<Value>>,             // the memo, PRIVATE, same owner
}
impl JoinLeftIndex {
    /// The ONE door. Sets the key list and indexes the left side in a single act,
    /// so a writer cannot do one without the other.
    pub(crate) fn key_and_index(&mut self, join_id: i64, keys: …, toks: &[Token], …) { … }
    pub(crate) fn is_keyed(&self, join_id: i64) -> bool { … }   // replaces `first_keying`
    pub(crate) fn get(&self, join_id: i64) -> Option<&JoinKeyMap<Token>> { … }
}
```

Then `FilterJoinIdx` carries it, and `left_activate_join`'s path cannot key a join while leaving its
left index short.

## The gate

Land the probe as `tests/rete/probe_arc278_left_idx_latch.{rs,wat}` (adjacent-fixture convention).
**Drop `report_the_six_numbers`** — it is a deliberate `panic!` and would be a permanent floor RED.
Keep `the_control_reaches_a_second_round` (non-vacuity: it proves the fixture reaches round 2 with a
non-empty Δright against a non-empty `old_left`) and `native_agrees_with_the_oracle_on_the_guarded_chain`.

## Blast radius

`src/rete/kernel/` + one test pair. No change to the wat corpus. No codemod.

## STOP triggers

1. **If curing the latch reddens ANY existing rete test, STOP and report** with the verbatim failure
   and the exact arm. `sequi` L2-a says the conflation is load-bearing as D2's guard — a red here is
   the predicted shape of reopening D2, and it is a finding, not a nuisance.
2. **If the structural cure cannot be written without touching the wat corpus or `export.rs`'s wire
   format, STOP.** That is a bigger decision than this strike.
3. **If you find yourself patching `first_keying` in place, STOP** — see the DESIGN's rejected option.
4. **On any RED elsewhere: DO NOT RE-RUN.** Capture whole, name the exact arm, surface it.

## Prior result to copy for shape

`session.rs:242-292` (the D2 cure) — and its commit, which proved unrepresentability with a compiler
error: `error[E0616]: field 'buckets' of struct 'JoinRightIndex' is private`. Aim for the same class
of proof here.
