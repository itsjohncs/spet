use std::fmt::{Debug};
use crate::span::Span;

/**
 * A reference to the start or end of a span.
 */
#[derive(PartialEq, Eq)]
pub enum Point<S: Span> {
    StartOf(S),
    EndOf(S),
}


impl<S: Span + Debug> Debug for Point<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Point::{StartOf, EndOf};
        match self {
            StartOf(v) => f.debug_tuple("StartOf").field(v).finish(),
            EndOf(v) => f.debug_tuple("EndOf").field(v).finish(),
        }
    }
}


/**
 * An end point.
 *
 * This is needed by PointIterator so that we can order the binary heap based
 * on the end points of the spans it contains.
 */
#[derive(Eq)]
struct OrderableEndPoint<S: Span>(S);


impl<S: Span> OrderableEndPoint<S> {
    pub fn value(&self) -> &S::Domain {
        self.0.end()
    }
}


impl<S: Span> Ord for OrderableEndPoint<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(&other.value())
    }
}

impl<S: Span> PartialEq for OrderableEndPoint<S> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}


impl<S: Span> PartialOrd for OrderableEndPoint<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


/**
 * Allows iteration of all points in a sorted iterator of spans.
 *
 * Let `(a, b)` represent a Span with start `a` and end `b`. Let a "point" be
 * either a start or end of a Span (so `a` and `b` are each a point).
 *
 * Given an iterator that yields `[(1, 3), (2, 2)]` this will yield the points
 * `[1, 2, 2, 3]`.
 *
 * This ends up being a building block that most/all set operations can be
 * built upon.
 */
pub struct PointIterator<I: Iterator> where I::Item: Span {
    // This iterator must give spans in ascending order of each span's start
    // (it does not matter whether the end is a secondary sort key).
    iterator: I,

    // When we ask self.iterator for another span, we may not want to yield its
    // start point immediately (there may be a smaller end in self.ends should
    // be yielded first). In this case we'll save the start value here, to be
    // yielded in a later Self::next() call (we save the whole span because a
    // Point needs a reference to the actual span).
    peeked_start: Option<I::Item>,

    // Conceptualize our iterator as (internally) ascending one-by-one through
    // all values in Span::Domain (I::Item::Domain). This min-heap will always
    // be storing the ends of all the spans that contain our current value.
    // This is an accurate way to conceptualize what we're doing, but our
    // implementation is just smart enough to skip past all the values that
    // don't matter.
    ends:
        // Binary heap...
        std::collections::BinaryHeap<
            // specifically a min-heap (rather than max-heap)...
            std::cmp::Reverse<
                // of spans, sorted by their end points
                OrderableEndPoint<I::Item>>>,
}


impl<I: Iterator> Iterator for PointIterator<I> where I::Item: Span {
    type Item = Point<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::cmp::Reverse;
        use Point::{StartOf, EndOf};

        if let Some(peeked_start) = &self.peeked_start {
            // If we've got a start point waiting for us we don't want to get
            // another item from self.iterator yet (because both points of the
            // next item will be greater than peeked_start.start() thanks to our
            // ascending iteration).
            Some(match self.ends.peek() {
                Some(Reverse(peeked_end))
                        if peeked_end.value() <= peeked_start.start() => {
                    let Reverse(popped_end) = self.ends.pop().unwrap();
                    EndOf(popped_end.0.clone())
                },
                _ => {
                    let result = StartOf(peeked_start.clone());
                    self.peeked_start = None;
                    result
                }
            })
        } else if let Some(span) = self.iterator.next() {
            // There's no peeked start, so we gotta process the next item from
            // the iterator. This push is where the log(N) part of our
            // complexity comes from. Everything else in this function is O(1).
            let to_yield = Some(match self.ends.peek() {
                Some(Reverse(end)) if end.value() <= span.start() => {
                    self.peeked_start = Some(span.clone());
                    let result = EndOf(end.0.clone());
                    self.ends.pop();
                    result
                },
                _ => StartOf(span.clone()),
            });

            self.ends.push(Reverse(OrderableEndPoint(span)));

            to_yield
        } else if let Some(Reverse(OrderableEndPoint(span))) = self.ends.pop() {
            // There's no peeked span, the iterator is depleted, so all that's
            // left is what's in our heap.
            Some(EndOf(span))
        } else {
            // We have no peeked span, the iterator is depleted, there's no
            // more points in the heap... we're done.
            None
        }
    }
}


