// DeletePackageElementData represents a DeletePackageElementData model.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeletePackageElementData {
    #[serde(rename="module_version_id")]
    pub module_version_id: String,
    #[serde(rename="type_name")]
    pub type_name: String,
    #[serde(rename="element_id")]
    pub element_id: String,
    #[serde(rename="additionalProperties", skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<std::collections::HashMap<String, serde_json::Value>>,
}
