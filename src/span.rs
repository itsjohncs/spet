pub trait Span: Ord + Clone {
    type Domain: Ord + Clone;

    fn start(&self) -> &Self::Domain;
    fn end(&self) -> &Self::Domain;
    fn contains(&self, query: &Self::Domain) -> bool {
        self.start() <= query && query < self.end()
    }
}


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SimpleSpan<T: Ord + Copy> {
    start: T,
    end: T
}


impl<T: Ord + Copy> Span for SimpleSpan<T> {
    type Domain = T;

    fn start(&self) -> &Self::Domain {
        &self.start
    }

    fn end(&self) -> &Self::Domain {
        &self.end
    }
}


impl<'a, T: Span> Span for &'a T {
    type Domain = T::Domain;

    fn start(&self) -> &Self::Domain {
        (*self).start()
    }

    fn end(&self) -> &Self::Domain {
        (*self).end()
    }
}


// If I put new() on the Span trait, I wouldn't be able to have all
// references to a Span also implement Span, since I couldn't have a function
// that returns a reference to an object it allocated.
pub trait CreatableSpan: Span {
    fn new(start: Self::Domain, end: Self::Domain) -> Self;
}


impl<T: Ord + Copy> CreatableSpan for SimpleSpan<T> {
    fn new(start: T, end: T) -> Self {
        SimpleSpan { start, end }
    }
}
