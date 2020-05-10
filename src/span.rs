pub trait Span: Ord {
    type Domain: Ord;

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


pub trait CreatableSpan: Span {
    fn create(start: Self::Domain, end: Self::Domain) -> Self;
}


impl<T: Ord + Copy> CreatableSpan for SimpleSpan<T> {
    fn create(start: T, end: T) -> Self {
        SimpleSpan { start, end }
    }
}
