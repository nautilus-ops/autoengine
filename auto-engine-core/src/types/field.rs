use crate::types::node::I18nValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectConstraint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArrayConstraint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<Box<ValueConstraint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BooleanConstraint {
    pub equals: bool,
}

impl BooleanConstraint {
    pub fn from(value: bool) -> Self {
        Self { equals: value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StringConstraint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>, // uri / email / date 等
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equals: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NumberConstraint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>, // >=
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<f64>, // >
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>, // <=
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<f64>, // <
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equals: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValueConstraint {
    Number(NumberConstraint),
    String(StringConstraint),
    Boolean(BooleanConstraint),
    Array(ArrayConstraint),
    Object(ObjectConstraint),
}

impl Default for ValueConstraint {
    fn default() -> Self {
        Self::Number(NumberConstraint::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct FieldCondition {
    pub field: String,
    pub constraint: ValueConstraint,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Condition {
    All { conditions: Vec<Box<Condition>> },
    Any { conditions: Vec<Box<Condition>> },
    Not { condition: Box<Condition> },
    Field(FieldCondition),
}

impl Default for Condition {
    fn default() -> Self {
        Self::All { conditions: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    #[default]
    String,
    Text,   // for long content
    Number,
    Boolean,
    Array,
    Object,
    Image,
    File,
    Password,
}
#[derive(Clone, Default, Serialize, Debug, Deserialize)]
pub struct SchemaField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<FieldType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<I18nValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
}
