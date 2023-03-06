pub trait Encoder {
    type Item;
    type Error: std::error::Error + 'static;

    fn encode(&self, item: &Self::Item) -> Result<Vec<u8>, Self::Error>;
    fn max_size(&self) -> usize;
}

pub trait Decoder {
    type Item;
    type Error: std::error::Error + 'static;

    fn decode(&self, src: &[u8]) -> Result<(Self::Item, usize), Self::Error>;
}
