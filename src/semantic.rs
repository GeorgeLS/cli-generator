use crate::types::{AttributeType, Field, FieldType, Spec, SpecMetadata, Struct};
use logos::Span;
use std::cmp::{max, min};
use std::collections::HashMap;

fn get_line_with_span(source: &str, span: &Span) -> usize {
    source[..span.start].lines().count()
}

fn get_line_span(source: &str, line: usize) -> Span {
    let mut current_chars = 0;
    for (i, l) in source.lines().enumerate() {
        if i + 1 == line {
            return Span::from(current_chars..current_chars + l.len());
        }
        current_chars += l.len() + 1;
    }

    unreachable!();
}

fn get_context(source: &str, span: &Span) -> (usize, Span) {
    let line_start = get_line_with_span(source, span);

    let previous_line = max(1, line_start - 1);
    let previous_line_span = get_line_span(source, previous_line);

    let next_line = min(line_start + 1, source.lines().count());
    let next_line_span = get_line_span(source, next_line);

    (
        line_start,
        Span::from(previous_line_span.start..next_line_span.end),
    )
}

fn make_chic_error<'s>(
    label: &'s str,
    source: &'s str,
    span: &'s Span,
    error_msg: &'s str,
) -> chic::Error<'s> {
    let (line_start, context_span) = get_context(source, span);

    chic::Error::new(label).error(
        line_start,
        span.start - context_span.start,
        span.end - context_span.start,
        &source[context_span.start..context_span.end],
        error_msg,
    )
}

fn make_chic_error_with_info<'s>(
    label: &'s str,
    source: &'s str,
    error_span: &'s Span,
    error_msg: &'s str,
    info_span: &'s Span,
    info_msg: &'s str,
) -> chic::Report<'s> {
    let (error_line_start, error_context_span) = get_context(source, error_span);
    let (info_line_start, info_context_span) = get_context(source, info_span);

    chic::Report::new_error(label)
        .error(
            error_line_start,
            error_span.start - error_context_span.start,
            error_span.end - error_context_span.start,
            &source[error_context_span.start..error_context_span.end],
            error_msg,
        )
        .info(
            info_line_start,
            info_span.start - info_context_span.start,
            info_span.end - info_context_span.start,
            &source[info_context_span.start..info_context_span.end],
            info_msg,
        )
}

fn check_for_multiple_struct_definitions<'s>(
    structs: &'s [Struct],
    source: &'s str,
) -> Result<HashMap<&'s str, &'s Struct>, String> {
    let mut id_to_struct = HashMap::with_capacity(structs.len());

    for strukt in structs {
        if id_to_struct.contains_key(strukt.name.as_str()) {
            let original_struct: &Struct = id_to_struct[strukt.name.as_str()];
            let chic_error = make_chic_error_with_info(
                "Multiple type definition",
                source,
                &strukt.name_span,
                "Redefinition of type",
                &original_struct.name_span,
                "Has already been defined here",
            );

            return Err(chic_error.to_string());
        }
        id_to_struct.insert(strukt.name.as_str(), strukt);
    }

    Ok(id_to_struct)
}

fn check_for_multiple_field_definitions(fields: &[Field], source: &str) -> Result<(), String> {
    let mut name_to_field = HashMap::with_capacity(fields.len());

    for field in fields {
        if name_to_field.contains_key(field.name.as_str()) {
            let original_field: &Field = name_to_field[field.name.as_str()];
            let chic_error = make_chic_error_with_info(
                "Multiple field definition",
                source,
                &field.name_span,
                "Redefinition of field",
                &original_field.name_span,
                "Has already been defined here",
            );

            return Err(chic_error.to_string());
        }

        name_to_field.insert(field.name.as_str(), field);
    }

    Ok(())
}

