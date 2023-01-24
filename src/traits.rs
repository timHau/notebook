pub trait Saveable {
    fn save(&self) -> Result<(), std::io::Error>;
}
