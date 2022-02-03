use serde::{Deserialize, Serialize};
use crate::input::UserInput;

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct UserSettings {
    pub(crate) input: UserInput,
}