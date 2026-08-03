# BRIEF — three fire-time failures become compile-time errors

Makes `constructor_meta` honestly `total: true` at both sites. Prerequisite for arming the third
conjunct. The three paths were measured and reproduced by the audit in `d6c32cf5` — read its message
first; the diagnosis is done and is not yours to redo.

**The builder's framing, and it is the point of the stone:**

> *"This is an amazing forcing function to make our language better — more things at compile time is
> amazing."*

Every fix here has one shape: **a failure that happens at fire moves to freeze.** You are not adding
restrictions. You are moving existing rejections earlier, where they can name the rule that caused
them instead of the reader that tripped over the wreckage.

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it — if the harness moves a long one to the background, that run is lost to
you. Your turn ends when the numbers are in your hands.

## The three

### 1. A nested surface constructor dies at fire with `UnknownFunction`

```clojure
:then [(:usr::Outer :inner (:usr::Inner :x 1))]   ;; --check clean, raises at fire
```

`dispatch_keyword_head_value`'s fallback (`runtime.rs:~6270`) has no arm for a bare aggregate keyword.
Only `build_insert_fact`'s special-cased top-level path knows how to construct one, so a constructor in
any *nested* position is unreachable.

**This one is the odd one out: the fix is to make it WORK, not to reject it.** Give the dispatcher its
arm so a nested constructor evaluates. Nothing about it is illegal — it was simply never wired.

### 2. Under-supplied kwargs silently builds a short record

```clojure
:then [(:usr::Rate :count 7)]                     ;; two-field Rate. --check clean. corpse.
```

`reorder_kwargs_by_field_name` (`validate.rs:~269`) documents that a kwargs RHS *"need not cover every
field"*; `build_insert_fact` (`matcher.rs:~679`) has no independent arity check. Two walls, both
declining, each apparently assuming the other holds. The raise lands later, at whatever reads the
missing field (`Record/field-at`, index-out-of-bounds), naming the reader rather than the rule.

**Add the freeze-time check.** The located error names the rule, the fact type, and the missing fields
by name — not a count.

> **⛔ STOP-A — GROUND WHO RELIES ON THE CURRENT BEHAVIOUR FIRST.** That doc line calls the under-supply
> deliberate and "pre-existing, unchanged". Before you close it, find every `:then` in the corpus that
> under-supplies. If any exist, **report them and stop** — partial construction may be load-bearing
> somewhere (defaults, a staged build), and "require every field" would then be the wrong fix. If none
> exist, the doc line is describing an accident nobody depends on, and closing it is free. Say which
> you found.

### 3. Enum-variant arity is not walled at freeze

Wrong arity on a tagged-variant constructor compiles clean and raises `ArityMismatch` at fire.
`lookup_fields` (`validate.rs:~556`) resolves only `TypeDef::Aggregate`, so a bare `:Enum::Variant`
head is never resolved at compile time. Resolve it, check arity there.

A clean located `ArityMismatch` at fire is a *better* failure than #2's silent corruption — but it is
still a failure the checker could have named, which is the whole objection.

## Then flip the classification

With all three closed, `constructor_meta` (`purity.rs:~612-680`) goes `total: true` at both sites.
`d6c32cf5` rewrote its comment to record the measured `false` and its reasons — **rewrite it again** to
record what closed each one. A stale justification is the debt this arc keeps paying.

**If any of the three cannot close, the corresponding site stays `false`** with the surviving reason
named. Do not flip a site whose path is still open — the audit's whole value was that its `false` was
measured.

## ★ The audit's probes assert the OLD behaviour — re-point, don't delete

`tests/rete/probe_constructor_meta_surface_audit.rs` and its fixtures currently prove *"this compiles
clean and dies at fire."* You are changing that. Their **subject** — that these forms are handled
honestly — survives; their **assertion** inverts: `--check` must now REJECT (#2, #3) or the form must
now WORK (#1).

Re-point them and write into each header what the form used to do, what it does now, and which commit
changed it. A probe whose assertion silently flipped meaning is worse than no probe.

## Prove it both directions

- **REJECT at compile** (#2, #3): `.wat.bad` negative fixtures, each with a located error naming the
  rule and the specific missing field / expected arity. A rejection with a vague message is half a fix.
- **STILL WORKS**: a fully-supplied `:then` item, a correct enum-variant construction, and a nested
  constructor (#1) all compile and derive — through **both** the oracle (`fire-rules-spec`) and the
  native kernel (`fire-rules`).

## Gates — foreground, report each result line

```
cargo build --release --all-targets            # exit 0, ZERO warnings
cargo clippy --release --all-targets           # likewise
cargo test --release --test rete
cargo test --release --test lint
cargo test --release --lib -p wat
./wat-scripts/perf/grid/check-where-shapes.sh  # 9 pairs, 98 rows agreeing
```

Closing a wall lights its violators — **expect the corpus to scream and read the screams as the
worklist.** But if a scream is a legitimate form your new check wrongly rejects, that is a defect in
the check: stop and report rather than loosening it back.

**Do NOT run `cargo nextest run`** — the orchestrator weighs the floor centrally.

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not add `#[allow(dead_code)]` or a
`rune:lint`. Do not arm `total?` — that is #57's. Do not flip a `total` column whose path you did not
actually close.
