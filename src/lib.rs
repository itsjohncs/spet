use std::cmp::Reverse;


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
struct Span<T> {
    start: T,
    end: T,
}

struct PeekedIter<T: Iterator> where T::Item: Ord {
    value: T::Item,
    iterator: T,
}

impl<T: Iterator> Eq for PeekedIter<T> where T::Item: Ord {}

impl<T: Iterator> Ord for PeekedIter<T> where T::Item: Ord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Iterator> PartialOrd for PeekedIter<T> where T::Item: Ord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Iterator> PartialEq for PeekedIter<T> where T::Item: Ord {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}


struct SortedMergeIter<T: Iterator> where T::Item: Ord {
    // A min-heap (thanks to Reverse) that'll tell us the next smallest value
    // that's waiting for us among our iterators.
    queue: std::collections::BinaryHeap<Reverse<PeekedIter<T>>>,
}


impl<T: Iterator> SortedMergeIter<T> where T::Item: Ord {
    // I wanted to implement From here, but I was having trouble getting the
    // compiler to not consider this implementation as conflicting with the
    // definition of From<T> for T... I think it's possible, but this is fine
    // for the time being.
    fn from_iterators<I: IntoIterator<IntoIter = T, Item = T::Item>, A: IntoIterator<Item = I>>(containers: A) -> SortedMergeIter<T> {
        // Could look at iterators.size_hint() and then use with_capacity...
        let mut result = SortedMergeIter::<T> { queue: std::collections::BinaryHeap::new() };
        for container in containers.into_iter() {
            let mut iterator = container.into_iter();
            if let Some(value) = iterator.next() {
                result.queue.push(Reverse(PeekedIter { value, iterator }));
            }
        }

        result
    }
}


impl<T: Iterator> Iterator for SortedMergeIter<T> where T::Item : Ord {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Reverse(mut peeked_iter)) = self.queue.pop() {
            if let Some(value) = peeked_iter.iterator.next() {
                self.queue.push(Reverse(PeekedIter {
                    value,
                    iterator: peeked_iter.iterator
                }));
            }

            Some(peeked_iter.value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut queue_iter = self.queue.iter();
        let (mut min, mut max) = if let Some(Reverse(peeked_iter)) = queue_iter.next() {
            peeked_iter.iterator.size_hint()
        } else {
            return (0, Some(0));
        };
        for reversed_peeked_iter in queue_iter {
            let Reverse(peeked_iter) = reversed_peeked_iter;
            let (iter_min, iter_max) = peeked_iter.iterator.size_hint();
            min = usize::checked_add(min, iter_min).unwrap_or(usize::max_value());
            max = if let(Some(i), Some(j)) = (max, iter_max) {
                // We'll only return a value in the case that both iterators have
                // an upper bound and they don't overflow when we add them
                // together (this is the most likely case).
                usize::checked_add(i, j)
            } else {
                None
            }
        }

        (min, max)
    }
}


// Merges two ascending-sorted iterators into a single ascending-sorted
// iterator. Only used internally, so it's not designed to be particularly
// ergonomic.
struct SortedMergedIter<T: Iterator> where T::Item : PartialOrd {
    left: std::iter::Peekable<T>,
    right: std::iter::Peekable<T>,
}


impl<T: Iterator> Iterator for SortedMergedIter<T> where T::Item : PartialOrd {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.left.peek(), self.right.peek()) {
            (Some(l), Some(r)) if l > r => self.right.next(),
            (Some(_), Some(_)) => self.left.next(),
            (Some(_), None) => self.left.next(),
            (None, Some(_)) => self.right.next(),
            (None, None) => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (left_min, left_max) = self.left.size_hint();
        let (right_min, right_max) = self.right.size_hint();

        let min = usize::checked_add(left_min, right_min).unwrap_or(usize::max_value());
        let max = if let (Some(l), Some(r)) = (left_max, right_max) {
            // We'll only return a value in the case that both iterators have
            // an upper bound and they don't overflow when we add them
            // together (this is the most likely case).
            usize::checked_add(l, r)
        } else {
            None
        };

        (min, max)
    }
}


struct VecSpet<T: Ord> {
    spans: Vec<Span<T>>
}


impl<'a, T: 'a + Ord + Clone> VecSpet<T> {
    /**
     * Creates a new VecSpet from a sorted iterator of spans.
     *
     * Note that this doesn't check that the iterator is sorted.
     */
    fn collect_from_sorted<I>(mut iter: I) -> VecSpet<T>
            where I: Iterator<Item = &'a Span<T>> {
        let mut spans = Vec::<Span<T>>::new();

        let first_span = if let Some(span) = iter.next() {
            span
        } else {
            return VecSpet { spans };
        };

        // start and end are candidate values for the next span we'll
        // push onto our vector.
        let mut start = &first_span.start;
        let mut end = &first_span.end;

        for span in iter {
            // If the span doesn't intersect with our candidate span
            // (Span { start, end })...
            if span.start > *end {
                // We create the candidate because it won't be extended anymore
                // (no other span is going to start before this one since we're
                // iterating over start values in ascending order).
                spans.push(Span { start: start.clone(), end: end.clone() });
                start = &span.start;
                end = &span.end;
            } else {
                // Union our candidate with the current span (extending it if
                // necessary).
                end = Ord::max(end, &span.end);
            }
        }

        spans.push(Span{ start: start.clone(), end: end.clone() });

