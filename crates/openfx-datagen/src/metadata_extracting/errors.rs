use std::collections::HashSet;

#[derive(Debug, snafu::Snafu, Default)]
pub struct Error {
    pub propdef_not_on_item_count: usize,
    /// The keys are the cnames of the items the `@propdef`s are associated
    /// with.
    pub propdef_errors: Vec<(String, PropdefError)>,

    pub propset_entries_on_items: HashSet<String>,
    pub propset_without_name_count: usize,

    pub propsetdef_entries_on_items: HashSet<String>,
    pub propsetdef_without_name_count: usize,

    pub actiondef_not_on_item_count: usize,
    pub actiondef_entries_with_names: HashSet<String>,
}

impl Error {
    pub fn is_empty(&self) -> bool {
        self.propdef_not_on_item_count == 0
            && self.propdef_errors.is_empty()
            && self.propset_entries_on_items.is_empty()
            && self.propset_without_name_count == 0
            && self.propsetdef_entries_on_items.is_empty()
            && self.propsetdef_without_name_count == 0
            && self.actiondef_not_on_item_count == 0
            && self.actiondef_entries_with_names.is_empty()
    }
}

#[derive(Debug, snafu::Snafu)]
pub enum PropdefError {
    /// e.g., `@propdef foo`, where there should not be a name after `@propdef`.
    PropdefWithName {
        name: String,
    },
    /// `type: ` and `dimension: ` fields are required, but at least one of them
    /// is missing.
    PropdefIncomplete {
        missing_type: bool,
        missing_dimension: bool,
    },
    PropdefUnexpectedFieldValue {
        field_name: String,
        value: String,
    },
    PropdefDuplicateField {
        field_name: String,
    },
    PropdefEnumWithoutValuesField,
    PropdefNonEnumWithValuesField,
    PropdefUnsupportedValueLineInValues {
        line_content: String,
    },
}
