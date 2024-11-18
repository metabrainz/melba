use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRelationship {
    #[serde(rename = "edit_version")]
    pub edit_version: Option<i64>,
    pub entity0: Option<Entity0>,
    pub entity1: Option<Entity1>,
    #[serde(rename = "entity_id")]
    pub entity_id: Option<i64>,
    #[serde(rename = "link_type")]
    pub link_type: Option<LinkType>,
    pub type0: Option<String>,
    pub type1: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity0 {
    pub gid: Option<String>,
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity1 {
    pub gid: Option<String>,
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkType {
    pub id: Option<i64>,
    #[serde(rename = "link_phrase")]
    pub link_phrase: Option<String>,
    #[serde(rename = "long_link_phrase")]
    pub long_link_phrase: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "reverse_link_phrase")]
    pub reverse_link_phrase: Option<String>,
}
