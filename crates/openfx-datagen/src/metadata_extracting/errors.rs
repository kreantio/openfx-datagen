#[derive(Debug, snafu::Snafu, Default)]
pub struct Error {
    pub propdef_not_on_item_count: usize,
    /// The keys are the cnames of the items the `@propdef`s are associated
    /// with.
    pub propdef_errors: Vec<(String, PropdefError)>,

    pub propset_without_name_count: usize,
    /// The keys are the names of the `@propset`s.
    pub propset_errors: Vec<(String, PropsetError)>,

    pub propsetdef_without_name_count: usize,
    /// The keys are the names of the `@propsetdef`s.
    pub propsetdef_errors: Vec<(String, PropsetdefError)>,

    pub actiondef_not_on_item_count: usize,
    /// The keys are the cnames of the items the `@actiondef`s are associated
    /// with.
    pub actiondef_errors: Vec<(String, ActiondefError)>,
}

impl Error {
    pub fn is_empty(&self) -> bool {
        self.propdef_not_on_item_count == 0
            && self.propdef_errors.is_empty()
            && self.propset_without_name_count == 0
            && self.propset_errors.is_empty()
            && self.propsetdef_errors.is_empty()
            && self.propsetdef_without_name_count == 0
            && self.actiondef_not_on_item_count == 0
            && self.actiondef_errors.is_empty()
    }
}

#[derive(Debug, snafu::Snafu)]
pub enum PropdefError {
    PropdefSerdeSaphyrParse {
        #[snafu(source)]
        error: serde_saphyr::Error,
    },
    /// e.g., `@propdef foo`, where there should not be a name after `@propdef`.
    PropdefWithName {
        name: String,
    },
    PropdefUnexpectedFieldValue {
        field_name: String,
        value: String,
    },
    PropdefDuplicateField {
        field_name: String,
    },
    PropdefNoType,
    PropdefEnumAmongOtherTypes,
    PropdefEnumWithoutValuesField,
    PropdefNonEnumWithValuesField,
}

#[derive(Debug, snafu::Snafu)]
pub enum PropsetError {
    PropsetSerdeSaphyrParse {
        #[snafu(source)]
        error: serde_saphyr::Error,
    },
    PropsetOnItem {
        item_cname: String,
    },
    PropsetUndefinedProp {
        stringname: String,
    },
    PropsetPropDuplicate {
        prop_cname: String,
    },
    PropsetPropsRefWithOptions {
        props_ref_cname: String,
    },
    PropsetPropValueError {
        prop_cname: String,
        #[snafu(source)]
        error: PropValueError,
    },
}

#[derive(Debug, snafu::Snafu)]
pub enum PropsetdefError {
    PropsetdefSerdeSaphyrParse {
        #[snafu(source)]
        error: serde_saphyr::Error,
    },
    PropsetdefOnItem {
        item_cname: String,
    },
    PropsetdefUndefinedProp {
        stringname: String,
    },
    PropsetdefPropDuplicate {
        prop_cname: String,
    },
    PropsetdefPropValueError {
        prop_cname: String,
        #[snafu(source)]
        error: PropValueError,
    },
}

#[derive(Debug, snafu::Snafu)]
pub enum PropValueError {
    PropValueUnexpectedOption {
        option_name: String,
        option_value: String,
    },
    PropValueUnexpectedOptionValue {
        option_name: String,
        option_value: String,
    },
    PropValueDuplicateOption {
        option_name: String,
    },
}

#[derive(Debug, snafu::Snafu)]
pub enum ActiondefError {
    ActiondefSerdeSaphyrParse {
        #[snafu(source)]
        error: serde_saphyr::Error,
    },
    ActiondefWithName {
        name: String,
    },
    ActiondefUndefinedInArg {
        stringname: String,
    },
    ActiondefInArgWithOptions {
        in_arg_cname: String,
    },
    ActiondefUndefinedOutArg {
        stringname: String,
    },
    ActiondefOutArgWithOptions {
        out_arg_cname: String,
    },
}
