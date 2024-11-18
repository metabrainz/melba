use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditUrl {
    pub affects: Option<i64>,
    pub entity: Option<Entity>,
    #[serde(rename = "is_merge")]
    pub is_merge: Option<i64>,
    pub new: Option<New>,
    pub old: Option<Old>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub gid: Option<String>,
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct New {
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Old {
    pub url: Option<String>,
}
