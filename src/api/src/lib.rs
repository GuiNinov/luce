pub mod handlers;
pub mod server;
pub mod services;

use luce_shared::error::LuceError;

pub type ApiResult<T> = Result<T, LuceError>;
