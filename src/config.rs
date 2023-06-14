use std::collections::HashMap;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub inject: HashMap<String, HashMap<String, Vec<InjectAttrConfig>>>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectAttrConfig {
    /// jsx attribute name
    pub name: String,
    /// inject rule
    #[serde(default)]
    pub rule: InjectAttrRule,
    /// jsx attribute value
    pub value: serde_json::Value,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InjectAttrRule {
    Append,
    Prepend,
    // Replace,
}

impl Default for InjectAttrRule {
    fn default() -> Self {
        InjectAttrRule::Prepend
    }
}
