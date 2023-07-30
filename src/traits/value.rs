pub trait Values<T> {
    fn values(&self) -> T;
    fn alias(&self) -> String;
    fn public_key(&self) -> String;

    fn classic_address(&self) -> Option<String>;
}