pub fn enumerate_points<I: IntoIterator>(iterable: I) -> PointIterator<I::IntoIter>
        where I::Item: Span + Clone {
    use std::collections::BinaryHeap;
    PointIterator {
        iterator: iterable.into_iter(),
        peeked_start: None,
        ends: BinaryHeap::new(),
    }
}
    
#[cfg(test)]
mod tests {
    use crate::points::{enumerate_points, Point::{StartOf, EndOf}};
    use crate::span::{SimpleSpan, CreatableSpan};

    #[test]
    fn consuming_iter() {
        let spans: Vec<SimpleSpan<usize>> = vec![
            SimpleSpan::new(1, 2),
            SimpleSpan::new(1, 3),
        ];

        // This makes implicit copies, which is important because we're going
        // to consume `spans` in the enumeration.
        let expected = vec![
            StartOf(spans[0]),
            StartOf(spans[1]),
            EndOf(spans[0]),
            EndOf(spans[1]),
        ];

        assert_eq!(enumerate_points(spans).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn visiting_iter() {
        let spans: Vec<SimpleSpan<usize>> = vec![
            SimpleSpan::new(1, 2),
            SimpleSpan::new(1, 3),
        ];

        assert_eq!(enumerate_points(&spans).collect::<Vec<_>>(), vec![
            StartOf(&spans[0]),
            StartOf(&spans[1]),
            EndOf(&spans[0]),
            EndOf(&spans[1]),
        ]);
    }

    #[test]
    fn subset() {
        let spans: Vec<SimpleSpan<usize>> = vec![
            SimpleSpan::new(1, 4),
            SimpleSpan::new(2, 3),
        ];

        assert_eq!(enumerate_points(&spans).collect::<Vec<_>>(), vec![
            StartOf(&spans[0]),
            StartOf(&spans[1]),
            EndOf(&spans[1]),
            EndOf(&spans[0]),
        ]);
    }

    #[test]
    fn nesting_subsets() {
        // This is the worst case because the binary heap grows to its maximum
        // size of N.
        let spans: Vec<SimpleSpan<usize>> = vec![
            SimpleSpan::new(1, 10),
            SimpleSpan::new(2, 9),
            SimpleSpan::new(3, 8),
            SimpleSpan::new(4, 7),
            SimpleSpan::new(5, 6),
        ];

        assert_eq!(enumerate_points(&spans).collect::<Vec<_>>(), vec![
            StartOf(&spans[0]),
            StartOf(&spans[1]),
            StartOf(&spans[2]),
            StartOf(&spans[3]),
            StartOf(&spans[4]),
            EndOf(&spans[4]),
            EndOf(&spans[3]),
            EndOf(&spans[2]),
            EndOf(&spans[1]),
            EndOf(&spans[0]),
        ]);
    }

    #[test]
    fn non_overlapping() {
        let spans: Vec<SimpleSpan<usize>> = vec![
            SimpleSpan::new(1, 2),
            SimpleSpan::new(4, 5),
        ];

        assert_eq!(enumerate_points(&spans).collect::<Vec<_>>(), vec![
            StartOf(&spans[0]),
            EndOf(&spans[0]),
            StartOf(&spans[1]),
            EndOf(&spans[1]),
        ]);
    }

    #[test]
    fn empty() {
        let spans: Vec<SimpleSpan<usize>> = vec![];

        assert_eq!(enumerate_points(&spans).collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn single() {
        let spans: Vec<SimpleSpan<usize>> = vec![
            SimpleSpan::new(1, 2),
        ];

        assert_eq!(enumerate_points(&spans).collect::<Vec<_>>(), vec![
            StartOf(&spans[0]),
            EndOf(&spans[0]),
        ]);
    }
}
