use async_trait::async_trait;
use luce_shared::LuceError;

#[async_trait]
pub trait UseCase<Input, Output> {
    async fn execute(&self, input: Input) -> Result<Output, LuceError>;
}