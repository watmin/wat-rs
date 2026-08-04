# NOTE — ruling A's SHAPE lock never runs on a parametric response

> # ✅ CLOSED 2026-08-05, hours after it was filed. Kept for the class, not the defect.
>
> **The lock now reaches parametric responses.** `if let TypeExpr::Path(resp_path) = ret`
> became a two-arm match normalizing either variant to the registered base name. Proven by
> re-running the exact fixtures below: the parametric-without-RequestTooLarge case is now
> REFUSED and located, the monomorphic control still refuses, and all four real parametric
> responses in the corpus still pass. Floor `4347/4347/0/262`, clippy clean.
>
> **★ AND THE DEFERRAL WAS THE MISTAKE, which is the part worth keeping.** This note
> originally argued the fix should wait for #75's accessor, on the grounds that arming an
> unarmed lock is a behaviour change that could turn the floor red. That was a *guess wearing
> a risk assessment's clothes* — the census that settles it takes one command and had not
> been run. Run: **4 parametric responses exist in the whole corpus and all 4 already carried
> both mandated variants**, so arming caught nothing and could never have reddened anything.
> The builder cut it in one line — *"it feels like this is a trivial thing to just do now?"* —
> and he was right. **Measure before you pose a deferral; a cost you have not measured is not
> a reason, it is a feeling.** ([[feedback_measure_before_you_pose_the_decision]])
>
> Everything below is the record as filed, unedited. Task #76 is closed against it.

## The defect

`src/types.rs`, in `synthesize_surface_protocol`:

```rust
if enforce_rtl_lock { if let TypeExpr::Path(resp_path) = ret {
    match env.get(resp_path) {
        Some(TypeDef::Aggregate(_)) => { /* records-as-Responses are retired */ }
        Some(TypeDef::Enum(EnumDef { variants, .. })) => {
            /* RequestTooLarge must be well-shaped … RequestMalformed must be well-shaped … */
        }
        _ => {}
    }
} }
```

The whole of ruling A — *every serviceable op-Response is an outcome enum carrying a well-shaped
`RequestTooLarge [bytes cap]` and `RequestMalformed [path expected got]`* — is gated on `ret` being
a **`TypeExpr::Path`**. A parametric response (`GetResponse<K,V>`) is a `TypeExpr::Parametric`, so
it falls straight through to `_ => {}` and **is never checked at all**.

## The proof, with its control

Run 2026-08-05 against `target/release/wat --check`, binary current at `1f730623`.

| fixture | response | RequestTooLarge present? | verdict |
|---|---|---|---|
| `:probe::Hole<K,V>` | `GetResponse<K,V>` | **no** | **ACCEPTED, silently** |
| `:probe::MHole` | `GetResponse` | **no** | REFUSED, located — `#wat.type/MalformedVariant`, *"must carry `:RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]`"* |

The monomorphic twin is the non-vacuity control: it differs from the parametric case in exactly one
respect — the type parameters — and it is refused. So the acceptance above is the hole, not a
quirk of the fixture.

Both fixtures are throwaways; re-create them from this table rather than hunting for them.

## What it means, stated without inflation

Every parametric serviceable Response in the corpus carries the two mandated variants **by author
diligence, not by the lock.** `wat-tests/service-parametric-messages.wat`'s `PCache::GetResponse<K,V>`
has never been checked. Nothing is broken today; the guarantee is simply narrower than the rule it
claims to enforce, and has been since 16.1c landed.

This is the same shape as R59 `NISI FRANGAS NIHIL PROBAS` at the lock layer: the lock has been
reporting a pass for a class it cannot see.

## ★ The class, and why it is worth naming rather than just fixing

**A check that reads one variant of a pair and silently skips its twin.** Three sites in three days:

1. **#75** — `TypeExpr`'s parametric head is stored BARE while a `Path` carries its colon; 137 sites
   match both by hand. The accessor that would make the mistake unwritable is unbuilt.
2. **This note** — the ruling-A lock, `Path` only.
3. **#74's own census, twice over** — it over-counted where the checker holds code as data (macro
   bodies), and under-counted where the corpus is not `.wat` (inline wat in Rust test strings). The
   second one had `req <-` and `-> ` on the same three lines and only one column was read.

So #75 is not cosmetic. **This note is the live instance that proves it** — a real, silent,
shipped-for-weeks gap produced by exactly the hand-matching #75 exists to delete.

## Why it was NOT folded into #74

It sits in the same twenty lines #74 edited and would have been trivially easy to "fix while we're
here." It was left alone deliberately, and #74's brief carried an explicit STOP forbidding the rider
to touch it:

- **It is a different law.** #74 is the response's NAME. This is the response's SHAPE. Two laws that
  happen to share a code region are still two laws.
- **Arming it is a behaviour change** that can turn the floor red with a different error class than
  #74's, which would have made #74's own result unreadable.
- **The ordering is a ruling, not a preference** — and rulings are the builder's.

## The disposition owed

Three questions, none of them the apparatus's to answer:

1. **Strike it, and when** — before #57, after it, or folded into #75's accessor work (which is where
   it would stop being possible to write).
2. **What the arming turns up.** Nobody has measured how many parametric serviceable Responses in
   the corpus would fail the lock once it can see them. That census is cheap and is owed *before* a
   brief, not during one.
3. **Whether #75 subsumes it.** If `TypeExpr` gains the accessor that returns a normalized base name,
   this defect and its 137 siblings close together, and striking this one alone may be wasted motion.

Until that ruling: the lock stays as it is, and **it must not be described as covering parametric
responses**, here or anywhere else.
