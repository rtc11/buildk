use std::collections::HashMap;

use serde_derive::Deserialize;

#[derive(Deserialize, Clone, PartialEq, Debug, Default)]
#[serde(transparent)]
pub struct Dependencies {
    pub deps: HashMap<String, String>
}
