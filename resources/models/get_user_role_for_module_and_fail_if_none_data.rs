// GetUserRoleForModuleAndFailIfNoneData represents a GetUserRoleForModuleAndFailIfNoneData model.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetUserRoleForModuleAndFailIfNoneData {
    #[serde(rename="moduleId")]
    pub module_id: String,
    #[serde(rename="additionalProperties", skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<std::collections::HashMap<String, serde_json::Value>>,
}
