use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveRelationship {
    #[serde(rename = "edit_version")]
    pub edit_version: Option<i64>,
    pub relationship: Option<Relationship>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    pub entity0: Option<Entity0>,
    #[serde(rename = "entity0_credit")]
    pub entity0_credit: Option<String>,
    pub entity1: Option<Entity1>,
    pub id: Option<i64>,
    pub link: Option<Link>,
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
pub struct Link {
    pub ended: Option<i64>,
    #[serde(rename = "type")]
    pub type_field: Option<Type>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    #[serde(rename = "entity0_type")]
    pub entity0_type: Option<String>,
    #[serde(rename = "entity1_type")]
    pub entity1_type: Option<String>,
    pub id: Option<i64>,
    #[serde(rename = "long_link_phrase")]
    pub long_link_phrase: Option<String>,
}