fn check_for_undefined_types(
    metadata: &SpecMetadata,
    fields: &[Field],
    source: &str,
) -> Result<(), String> {
    for field in fields {
        match &field.ty {
            FieldType::Vec(inner) => match inner.as_ref() {
                FieldType::Vec(_) => unreachable!(),
                FieldType::Struct(name) => {
                    if !metadata.identifier_to_struct.contains_key(name.as_str()) {
                        return Err(make_chic_error(
                            "Semantic error",
                            source,
                            &field.type_span,
                            "Undefined type",
                        )
                        .to_string());
                    }
                }
                _ => {}
            },
            FieldType::Struct(name) => {
                if !metadata.identifier_to_struct.contains_key(name.as_str()) {
                    return Err(make_chic_error(
                        "Semantic error",
                        source,
                        &field.type_span,
                        "Undefined type",
                    )
                    .to_string());
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn check_struct_attributes(strukt: &Struct, source: &str) -> Result<(), String> {
    let mut main_span = Span::default();
    let mut subcommand_span = Span::default();
    let mut has_main = false;
    let mut has_subcommand = false;

    for attribute in &strukt.attributes {
        match attribute.ty {
            AttributeType::Short
            | AttributeType::Long
            | AttributeType::Alias
            | AttributeType::Flatten => {
                let help_msg = format!(
                    "Allowed attributes: {}",
                    AttributeType::allowed_struct_attribute_types()
                        .iter()
                        .map(|v| v.to_literal())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                let chic_error = make_chic_error(
                    "Semantic error",
                    source,
                    &attribute.span,
                    "Invalid attribute",
                )
                .help(help_msg.as_str());

                return Err(chic_error.to_string());
            }
            AttributeType::Main => {
                has_main = true;
                main_span = attribute.span.clone();
            }
            AttributeType::SubCommand => {
                has_subcommand = true;
                subcommand_span = attribute.span.clone();
            }
        }
    }

    if has_main && has_subcommand {
        let error_span = Span::from(
            min(main_span.start, subcommand_span.start)..max(main_span.end, subcommand_span.end),
        );

        let chic_error = make_chic_error(
            "Semantic error",
            source,
            &error_span,
            "Invalid attribute combination",
        )
        .help("Only main or subcommand attributes are allowed");

        return Err(chic_error.to_string());
    }

    Ok(())
}

fn check_field_attributes(fields: &[Field], source: &str) -> Result<(), String> {
    let mut shorts = HashMap::new();
    let mut longs = HashMap::new();
    let mut aliases = HashMap::new();

    for field in fields {
        for attribute in &field.attributes {
            match attribute.ty {
                AttributeType::Short => {
                    let value = attribute.value.as_ref().unwrap().as_str();

                    if shorts.contains_key(value) {
                        let original_field: &Field = shorts[value];

                        let chic_error = make_chic_error_with_info(
                            "Invalid field attribute usage",
                            source,
                            &attribute.span,
                            "There's already a field with the same starting character",
                            &original_field.name_span,
                            "Field with same starting letter",
                        );

                        return Err(chic_error.to_string());
                    }

                    shorts.insert(value, field);
                }
                AttributeType::Long => {
                    let value = attribute.value.as_ref().unwrap().as_str();

                    if longs.contains_key(value) && aliases.contains_key(value) {
                        let original_field: &Field = longs[value];
                        let chic_error = make_chic_error_with_info(
                            "Invalid field attribute usage",
                            source,
                            &attribute.span,
                            "There's already a field with the same long name or alias",
                            &original_field.name_span,
                            "Field with same long or alias value",
                        );

                        return Err(chic_error.to_string());
                    }

                    longs.insert(value, field);
                }
                AttributeType::Alias => {
                    let value = attribute.value.as_ref().unwrap().as_str();

                    if aliases.contains_key(value) && longs.contains_key(value) {
                        let original_field: &Field = aliases[value];
                        let chic_error = make_chic_error_with_info(
                            "Invalid field attribute usage",
                            source,
                            &attribute.span,
                            "There's already a field with the same alias or long name",
                            &original_field.name_span,
                            "Field with same alias or long value",
                        );

                        return Err(chic_error.to_string());
                    }

                    aliases.insert(value, field);
                }
                AttributeType::Flatten => match &field.ty {
                    FieldType::Vec(inner) => match inner.as_ref() {
                        FieldType::Vec(_) => unreachable!(),
                        FieldType::Struct(_) => {}
                        _ => {
                            return Err(make_chic_error(
                                "Invalid field attribute",
                                source,
                                &attribute.span,
                                "Flatten should be used with a custom type",
                            )
                            .to_string());
                        }
                    },
                    FieldType::Struct(_) => {}
                    _ => {
                        return Err(make_chic_error(
                            "Invalid field attribute",
                            source,
                            &attribute.span,
                            "Flatten should be used with a custom type",
                        )
                        .to_string());
                    }
                },
                AttributeType::Main | AttributeType::SubCommand => {
                    let help_msg = format!(
                        "Valid field attributes are: {}",
                        AttributeType::allowed_field_attribute_types()
                            .iter()
                            .map(|v| v.to_literal())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );

                    let chic_error = make_chic_error(
                        "Semantic error",
                        source,
                        &attribute.span,
                        "Invalid field attribute",
                    )
                    .help(help_msg.as_str());

                    return Err(chic_error.to_string());
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn check_semantics<'s>(spec: &'s Spec) -> Result<SpecMetadata<'s>, String> {
    let identifier_to_struct = check_for_multiple_struct_definitions(&spec.structs, spec.source)?;
    let mut spec_metadata = SpecMetadata::default();
    spec_metadata.identifier_to_struct = identifier_to_struct;

    for strukt in &spec.structs {
        check_for_undefined_types(&spec_metadata, &strukt.fields, spec.source)?;
        check_for_multiple_field_definitions(&strukt.fields, spec.source)?;
        check_struct_attributes(strukt, spec.source)?;
        check_field_attributes(&strukt.fields, spec.source)?;
    }

    Ok(spec_metadata)
}
