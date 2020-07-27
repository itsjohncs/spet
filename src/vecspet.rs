use std::fmt::Debug;
use crate::span::{Span, CreatableSpan};
use crate::points::{enumerate_points, Point::{StartOf, EndOf}};
use crate::mergeiter::sorted_chain;


#[derive(PartialEq, Eq)]
pub struct VecSpet<S: CreatableSpan> {
    spans: Vec<S>
}


impl<S: CreatableSpan + Debug> Debug for VecSpet<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.spans.iter()).finish()
    }
}


impl<S: CreatableSpan> VecSpet<S> {
    // Consumes a vector and transforms it into a vecspet (will not reuse the
    // space allocated by the vector, but will sort it in-place).
    fn from_vec(mut src: Vec<S>) -> VecSpet<S> {
        src.sort_unstable();
        Self::collect_from_sorted(src)
    }

    // Restricting T to IntoIterator<Item=S> here wouldn't allow
    // collect_from_sorted to be run on an iterator that yields references to
    // S... a common use case. So we just make sure that we're iterating over
    // spans that can be converted into S.
    fn collect_from_sorted<T: IntoIterator>(iterable: T) -> VecSpet<S>
            where T::Item: Span,
                  <T::Item as Span>::Domain: Into<S::Domain> {
        let mut spans = Vec::<S>::new();

        let mut awaiting_start: Option<S::Domain> = None;
        let mut count = 0;
        for point in enumerate_points(iterable) {
            match point {
                StartOf(span) => {
                    if count == 0 {
                        assert!(awaiting_start.is_none());
                        awaiting_start = Some(span.start().clone().into());
                    }
                    count += 1;
                },
                EndOf(span) => {
                    count -= 1;
                    if count == 0 {
                        assert!(awaiting_start.is_some());
                        spans.push(S::create(
                            awaiting_start.unwrap(),
                            span.end().clone().into()));
                        awaiting_start = None;
                    }
                },
            }
        }

        VecSpet { spans }
    }

    fn union(&self, other: &VecSpet<S>) -> VecSpet<S> {
        Self::collect_from_sorted(
            sorted_chain(&mut [self.spans.iter(), other.spans.iter()]))
    }
}


#[cfg(test)]
mod tests {
    mod from_vec {
        use crate::vecspet::VecSpet;
        use crate::span::{SimpleSpan, CreatableSpan};

        #[test]
        fn unsorted() {
            let spans = vec![
                SimpleSpan::create(10, 12),
                SimpleSpan::create(3, 7),
            ];
            let result = VecSpet::from_vec(spans);
            assert_eq!(result.spans, vec![
                SimpleSpan::create(3, 7),
                SimpleSpan::create(10, 12),
            ]);
        }
    }

    mod collect_from_sorted {
        use crate::vecspet::VecSpet;
        use crate::span::{SimpleSpan, CreatableSpan};

        #[test]
        fn consuming() {
            let spans = vec![
                SimpleSpan::create(1, 5),
                SimpleSpan::create(3, 7),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::collect_from_sorted(spans);
            assert_eq!(result.spans, vec![SimpleSpan::create(1, 7)]);
        }

        #[test]
        fn visiting() {
            let spans = vec![
                SimpleSpan::create(1, 5),
                SimpleSpan::create(3, 7),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::collect_from_sorted(&spans);
            assert_eq!(result.spans, vec![SimpleSpan::create(1, 7)]);
        }

        #[test]
        fn single() {
            let spans = vec![
                SimpleSpan::create(1, 2),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::collect_from_sorted(&spans);
            assert_eq!(result.spans, vec![SimpleSpan::create(1, 2)]);
        }

        #[test]
        fn non_overlapping() {
            let spans = vec![
                SimpleSpan::create(1, 2),
                SimpleSpan::create(3, 5),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::collect_from_sorted(&spans);
            assert_eq!(result.spans, vec![
                SimpleSpan::create(1, 2),
                SimpleSpan::create(3, 5),
            ]);
        }

        #[test]
        fn nesting_subsets() {
            let spans = vec![
                SimpleSpan::create(1, 6),
                SimpleSpan::create(2, 5),
                SimpleSpan::create(3, 4),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::collect_from_sorted(&spans);
            assert_eq!(result.spans, vec![
                SimpleSpan::create(1, 6),
            ]);
        }

        #[test]
        fn duplication() {
            let spans = vec![
                SimpleSpan::create(1, 2),
                SimpleSpan::create(1, 2),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::collect_from_sorted(&spans);
            assert_eq!(result.spans, vec![
                SimpleSpan::create(1, 2),
            ]);
        }
    }

    mod union {
        use crate::vecspet::VecSpet;
        use crate::span::{SimpleSpan, CreatableSpan};

        #[test]
        fn simple() {
            let a = VecSpet { spans: vec![SimpleSpan::create(1, 5)] };
            let b = VecSpet { spans: vec![SimpleSpan::create(3, 7)] };

            let result = a.union(&b);
            assert_eq!(result.spans, vec![SimpleSpan::create(1, 7)]);
        }
    }
}
