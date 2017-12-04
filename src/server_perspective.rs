use {Perspective, DataStream};

#[derive(Debug)]
pub struct ServerPerspective {}

impl Perspective for ServerPerspective {
    fn open_stream(&self) -> DataStream<Self>{
        unimplemented!()
    }
}
