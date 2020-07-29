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
    // Restricting T to IntoIterator<Item=S> here wouldn't allow
    // from_sorted_iter to be run on an iterator that yields references to
    // S... a common use case. So we just make sure that we're iterating over
    // spans that can be converted into S.
    pub fn from_sorted_iter<T: IntoIterator>(iterable: T) -> VecSpet<S>
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
                        spans.push(S::new(
                            awaiting_start.unwrap(),
                            span.end().clone().into()));
                        awaiting_start = None;
                    }
                },
            }
        }

        VecSpet { spans }
    }

    pub fn union(&self, other: &VecSpet<S>) -> VecSpet<S> {
        Self::from_sorted_iter(
            sorted_chain(&mut [self.spans.iter(), other.spans.iter()]))
    }

    pub fn intersection(&self, other: &VecSpet<S>) -> VecSpet<S> {
        crate::overlapping::n_overlapping(2, vec![self, other])
    }
}


impl<S: CreatableSpan> IntoIterator for VecSpet<S> {
    type Item = S;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.spans.into_iter()
    }
}


impl<'a, S: CreatableSpan> IntoIterator for &'a VecSpet<S> {
    type Item = &'a S;
    type IntoIter = std::slice::Iter<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.spans.iter()
    }
}


#[cfg(test)]
mod tests {
    mod from_sorted_iter {
        use crate::vecspet::VecSpet;
        use crate::span::{SimpleSpan, CreatableSpan};

        #[test]
        fn consuming() {
            let spans = vec![
                SimpleSpan::new(1, 5),
                SimpleSpan::new(3, 7),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::from_sorted_iter(spans);
            assert_eq!(result.spans, vec![SimpleSpan::new(1, 7)]);
        }

        #[test]
        fn visiting() {
            let spans = vec![
                SimpleSpan::new(1, 5),
                SimpleSpan::new(3, 7),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::from_sorted_iter(&spans);
            assert_eq!(result.spans, vec![SimpleSpan::new(1, 7)]);
        }

        #[test]
        fn single() {
            let spans = vec![
                SimpleSpan::new(1, 2),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::from_sorted_iter(&spans);
            assert_eq!(result.spans, vec![SimpleSpan::new(1, 2)]);
        }

        #[test]
        fn non_overlapping() {
            let spans = vec![
                SimpleSpan::new(1, 2),
                SimpleSpan::new(3, 5),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::from_sorted_iter(&spans);
            assert_eq!(result.spans, vec![
                SimpleSpan::new(1, 2),
                SimpleSpan::new(3, 5),
            ]);
        }

        #[test]
        fn nesting_subsets() {
            let spans = vec![
                SimpleSpan::new(1, 6),
                SimpleSpan::new(2, 5),
                SimpleSpan::new(3, 4),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::from_sorted_iter(&spans);
            assert_eq!(result.spans, vec![
                SimpleSpan::new(1, 6),
            ]);
        }

        #[test]
        fn duplication() {
            let spans = vec![
                SimpleSpan::new(1, 2),
                SimpleSpan::new(1, 2),
            ];

            let result: VecSpet<SimpleSpan<i32>> =
                VecSpet::from_sorted_iter(&spans);
            assert_eq!(result.spans, vec![
                SimpleSpan::new(1, 2),
            ]);
        }
    }

    mod union {
        use crate::vecspet::VecSpet;
        use crate::span::{SimpleSpan, CreatableSpan};

        #[test]
        fn simple() {
            let a = VecSpet { spans: vec![SimpleSpan::new(1, 5)] };
            let b = VecSpet { spans: vec![SimpleSpan::new(3, 7)] };

            let result = a.union(&b);
            assert_eq!(result.spans, vec![SimpleSpan::new(1, 7)]);
        }
    }

    mod intersection {
        use crate::vecspet::VecSpet;
        use crate::span::{SimpleSpan, CreatableSpan};

        #[test]
        fn simple() {
            let a = VecSpet { spans: vec![SimpleSpan::new(1, 5)] };
            let b = VecSpet { spans: vec![SimpleSpan::new(3, 7)] };

            let result = a.intersection(&b);
            assert_eq!(result.spans, vec![SimpleSpan::new(3, 5)]);
        }
    }
}
