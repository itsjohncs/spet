mod mergeiter;
mod span;
mod pivots;
use span::Span;

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
// struct SimpleSpan<T> {
//     start: T,
//     end: T,
// }


struct VecSpet<S: Span> {
    spans: Vec<S>
}


impl<'a, S: 'a + Span> VecSpet<S> where S::Domain: Clone {
    /**
     * Creates a new VecSpet from a sorted iterator of spans.
     */
    fn collect_from_sorted(mut iter: impl Iterator<Item = &'a S>) -> VecSpet<S> {
        let mut spans = Vec::<S>::new();

        let first_span = if let Some(span) = iter.next() {
            span
        } else {
            return VecSpet { spans };
        };

        // start and end are candidate values for the next span we'll
        // push onto our vector.
        let mut start = first_span.start();
        let mut end = first_span.end();

        for span in iter {
            // If the span doesn't intersect with our candidate span
            // (Span { start, end })...
            if span.start() > end {
                // We create the candidate because it won't be extended anymore
                // (no other span is going to start before this one since we're
                // iterating over start values in ascending order).
                spans.push(S::create(start.clone(), end.clone()));
                start = &span.start();
                end = &span.end();
            } else {
                // Union our candidate with the current span (extending it if
                // necessary).
                end = Ord::max(end, span.end());
            }
        }

        spans.push(S::create(start.clone(), end.clone()));

        VecSpet {spans}
    }


    fn iter(&self) -> std::slice::Iter<S> {
        self.spans.iter()
    }


    fn union(&self, other: &VecSpet<S>) -> VecSpet<S> {
        VecSpet::collect_from_sorted(
            mergeiter::sorted_chain([self.iter(), other.iter()].iter_mut()))
    }
}


// impl<T: Ord + Clone> From<Vec<Span<T>>> for VecSpet<T> {
//     fn from(mut vector: Vec<Span<T>>) -> VecSpet<T> {
//         vector.sort_unstable();
//         VecSpet::collect_from_sorted(vector.iter())
//     }
// }


// impl<T: Ord + Clone> Into<Vec<Span<T>>> for VecSpet<T> {
//     fn into(self) -> Vec<Span<T>> {
//         self.spans
//     }
// }


impl<S: Span> IntoIterator for VecSpet<S> {
    type Item = S;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.spans.into_iter()
    }
}


// struct SpetPivotIterator<T: Ord, I: Iterator<Item = Span<T>> {
//     spans: I,
//     BinaryHeap<Reverse<&T>>
// }


// #[cfg(test)]
// mod tests {
//     use crate::{Span, VecSpet, SortedMergeIter};

//     fn create_spans(tuple: Vec<(usize, usize)>) -> Vec<Span<usize>> {
//         tuple.into_iter()
//              .map(|(start, end)| Span { start, end })
//              .collect()
//     }

//     #[test]
//     fn sorted_merge_iter() {
//         let result = SortedMergeIter::from(vec![
//             vec![1, 4],
//             vec![1, 2],
//             vec![1, 3],
//         ]);
//         assert_eq!(result.collect::<Vec<i32>>(), vec![1, 1, 1, 2, 3, 4]);
//     }

//     #[test]
//     fn from_single() {
//         let spans = create_spans(vec![(0, 1)]);
//         let result: Vec<_> = VecSpet::from(spans.clone()).into();
//         assert_eq!(spans, result);
//     }

//     #[test]
//     fn from_many_unsorted() {
//         let result: Vec<_> = VecSpet::from(
//             create_spans(vec![(3, 4), (0, 1), (5, 6)])).into();
//         assert_eq!(create_spans(vec![(0, 1), (3, 4), (5, 6)]), result);
//     }

//     #[test]
//     fn from_many_unsorted_overlapping() {
//         let result: Vec<_> = VecSpet::from(
//             create_spans(vec![(3, 4), (0, 1), (4, 6)])).into();
//         assert_eq!(create_spans(vec![(0, 1), (3, 6)]), result);
//     }

//     #[test]
//     fn union() {
//         let spet = VecSpet::from(create_spans(vec![(0, 1)]));
//         let result: Vec<_> = spet.union(
//             &VecSpet::from(create_spans(vec![(2, 3)]))).into();
//         assert_eq!(create_spans(vec![(0, 1), (2, 3)]), result);
//     }

//     #[test]
//     fn union_overlapping() {
//         let spet = VecSpet::from(create_spans(vec![(1, 2)]));
//         let result: Vec<_> = spet.union(
//             &VecSpet::from(create_spans(vec![(0, 4)]))).into();
//         assert_eq!(create_spans(vec![(0, 4)]), result);
//     }
// }
