use async_trait::async_trait;

#[async_trait]
pub trait StorageCollection{
    type Error;

    async fn init() -> Result<Self, Self::Error> where Self: Sized;
}