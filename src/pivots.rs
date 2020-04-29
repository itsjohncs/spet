use crate::span::Span;

/**
 * Represents the start or end of a span.
 */
pub enum Point<'a, S: Span> {
    StartOf(&'a S),
    EndOf(S),
}


impl<'a, S: Span> Point<'a, S> {
    pub fn value(&'a self) -> &'a S::Domain {
        use Point::{StartOf, EndOf};
        match self {
            StartOf(span) => span.start(),
            EndOf(span) => span.end(),
        }
    }
}


/**
 * Similar to Point, but always represents an end point.
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
pub struct PointIterator<'a, I: Iterator> where I::Item: Span {
    // This iterator must give spans in ascending order of each span's start
    // (it does not matter whether the end is a secondary sort key).
    iterator: I,

    // When we ask self.iterator for another span, we may not want to yield its
    // start point immediately (there may be a smaller end in self.ends should
    // be yielded first). In this case we'll save the start value here, to be
    // yielded in a later Self::next() call (we save the whole span because a
    // Point needs a reference to the actual span).
    peeked_start: Option<&'a I::Item>,

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


impl<'a, I: Iterator> Iterator for PointIterator<'a, I> where I::Item: Span {
    type Item = Point<'a, I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::cmp::Reverse;
        use Point::{StartOf, EndOf};

        if let Some(peeked_span) = self.peeked_start {
            // If we've got a start point waiting for us we don't want to get
            // another item from self.iterator yet (because both points of the
            // next item will be greater than peeked_span.start() thanks to our
            // ascending iteration).
            Some(match self.ends.peek() {
                Some(Reverse(peeked_end))
                        if peeked_end.value() <= peeked_span.start() => {
                    let Reverse(popped_end) = self.ends.pop().unwrap();
                    EndOf(popped_end.0)
                },
                _ => {
                    self.peeked_start = None;
                    StartOf(peeked_span)
                }
            })
        } else if let Some(span) = self.iterator.next() {
            // There's no peeked span, so we gotta process the next item from
            // the iterator. This push is where the log(N) part of our
            // complexity comes from. Everything else in this function is O(1).
            let to_yield = Some(match self.ends.peek() {
                Some(Reverse(end)) if end.value() <= span.start() => {
                    self.peeked_start = Some(&span);
                    EndOf(end.0)
                },
                _ => StartOf(&span),
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


pub fn enumerate_pivots<'a, I: IntoIterator>(iterable: I) -> PointIterator<'a, I::IntoIter>
        where I::Item: Span {
    use std::collections::BinaryHeap;
    PointIterator {
        iterator: iterable.into_iter(),
        peeked_start: None,
        ends: BinaryHeap::new(),
    }
}
