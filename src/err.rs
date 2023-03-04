pub(crate) type LazyResult<A> = Result<A, Box<dyn std::error::Error>>;
