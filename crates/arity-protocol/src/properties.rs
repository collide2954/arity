use std::collections::HashMap;
use std::hash::BuildHasher;

use arity_core::Curie;
use arity_core::vocabulary::ValueType;
use serde::{Deserialize, Serialize};

/// The stored definition of a property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyDefinition {
    pub key: String,
    pub label: String,
    pub value_type: ValueType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<Curie>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// The properties that the database already defines.
pub trait Properties {
    fn property(&self, key: &str) -> Option<PropertyDefinition>;
}

impl<S: BuildHasher> Properties for HashMap<String, PropertyDefinition, S> {
    fn property(&self, key: &str) -> Option<PropertyDefinition> {
        self.get(key).cloned()
    }
}

impl Properties for [PropertyDefinition] {
    fn property(&self, key: &str) -> Option<PropertyDefinition> {
        self.iter()
            .find(|definition| definition.key == key)
            .cloned()
    }
}
