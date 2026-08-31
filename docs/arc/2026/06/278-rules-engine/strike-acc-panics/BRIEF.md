# BRIEF — a wire-reachable invariant may not be spelled `panic!`

Turn `acc.rs`'s host panics into refusals the caller can match, so an imported network cannot
unwind the process. `acc_var_i64` and its siblings return `Result<_, EvalBreak>` carrying a
`MalformedForm`; the accumulate pass, which already returns `Result`, propagates with `?`. Read
`DESIGN.md` beside this file first — its ★ ONE CONTRACT DECISION governs every message you write,
and its arm table tells you which panic the probe actually reaches.

## Read in order, and why

1. **`src/rete/kernel/fire/acc.rs:55-90`** — `acc_var_i64`, its doc, and its rune. Five panic arms
   live here. The doc's *"compile-time-impossible shape"* and the rune's *"AccFold compile proved
   i64"* are the false licence; both change in this edit.
2. **`src/rete/kernel/fire/acc.rs:120-145`** — the slot path, four more arms (`:129`, `:139`,
   `:140`, `:142`). Same class, same treatment.
3. **`src/rete/kernel/fire/mod.rs:235-244`** — `driver_of`. **This is the shape to copy**: a
   missing id in a setup-populated table, returning `Err(MalformedForm)` with a reason that names
   the invariant. Do not invent a new error shape; this one is the precedent.
4. **`src/rete/kernel/fire/pass/accumulate.rs`** — the callers. Confirm the enclosing fns already
   return `Result`; the propagation should be `?` and nothing more. If any caller does not, say so
   before changing its signature.
5. **`src/rete/export.rs`, `unpack_fold`'s `:sum` arm** — where the key enters from the wire. You
   are not changing it (see DESIGN's out-of-scope); read it so your refusal message can name where
   the value came from.
6. **`strike-acc-panics/probe.rs.txt` and `probe.wat.txt`** — the probe pair, written and driven at
   HEAD `788e5b66d`. Copy them to `tests/rete/probe_arc278_import_fold_key.{rs,wat}`. The `.wat`
   is a new fixture: the corpus had no keyed fold inside a rule that also exports.

## The probe pair, and what each half is for

- **`fold_key_fixture_native_and_imported_agree`** — the control. Untampered, native and imported
  fire must agree, and the rule carries a `where` fence pinning the fold's value at 30, so the
  count it asserts cannot pass on a silently wrong sum. **This must be green before and after.**
- **`import_refuses_a_fold_key_no_condition_binds`** — the disconfirming half, RED today. It
  rewrites the `:sum` fold's key to a keyword nothing binds, imports, fires, and fails if the call
  unwinds. After your change it must take the `Ok(Err(_))` arm — refused as a value.

## Implementation sketch

```rust
pub(super) fn acc_var_i64(
    el: &Element, var: &Value, view: &AccView<'_>,
) -> Result<i64, EvalBreak> {
    // each former panic! becomes:
    //   return Err(acc_refusal(format!("...")));
    // with a helper mirroring driver_of's construction
}
```

Callers at `acc.rs:290,298,317,334` become `?`. Note `:290` is inside an iterator
(`gathered.iter().map(|el| acc_var_i64(el, var, view))`) — that map now yields `Result`, so it
needs `collect::<Result<Vec<_>,_>>()?` or an equivalent; that is the one non-mechanical spot and
it is why it is called out here.

## Blast radius

`src/rete/kernel/fire/acc.rs`, `src/rete/kernel/fire/pass/accumulate.rs`, and the two probe files.
No wire-format change, no version bump.

## STOP triggers — halt and surface, do not improvise

1. **If a caller of `acc_var_i64` does NOT already return `Result`, STOP** and surface it before
   widening any signature beyond `acc.rs` and `accumulate.rs`. The blast radius above is a claim
   about the call graph; if it is wrong, the strike needs redrawing, not stretching.
2. **If any refusal message you write says "impossible", "cannot happen", or names the compiler as
   the reason, STOP.** That sentence is the defect. The ★ ONE CONTRACT DECISION is that the
   refusal names the door.
3. **If the control test (`..._native_and_imported_agree`) is not green BEFORE you change
   anything, STOP** — the fixture did not survive the copy and the probe is measuring nothing.
4. **If making the folds return `Result` forces a change to the `AccFold` enum or the wire
   format, STOP.** That is a different strike; DESIGN cuts it explicitly.

## The mutation proof — ONE PER ARM, and the probe reaches only one

This is the step A1's strike got wrong, and the correction is the whole point of doing it here.

The probe reaches **`acc.rs:72` only** — the packed `slot_keys` arm — because its fixture takes
the packed path (`el.binds.len == 0`). It proves that arm and no other.

Required:
- **Arm `:72`** — RED→GREEN across your change is the proof. Free; you already have it.
- **Arms `:64`/`:65`** (the `Bindings::get` path) — these need `el.binds.len > 0`. Either extend
  the fixture so a second rule's accumulate takes the unpacked path, or drive them directly.
  **If you cannot reach them, say so explicitly and name them as unproven** — an unreached arm
  reported as unreached is an honest result; an unreached arm silently shipped is not.
- **The remaining arms** (`:76`, `:83`, `:129`, `:139`, `:140`, `:142`) — report which are
  reachable from a tampered Export and which are not. You are not required to reach all of them.
  You are required to **say which you reached**.

A count of arms converted is not the deliverable. Which arms are *proven* is.

## A prior comparable result to copy for shape

`strike-import-graph-wall/` — the same arc, landed as `788e5b66d`: probe applied first, RED
confirmed and quoted verbatim, wall written, GREEN, then deliberate mutation. Its rider's report is
the standard for the honest-deltas section, and its lesson is the arm table in `DESIGN.md`.
