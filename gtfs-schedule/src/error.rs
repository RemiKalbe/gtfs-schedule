use std::fmt::Debug;

use miette::Diagnostic;
use thiserror::Error;

use crate::schemas::Schema;

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("Parse error: {0}")]
    #[diagnostic(transparent)]
    ParseError(#[from] ParseError),
    #[error("Schema validation error: {0}")]
    #[diagnostic(transparent)]
    SchemaValidationError(#[from] SchemaValidationError),
    #[error("Dataset validation error: {0}")]
    #[diagnostic(transparent)]
    DatasetValidationError(#[from] DatasetValidationError),
}

#[derive(Error, Debug, Diagnostic)]
pub struct ErrorContext(pub String);

impl<'s> std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug, Diagnostic)]
pub struct ParseError {
    #[source]
    #[diagnostic_source]
    pub kind: ParseErrorKind,
    #[related]
    pub context: Vec<ErrorContext>,
}

impl ParseError {
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context.push(context);
        self
    }
}

impl From<ParseErrorKind> for ParseError {
    fn from(kind: ParseErrorKind) -> Self {
        Self {
            kind,
            context: vec![],
        }
    }
}

impl<'s> std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Error, Debug, Diagnostic)]
pub enum ParseErrorKind {
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Chrono error: {0}")]
    Chrono(#[from] chrono::ParseError),
    #[error("ParseInt error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug, Diagnostic)]
pub enum SchemaValidationErrorKind {
    #[error("Missing value for field: {field_name}; reason: {reason:?}")]
    MissingValue {
        field_name: String,
        reason: Option<String>,
    },
    #[error("Forbidden value for field: {field_name}; reason: {reason:?}")]
    ForbiddenValue {
        field_name: String,
        reason: Option<String>,
    },
    #[error("Invalid value for field: {field_name}; reason: {reason:?}")]
    InvalidValue {
        field_name: String,
        reason: Option<String>,
    },
}

#[derive(Error, Debug, Diagnostic)]
pub enum DatasetValidationErrorKind {
    #[error("Primary key is not unique; {value} is duplicated in {field_name}")]
    PrimaryKeyNotUnique { field_name: String, value: String },
    #[error(
        "Foreign key in {field_name} with value {value} does not exist in {reference_file_name}"
    )]
    ForeignKeyNotFound {
        field_name: String,
        value: String,
        reference_file_name: String,
    },
    #[error("Inconsistent field {field_name} with value: {value}; reason: {reason:?}")]
    InconsistentValue {
        field_name: String,
        value: String,
        reason: Option<String>,
    },
    #[error("Invalid combination of fields: {fields:?}; reason: {reason:?}")]
    InvalidCombination {
        fields: Vec<String>,
        reason: Option<String>,
    },
    #[error("Missing value for field: {field_name}; reason: {reason:?}")]
    MissingValue {
        field_name: String,
        reason: Option<String>,
    },
    #[error("Overlapping intervals found: {details}")]
    OverlappingIntervals { details: String },
}

#[derive(Error, Debug, Diagnostic)]
pub struct DatasetValidationError {
    #[source]
    #[diagnostic_source]
    pub kind: DatasetValidationErrorKind,
    pub schema_instances: Vec<Schema>,
}

#[derive(Error, Debug, Diagnostic)]
pub struct SchemaValidationError {
    #[source]
    #[diagnostic_source]
    pub kind: SchemaValidationErrorKind,
    pub schema_instance: Schema,
}

impl<'s> std::fmt::Display for DatasetValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}; row: {:?}", self.kind, self.schema_instances)
    }
}

impl std::fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}; row: {:?}", self.kind, self.schema_instance)
    }
}

impl SchemaValidationError {
    pub fn new(kind: SchemaValidationErrorKind, schema_instance: Schema) -> Self {
        Self {
            kind,
            schema_instance,
        }
    }
    pub fn new_missing_value(
        field_name: String,
        reason: Option<String>,
        schema_instance: Schema,
    ) -> Self {
        Self::new(
            SchemaValidationErrorKind::MissingValue { field_name, reason },
            schema_instance,
        )
    }
    pub fn new_forbidden_value(
        field_name: String,
        reason: Option<String>,
        schema_instance: Schema,
    ) -> Self {
        Self::new(
            SchemaValidationErrorKind::ForbiddenValue { field_name, reason },
            schema_instance,
        )
    }
    pub fn new_invalid_value(
        field_name: String,
        reason: Option<String>,
        schema_instance: Schema,
    ) -> Self {
        Self::new(
            SchemaValidationErrorKind::InvalidValue { field_name, reason },
            schema_instance,
        )
    }
}

impl DatasetValidationError {
    pub fn new(kind: DatasetValidationErrorKind, schema_instances: Vec<Schema>) -> Self {
        Self {
            kind,
            schema_instances,
        }
    }
    pub fn new_primary_key_not_unique(
        field_name: String,
        value: String,
        schema_instances: Vec<Schema>,
    ) -> Self {
        Self::new(
            DatasetValidationErrorKind::PrimaryKeyNotUnique { field_name, value },
            schema_instances,
        )
    }
    pub fn new_foreign_key_not_found(
        field_name: String,
        value: String,
        reference_file_name: String,
        schema_instances: Vec<Schema>,
    ) -> Self {
        Self::new(
            DatasetValidationErrorKind::ForeignKeyNotFound {
                field_name,
                value,
                reference_file_name,
            },
            schema_instances,
        )
    }
    pub fn new_inconsistent_value(
        field_name: String,
        value: String,
        reason: Option<String>,
        schema_instances: Vec<Schema>,
    ) -> Self {
        Self::new(
            DatasetValidationErrorKind::InconsistentValue {
                field_name,
                value,
                reason,
            },
            schema_instances,
        )
    }
    pub fn new_invalid_combination(
        fields: Vec<String>,
        reason: Option<String>,
        schema_instances: Vec<Schema>,
    ) -> Self {
        Self::new(
            DatasetValidationErrorKind::InvalidCombination { fields, reason },
            schema_instances,
        )
    }
    pub fn new_missing_value(
        field_name: String,
        reason: Option<String>,
        schema_instances: Vec<Schema>,
    ) -> Self {
        Self::new(
            DatasetValidationErrorKind::MissingValue { field_name, reason },
            schema_instances,
        )
    }
    pub fn new_overlapping_intervals(details: String, schema_instances: Vec<Schema>) -> Self {
        Self::new(
            DatasetValidationErrorKind::OverlappingIntervals { details },
            schema_instances,
        )
    }
}

pub type Result<T> = std::result::Result<T, Error>;
