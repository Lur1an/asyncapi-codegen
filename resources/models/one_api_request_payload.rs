// OneApiRequestPayload represents a union of types: GetUserRoleForModuleAndFailIfNone, DeletePackageElement
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "event")]
pub enum OneApiRequestPayload {
    #[serde(rename="GetUserRoleForModuleAndFailIfNone")]
    GetUserRoleForModuleAndFailIfNone(crate::GetUserRoleForModuleAndFailIfNone),
    #[serde(rename="DeletePackageElement")]
    DeletePackageElement(crate::DeletePackageElement),
}

