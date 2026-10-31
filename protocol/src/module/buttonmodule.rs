use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ButtonEvent {
    /// Misspelling kept on the wire (C-01); `Click` is accepted on input.
    #[serde(alias = "Click")]
    Ckick {},
}