        VecSpet {spans}
    }


    fn iter(&self) -> std::slice::Iter<Span<T>> {
        self.spans.iter()
    }


    fn union(&self, other: &VecSpet<T>) -> VecSpet<T> {
        VecSpet::collect_from_sorted(SortedMergedIter {
            left: self.iter().peekable(),
            right: other.iter().peekable(),
        })
    }

    // Worst case is O(n*log(n)), best case is O(n) (amortized since we're
    // growing a vector to house the result). Performance slows within these
    // bounds as the number of spans that intersect eachother increases. So if
    // you have a pair of sets (A, B) where every set in A intersects with
    // every set in B you'll get O(n*log(n)).
    //
    // I think this ought to generally perform faster than an equivalent
    // algorithm that breaks up every span then sorts + iterates those, though
    // I'm not sure... That likely performs better in the worst case because of
    // quicksort's features. But the heap likely wins in more ideal cases
    // because the heap will stay very small.
    // fn intersection(&self, other: &VecSpet<T>) -> VecSpet<T> {
    //     use std::collections::BinaryHeap;
    //     use std::cmp::Reverse;

    //     let mut spans = Vec::<Span<T>>::new();

    //     let merged = SortedMergedIter {
    //         left: self.iter(),
    //         right: other.iter(),
    //     };

    //     // Once we detect that we've entered a span that will exist in the
    //     // result (ie: we're within two or more spans in merged) we'll set
    //     // this.
    //     let mut start = None;

    //     // Conceptualize our loop as ascending one-by-one through all the
    //     // possible values of T. This heap will store the ends of all of the
    //     // spans we're currently within. If there's 2 or more ends in this heap
    //     // then we know we're within a span in our result.
    //     let mut ends: BinaryHeap<&T> = BinaryHeap::new();

    //     for span in merged {
    //         ends.push(span.end);

    //         while let Some(end) = ends.peek() {
    //             if span.start > *end {
    //                 // If we're popping off an end that'll bring us from having
    //                 // 2+ ends in the heap to less than 2, it means we've found
    //                 // an end of a span in our result.
    //                 if ends.len() == 2 {
    //                     spans.push(Span {
    //                         // ends will never grow beyond 1 element without
    //                         // start being set to some value.
    //                         start: start.unwrap().clone(),
    //                         end: end.clone()
    //                     });
    //                     start = None;
    //                 }
    //                 ends.pop();
    //             } else {
    //                 break;
    //             }
    //         }

    //         while let Some(end) = ends.peek() {
    //             if span.start > *end {

    //             }
    //         }
    //     }
    // }
}


impl<T: Ord + Clone> From<Vec<Span<T>>> for VecSpet<T> {
    fn from(mut vector: Vec<Span<T>>) -> VecSpet<T> {
        vector.sort_unstable();
        VecSpet::collect_from_sorted(vector.iter())
    }
}


impl<T: Ord + Clone> Into<Vec<Span<T>>> for VecSpet<T> {
    fn into(self) -> Vec<Span<T>> {
        self.spans
    }
}


impl<T: Ord> IntoIterator for VecSpet<T> {
    type Item = Span<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.spans.into_iter()
    }
}


#[cfg(test)]
mod tests {
    use crate::{Span, VecSpet, SortedMergedIter, SortedMergeIter};

    fn create_spans(tuple: Vec<(usize, usize)>) -> Vec<Span<usize>> {
        tuple.into_iter()
             .map(|(start, end)| Span { start, end })
             .collect()
    }

    #[test]
    fn sorted_merged_iter() {
        let a = vec![0, 1, 2, 3, 4];
        let b = vec![3, 4, 5, 6, 7];
        let merged = SortedMergedIter {
            left: a.into_iter().peekable(),
            right: b.into_iter().peekable()
        };
        assert_eq!(merged.collect::<Vec<_>>(),
                   vec![0, 1, 2, 3, 3, 4, 4, 5, 6, 7]);
    }

    #[test]
    fn foo() {
        let result = SortedMergeIter::from_iterators(vec![
            vec![1, 4],
            vec![1, 2],
            vec![1, 3],
        ]);
        assert_eq!(result.collect::<Vec<i32>>(), vec![1, 1, 1, 2, 3, 4]);
    }

    #[test]
    fn from_single() {
        let spans = create_spans(vec![(0, 1)]);
        let result: Vec<_> = VecSpet::from(spans.clone()).into();
        assert_eq!(spans, result);
    }

    #[test]
    fn from_many_unsorted() {
        let result: Vec<_> = VecSpet::from(
            create_spans(vec![(3, 4), (0, 1), (5, 6)])).into();
        assert_eq!(create_spans(vec![(0, 1), (3, 4), (5, 6)]), result);
    }

    #[test]
    fn from_many_unsorted_overlapping() {
        let result: Vec<_> = VecSpet::from(
            create_spans(vec![(3, 4), (0, 1), (4, 6)])).into();
        assert_eq!(create_spans(vec![(0, 1), (3, 6)]), result);
    }

    #[test]
    fn union() {
        let spet = VecSpet::from(create_spans(vec![(0, 1)]));
        let result: Vec<_> = spet.union(
            &VecSpet::from(create_spans(vec![(2, 3)]))).into();
        assert_eq!(create_spans(vec![(0, 1), (2, 3)]), result);
    }

    #[test]
    fn union_overlapping() {
        let spet = VecSpet::from(create_spans(vec![(1, 2)]));
        let result: Vec<_> = spet.union(
            &VecSpet::from(create_spans(vec![(0, 4)]))).into();
        assert_eq!(create_spans(vec![(0, 4)]), result);
    }
}
