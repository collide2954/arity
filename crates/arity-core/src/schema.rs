use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};

use crate::{Curie, Decimal, Digest, Doi, Orcid};

macro_rules! string_schema {
    ($type:ty, $name:literal, $description:literal) => {
        impl JsonSchema for $type {
            fn schema_name() -> Cow<'static, str> {
                $name.into()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({ "type": "string", "description": $description })
            }
        }
    };
}

string_schema!(
    Decimal,
    "Decimal",
    "A decimal number as text, exactly as the source writes it, for example \"3.20\" or \"1.5e-3\"."
);
string_schema!(
    Curie,
    "Curie",
    "A compact identifier in a public registry, for example \"CHEBI:28748\"."
);
string_schema!(
    Orcid,
    "Orcid",
    "An ORCID identifier, for example \"0000-0002-1825-0097\"."
);
string_schema!(
    Digest,
    "Digest",
    "A SHA-256 digest as 64 hexadecimal digits."
);
string_schema!(
    Doi,
    "Doi",
    "A DOI, for example \"10.1371/journal.pone.0291981\"."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_types_have_string_schemas() {
        let schema = serde_json::to_value(schemars::schema_for!(Decimal)).unwrap();
        assert_eq!(schema["type"], "string");
        assert!(schema["description"].as_str().unwrap().contains("3.20"));
    }
}
