use crate::span::{Span, CreatableSpan};
use crate::vecspet::VecSpet;
use crate::points::{enumerate_points, Point};
use crate::mergeiter::sorted_chain;


// spets should be an iterable over Spets. Spets are containers for Spans, and
// this function will accept other containers for spans as well (so spets could
// be a Vec<Vec<SomeSpan>>), however in this case it's up to you to make sure
// that each individual container of spans is sorted.
pub fn n_overlapping<T: CreatableSpan, I: IntoIterator>(
        n: usize,
        spets: I) -> VecSpet<T>
        where I::Item: IntoIterator,
              <I::Item as IntoIterator>::Item: Span,
              <<I::Item as IntoIterator>::Item as Span>::Domain: Into<T::Domain> {
    assert!(n > 0);

    // Could (maybe) be a bit more efficient by putting the below code in an
    // iterator and passing that iterator to from_sorted_iter, rather than
    // growing this vector and then immediately throwing it away. Could also
    // add a way to construct a VecSpet from an already-sorted Vec by just
    // taking the Vec into itself.
    let mut result_spans = Vec::new();

    let mut num_overlapping = 0;
    let mut pending_start: Option<T::Domain> = None;
    for point in enumerate_points(sorted_chain(spets)) {
        use Point::{StartOf, EndOf};
        match point {
            StartOf(span) => {
                num_overlapping += 1;
                if num_overlapping == n {
                    pending_start = Some(span.start().clone().into());
                }
            },
            EndOf(span) => {
                if num_overlapping == n {
                    result_spans.push(
                        T::new(pending_start.unwrap(), span.end().clone().into()));
                    pending_start = None;
                }
                num_overlapping -= 1;
            },
        }
    }

    VecSpet::from_sorted_iter(result_spans)
}

#[cfg(test)]
mod test {
    use crate::overlapping::n_overlapping;
    use crate::span::{SimpleSpan, CreatableSpan};
    use crate::vecspet::VecSpet;

    type SSpan = SimpleSpan<usize>;
    type Spet = VecSpet<SSpan>;

    #[test]
    fn some_overlap() {
        let result = n_overlapping(2, vec![
            Spet::from_sorted_iter(vec![
                SSpan::new(1, 3),
                SSpan::new (5, 6),
            ]),
            Spet::from_sorted_iter(vec![
                SSpan::new(2, 5)
            ]),
        ]);

        assert_eq!(result, Spet::from_sorted_iter(vec![
            SSpan::new(2, 3),
            SSpan::new(5, 5),
        ]));
    }

    #[test]
    fn no_overlap() {
        let result = n_overlapping(2, vec![
            Spet::from_sorted_iter(vec![
                SSpan::new(1, 3),
            ]),
            Spet::from_sorted_iter(vec![
                SSpan::new(5, 7),
            ]),
        ]);

        let src: Vec<SSpan> = Vec::new();
        assert_eq!(result, Spet::from_sorted_iter(src));
    }
}
