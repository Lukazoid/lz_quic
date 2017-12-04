use DataStream;

pub trait Perspective {
    fn open_stream(&self) -> DataStream<Self>
    where
        Self: Sized;
}
