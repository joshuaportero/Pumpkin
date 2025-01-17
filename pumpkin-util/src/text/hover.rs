use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::TextComponent;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "action", content = "contents", rename_all = "snake_case")]
pub enum HoverEvent {
    /// Displays a tooltip with the given text.
    ShowText(Cow<'static, str>),
    /// Shows an item.
    ShowItem {
        /// Resource identifier of the item
        id: Cow<'static, str>,
        /// Number of the items in the stack
        count: Option<i32>,
        /// NBT information about the item (sNBT format)
        tag: Cow<'static, str>,
    },
    /// Shows an entity.
    ShowEntity {
        /// The entity's UUID
        id: uuid::Uuid,
        /// Resource identifier of the entity
        #[serde(rename = "type")]
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<Cow<'static, str>>,
        /// Optional custom name for the entity
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<Box<TextComponent>>,
    },
}
