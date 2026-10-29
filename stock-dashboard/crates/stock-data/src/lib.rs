pub mod common;
pub mod data;

pub use common::{error::DataError, models::*};
pub use data::eastmoney::EastmoneyClient;
pub use data::polling::{PollingHandle, spawn_polling};
