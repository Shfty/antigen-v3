/// A type that can map from some type `T` to some other type `Mapped`
pub trait Mapper<T> {
    type Mapped;

    fn map(input: T) -> Self::Mapped;
}

/// A type that can apply a `Mapper` across some owned data
pub trait Map<M> {
    type Item;
    type Iter: Iterator<Item = Option<Self::Item>>;

    fn map(&self) -> Self::Iter;
}

/// A `Mapper` wrapping the ToString trait
pub struct ToStringMapper;
impl<T> Mapper<T> for ToStringMapper
where
    T: ToString,
{
    type Mapped = String;

    fn map(input: T) -> Self::Mapped {
        input.to_string()
    }
}
