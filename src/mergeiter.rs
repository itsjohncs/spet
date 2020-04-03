use std::cmp::Reverse;

/**
 * Keeps track of the next item in the iterator.
 *
 * This is similar to std::iter::Peekable in that it's grabbing a value
 * from an iterator and then keeping the value handy alongside the
 * iterator.
 *
 * Except in our case, the value isn't optional, PeekedIter doesn't
 * actually implement Iterator itself, and PeekedIter implements Ord.
 */
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

/**
 * Chains multiple sorted iterators into a single sorted iterator.
 *
 * All the iterators must be sorted in ascending order, and the resulting
 * iterator will be sorted in ascending order.
 *
 * Given N total items split among M iterators:
 *
 *  - Construction is amortized O(M * log M) (amortized because the binary
 *    heap is grown from an empty Vec)
 *  - next() is O(log M)
 *  - Consuming entire iterator is O(N * log M)
 *
 * It might be worth using a "small vec" to back the BinaryHeap or make a
 * special case implementation because the case of just having 2 items is a
 * very common one and the cache locality could be impactful.
 */
pub struct SortedChain<T: Iterator> where T::Item: Ord {
    // A min-heap (thanks to Reverse) that'll tell us the next smallest value
    // that's waiting for us among our iterators.
    queue: std::collections::BinaryHeap<Reverse<PeekedIter<T>>>,
}

impl<T: Iterator> Iterator for SortedChain<T> where T::Item : Ord {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Reverse(mut peeked_iter)) = self.queue.pop() {
            // Only put the iterator back if there's actually something
            // there. As we drain the iterators, our heap will shrink.
            if let Some(value) = peeked_iter.iterator.next() {
                self.queue.push(Reverse(PeekedIter {
                    value,
                    iterator: peeked_iter.iterator,
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

/**
 * Creates a SortedChain from an iterable of iterables of T.
 *
 * For example, you could call
 * SortedChain::<i32>::from(vec![vec![1, 2], vec![3, 4]]).
 */
pub fn sorted_chain<
        T: Iterator,
        I: IntoIterator<IntoIter = T, Item = T::Item>,
        A: IntoIterator<Item = I>>(containers: A) -> SortedChain<T>
            where T::Item: Ord {
    // Could look at iterators.size_hint() and then use with_capacity...
    let mut result = SortedChain::<T> {
        queue: std::collections::BinaryHeap::new()
    };
    for container in containers.into_iter() {
        let mut iterator = container.into_iter();
        if let Some(value) = iterator.next() {
            result.queue.push(Reverse(PeekedIter { value, iterator }));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    mod sorted_chain {
        use crate::mergeiter::sorted_chain;

        #[test]
        fn no_iterables() {
            let merged = sorted_chain(Vec::<Vec<i32>>::new());
            assert_eq!(merged.collect::<Vec<_>>(),
                       vec![]);
        }

        #[test]
        fn empty_iterables() {
            let merged = sorted_chain(vec![
                vec![],
                vec![],
                vec![],
            ] as Vec<Vec<i32>>);
            assert_eq!(merged.collect::<Vec<_>>(),
                       vec![]);
        }

        #[test]
        fn several_small() {
            let merged = sorted_chain(vec![
                vec![1, 4],
                vec![1, 2, 3],
                vec![1, 3],
            ]);
            assert_eq!(merged.collect::<Vec<_>>(),
                       vec![1, 1, 1, 2, 3, 3, 4]);
        }

        #[test]
        fn borrowing() {
            let mut iterables = vec![
                vec![1, 4],
                vec![1, 2, 3],
                vec![1, 3],
            ];
            let merged = sorted_chain(&iterables);
            assert_eq!(merged.collect::<Vec<_>>(),
                       vec![&1, &1, &1, &2, &3, &3, &4]);

            // And we still get to do stuff with iterables
            iterables.push(vec![1, 5]);
        }
    }
}
