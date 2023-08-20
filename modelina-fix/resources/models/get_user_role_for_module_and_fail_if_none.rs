// GetUserRoleForModuleAndFailIfNone represents a GetUserRoleForModuleAndFailIfNone model.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetUserRoleForModuleAndFailIfNone {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "kind")]
    pub kind: Box<crate::AnonymousSchema2>,
    #[serde(rename = "event")]
    pub event: Box<crate::AnonymousSchema4>,
    #[serde(rename = "data")]
    pub data: Box<crate::GetUserRoleForModuleAndFailIfNoneData>,
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<std::collections::HashMap<String, serde_json::Value>>,
}
