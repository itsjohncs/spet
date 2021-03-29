# Spet

A **spet** is a set of spans (aka intervals). Ex: `{1.2–4, 8.1–8.4}`

Set operations behave intuitively. Ex:

* `{1.2–4, 8.1–8.4} ∪ {3–8.2} = {1.2–8.4}`
* `{1.2–4, 8.1–8.4} ∩ {3–8.2} = {3–4, 8.1–8.2}`

Spets are especially useful when working with timespans (ex: figuring out when multiple users are viewing a collaborative document by analyzing server logs).

# Construction

Use `SimpleSpan::new` to construct a span (`CreatableSpan` trait must be in scope).

```rust
use spet::{SimpleSpan, CreatableSpan};

let my_span: SimpleSpan<f64> = SimpleSpan::new(1.2, 4);
```

Use `VecSpet::from_sorted_iter` to construct a spet.

```rust
use spet::{VecSpet, SimpleSpan, CreatableSpan};

let spans = vec![
    SimpleSpan::new(1.2, 4),
    SimpleSpan::new(8.1, 8.4),
];

let my_spet: VecSpet<SimpleSpan<f64>> =
    VecSpet::from_sorted_iter(spans);
```

# Mutability

A `VecSpet` is immutable once constructed. All operations create new spets.

```rust
let a_new_spet = a.union(b);
```

A tree-based mutable spet implementation could be made: PRs welcome.

# n_overlapping

`n_overlapping` efficiently finds the spans where multiple spets intersect.

If you had spets containing the times that the users A, B, C, and D were logged in, you could find the times where at least 2 of them were logged in at the same time with `n_overlapping(2, vec![A, B, C, D])`.

If you wanted the times when at least 3 of them were logged in: `n_overlapping(3, vec![A, B, C, D])`.
