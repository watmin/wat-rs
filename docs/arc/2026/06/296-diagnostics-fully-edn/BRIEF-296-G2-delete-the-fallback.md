# BRIEF — 296 G-2: delete the `field-N` fallback

G landed: `AggregateValue` carries `names`. The renderer's registry lookup now has nothing to do —
every aggregate arrives already knowing what its fields are called. **Delete the fallback; there is
nothing left to fall back *from*.**

Floor at brief time: **4417 / 4417**, clippy 0, HEAD `f64e1136`.

## ⛔ THE SEVEN SITES ARE NOT ONE THING — measured, 2026-08-15

The design stone said "the 7 `format!("field-{}", i)` sites go." That was written before the split
was visible. Measured, they are **6 aggregate + 1 enum**, and only the aggregate half is solved:

| site | arm | solved by G? |
|---|---|---|
| `edn_shim.rs:2692` | `Value::Aggregate` (json-natural, the `_ =>` catch-all) | **yes** |
| `edn_shim.rs:2703` | `Value::Aggregate` (json-natural, per-field `unwrap_or_else`) | **yes** |
| `edn_shim.rs:3676` | `Value::Aggregate` (edn_with, the `_ =>` catch-all) | **yes** |
| `edn_shim.rs:3686` | `Value::Aggregate` (edn_with, per-field) | **yes** |
| `edn_shim.rs:3837` | `Value::Aggregate` (edn_with, the `_ =>` catch-all) | **yes** |
| `edn_shim.rs:3847` | `Value::Aggregate` (edn_with, per-field) | **yes** |
| `edn_shim.rs:2736` | **`Value::Enum`** (json-natural) | **NO — out of scope, see below** |

**This stone is the six.** The seventh is a different defect with a different fix and is ruled out of
scope below — affirmatively, with the measurement that says why.

## THE WORK

Each aggregate arm currently does a registry lookup to recover names, then falls back positionally.
Both halves go:

```rust
// BEFORE — a lookup that can fail four ways, collapsing into one lie
let type_key = format!(":{}", sv.class);
let field_names: Vec<String> = match types.and_then(|t| t.get(&type_key)) {
    Some(TypeDef::Aggregate(a)) if a.nature == Nature::Struct => { … }
    _ => (0..sv.fields.len()).map(|i| format!("field-{}", i)).collect(),
};
… field_names.get(i).cloned().unwrap_or_else(|| format!("field-{}", i))

// AFTER — the value already knows
… sv.names.iter().zip(sv.fields.iter())
```

Zipping `names` against `fields` also removes the `.get(i).unwrap_or_else(…)` per-field guard: the
two are the same length by construction, so there is no index to miss. **Prefer `zip` over indexing**
— it makes the arity invariant structural rather than checked.

Where `types` was threaded into one of these functions *only* to look up field names, it becomes
unused; drop the parameter if nothing else in the function needs it, and let the compiler tell you
which callers change. Where `types` is still needed for recursion into nested values, keep it.

## THE PREDICTED SCREAMS — an earlier session already measured this

An earlier session deleted these fallbacks **without** G and read the result: **4 reds out of 4413**,
every one real.

- a heretic test that pinned `{:field-0 3 :field-1 4}` as its **expected** value
- a CLI freeze-panic path whose doc comment claimed *"those values only carry primitive Strings"* —
  **false**
- two self-inflicted

That measurement is the disconfirming evidence that this fallback is not load-bearing. With G landed,
expect **fewer** reds than that, not more — the names now arrive with the value. Any red here is a
site that was depending on the lie; name it, don't paper over it.

## ⛔ OUT OF SCOPE, AFFIRMATIVELY — the enum half is a DIFFERENT defect

Not deferred; **cut, with the reason measured.** Do not touch `edn_shim.rs:2736` or
`enum_variant_field_names`.

`EnumValue { type_path, variant_name, fields }` has no `names`, and G did not give it any. Its three
silent `return vec![]` arms (no `types`; the path is not an Enum in the registry; the variant is not
found) each turn into `field-N` at the one call site. That is a real, user-reachable lie —
`value_to_json_natural` is live, reached from `eval_edn_write_json_natural`.

**But the enum fix is not "carry names," and that is the point.** The canonical EDN rendering of a
variant (`edn_shim.rs:3699`) is deliberately **positional**: `#tag [v1 v2]`, a tagged *vector*, and
its own comment says why — *"body-shape is a perfect discriminator (map=record, vector=variant,
nil=unit)."* Names would break the discriminator. So the EDN form is correct as it stands and must
not change, and only the JSON-convenience view wants names at all.

Which means the enum question is: **what should a view do when it cannot name a field?** — with
`EnumValue` carrying names (106 construction sites, for a form whose canonical rendering does not use
them) as one option and the three empty-vec arms raising as another. That is a ruling, not a sweep,
and it belongs to the builder. Tracked, not forgotten.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — an aggregate arm where `names` is absent or the wrong length.** The two are built
  together by construction. A disagreement means a construction site assembled them from different
  places, and *that* site is the finding.
- **STOP-2 — a red whose fix would be to re-introduce positional naming under any spelling**
  (`field-N`, `"0"`, an index, a placeholder). That is the defect returning under a new name. Report
  the site instead.
- **STOP-3 — a test that pins `field-N` as its EXPECTED value.** Do not "fix" it by keeping the
  fallback. Ask what the test MEASURES: if its subject survives, re-express the expectation against
  the real names; if its only subject was the fallback, it retires with it. Say which and why.

## BLAST RADIUS

`src/edn_shim.rs` and whatever the compiler names downstream of a dropped `types` parameter. No
`.wat` corpus changes. Do not touch `Value::Enum` handling, `enum_variant_field_names`, or the two
positional labels ruled correct in G-1b (`runtime.rs:13880` — a Newtype's `"0"` **is** its accessor
name `<Type>/0`; `runtime.rs:34883` — a unit test's synthetic class that never renders).

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.
Baseline **4417 passed / 0 failed / 263 skipped**.

**On any red: do NOT re-run.** A re-run that goes green destroys the only evidence. Copy the failing
test's entire stdout+stderr block verbatim — never a `| head` window — name the exact assertion that
fired, and report.

Finish by confirming the count: `grep -c 'format!("field-{}", i)' src/edn_shim.rs` should be **1**
(the enum site, out of scope), down from 7.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the FOREGROUND and block on it. Anchor at `/home/watmin/work/holon/wat-rs`;
`pwd` first. Leave the work uncommitted; the orchestrator weighs and commits.

Report: the six sites as you found them, every red with its verbatim block and disposition, the floor
Summary line, the final `field-N` count, and the honest deltas — especially anywhere this brief did
not match the disk. Both riders before you caught a defect in the orchestrator's brief; that is the bar.
