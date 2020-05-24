use crate::data::base::coerce_compare;
use crate::data::base::shape::{Column, InlineShape};
use crate::data::primitive::style_primitive;
use chrono::DateTime;
use nu_errors::ShellError;
use nu_protocol::hir::Operator;
use nu_protocol::ShellTypeName;
use nu_protocol::{Primitive, Type, UntaggedValue};
use nu_source::{DebugDocBuilder, PrettyDebug, Tagged};

pub fn date_from_str(s: Tagged<&str>) -> Result<UntaggedValue, ShellError> {
    let date = DateTime::parse_from_rfc3339(s.item).map_err(|err| {
        ShellError::labeled_error(
            &format!("Date parse error: {}", err),
            "original value",
            s.tag,
        )
    })?;

    let date = date.with_timezone(&chrono::offset::Utc);

    Ok(UntaggedValue::Primitive(Primitive::Date(date)))
}

pub fn compare_values(
    operator: Operator,
    left: &UntaggedValue,
    right: &UntaggedValue,
) -> Result<bool, (&'static str, &'static str)> {
    let coerced = coerce_compare(left, right)?;
    let ordering = coerced.compare();

    use std::cmp::Ordering;

    let result = match (operator, ordering) {
        (Operator::Equal, Ordering::Equal) => true,
        (Operator::NotEqual, Ordering::Less) | (Operator::NotEqual, Ordering::Greater) => true,
        (Operator::LessThan, Ordering::Less) => true,
        (Operator::GreaterThan, Ordering::Greater) => true,
        (Operator::GreaterThanOrEqual, Ordering::Greater)
        | (Operator::GreaterThanOrEqual, Ordering::Equal) => true,
        (Operator::LessThanOrEqual, Ordering::Less)
        | (Operator::LessThanOrEqual, Ordering::Equal) => true,
        _ => false,
    };

    Ok(result)
}

pub fn merge_values(
    left: &UntaggedValue,
    right: &UntaggedValue,
) -> Result<UntaggedValue, (&'static str, &'static str)> {
    match (left, right) {
        (UntaggedValue::Row(columns), UntaggedValue::Row(columns_b)) => {
            Ok(UntaggedValue::Row(columns.merge_from(columns_b)))
        }
        (left, right) => Err((left.type_name(), right.type_name())),
    }
}

pub fn format_type<'a>(value: impl Into<&'a UntaggedValue>, width: usize) -> String {
    Type::from_value(value.into()).colored_string(width)
}

pub fn format_leaf<'a>(value: impl Into<&'a UntaggedValue>) -> DebugDocBuilder {
    InlineShape::from_value(value.into()).format().pretty()
}

pub fn style_leaf<'a>(value: impl Into<&'a UntaggedValue>) -> &'static str {
    match value.into() {
        UntaggedValue::Primitive(p) => style_primitive(p),
        _ => "",
    }
}

pub fn format_for_column<'a>(
    value: impl Into<&'a UntaggedValue>,
    column: impl Into<Column>,
) -> DebugDocBuilder {
    InlineShape::from_value(value.into())
        .format_for_column(column)
        .pretty()
}

#[cfg(test)]
mod tests {
    use super::UntaggedValue as v;
    use indexmap::indexmap;

    use super::merge_values;

    #[test]
    fn merges_tables() {
        let table_author_row = v::row(indexmap! {
            "name".into() => v::string("Andrés").into_untagged_value(),
            "country".into() => v::string("EC").into_untagged_value(),
            "date".into() => v::string("April 29-2020").into_untagged_value()
        });

        let other_table_author_row = v::row(indexmap! {
            "name".into() => v::string("YK").into_untagged_value(),
            "country".into() => v::string("US").into_untagged_value(),
            "date".into() => v::string("October 10-2019").into_untagged_value()
        });

        assert_eq!(
            other_table_author_row,
            merge_values(&table_author_row, &other_table_author_row).unwrap()
        );
    }
}
