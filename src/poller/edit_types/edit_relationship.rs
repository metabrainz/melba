use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditRelationship {
    #[serde(rename = "edit_version")]
    pub edit_version: Option<i64>,
    pub link: Option<Link>,
    pub new: Option<New>,
    pub old: Option<Old>,
    #[serde(rename = "relationship_id")]
    pub relationship_id: Option<i64>,
    pub type0: Option<String>,
    pub type1: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub attributes: Option<Vec<Value>>,
    #[serde(rename = "begin_date")]
    pub begin_date: Option<BeginDate>,
    #[serde(rename = "end_date")]
    pub end_date: Option<EndDate>,
    pub ended: Option<i64>,
    pub entity0: Option<Entity0>,
    pub entity1: Option<Entity1>,
    #[serde(rename = "link_type")]
    pub link_type: Option<LinkType>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginDate {
    pub day: Option<Value>,
    pub month: Option<Value>,
    pub year: Option<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndDate {
    pub day: Option<Value>,
    pub month: Option<Value>,
    pub year: Option<Value>,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct New {
    pub entity1: Option<Entity12>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity12 {
    pub gid: Option<String>,
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Old {
    pub entity1: Option<Entity13>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity13 {
    pub gid: Option<String>,
    pub id: Option<i64>,
    pub name: Option<String>,
}
