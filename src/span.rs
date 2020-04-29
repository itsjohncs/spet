pub trait Span: Ord {
    type Domain: Ord;

    fn create(start: Self::Domain, end: Self::Domain) -> Self;
    fn start(&self) -> &Self::Domain;
    fn end(&self) -> &Self::Domain;
    fn contains(&self, query: &Self::Domain) -> bool {
        self.start() <= query && query < self.end()
    }
}
