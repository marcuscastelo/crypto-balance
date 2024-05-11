#[async_trait::async_trait]
pub trait Routine<T = ()> {
    async fn run(&self) -> T;
}