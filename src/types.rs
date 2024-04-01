use logos::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub(crate) enum AttributeType {
    Short,
    Long,
    Alias,
    Flatten,
    Main,
    SubCommand,
}

impl AttributeType {
    pub const fn to_literal(&self) -> &'static str {
        match self {
            AttributeType::Short => "short",
            AttributeType::Long => "long",
            AttributeType::Alias => "alias",
            AttributeType::Flatten => "flatten",
            AttributeType::Main => "main",
            AttributeType::SubCommand => "subcommand",
        }
    }

    pub const fn allowed_struct_attribute_types() -> &'static [AttributeType] {
        &[AttributeType::Main, AttributeType::SubCommand]
    }

    pub const fn allowed_field_attribute_types() -> &'static [AttributeType] {
        &[
            AttributeType::Short,
            AttributeType::Long,
            AttributeType::Alias,
            AttributeType::Flatten,
        ]
    }
}

#[derive(Debug)]
pub(crate) struct Attribute {
    pub ty: AttributeType,
    pub value: Option<String>,
    pub span: Span,
}

#[derive(Debug)]
pub(crate) enum FieldType {
    String,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Bool,
    Vec(Box<FieldType>),
    Optional(Box<FieldType>),
    Struct(String),
}

#[derive(Debug)]
pub(crate) struct Field {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub ty: FieldType,
    pub name_span: Span,
    pub type_span: Span,
}

impl Field {
    pub fn short_value(&self) -> Option<String> {
        self.attributes
            .iter()
            .find_map(|attr| matches!(attr.ty, AttributeType::Short).then(|| attr.value.as_ref()))
            .flatten()
            .map(|value| value.replace('_', "-"))
    }

    pub fn long_value(&self) -> Option<String> {
        self.attributes
            .iter()
            .find_map(|attr| matches!(attr.ty, AttributeType::Long).then(|| attr.value.as_ref()))
            .flatten()
            .map(|value| value.replace('_', "-"))
    }
}

#[derive(Debug)]
pub(crate) struct Struct {
    pub attributes: Vec<Attribute>,
    pub fields: Vec<Field>,
    pub name: String,
    pub name_span: Span,
}

impl Struct {
    pub fn get_fields<'s>(
        &'s self,
        spec_metadata: &'s SpecMetadata,
    ) -> impl Iterator<Item = &'s Field> {
        let flattened_fields_iter = self
            .fields
            .iter()
            .filter_map(|field| {
                field.attributes.iter().find_map(|attr| match attr.ty {
                    AttributeType::Flatten => match &field.ty {
                        FieldType::Vec(inner) => match inner.as_ref() {
                            FieldType::Vec(_) => unreachable!(),
                            FieldType::Struct(name) => {
                                Some(&spec_metadata.identifier_to_struct[name.as_str()].fields)
                            }
                            _ => None,
                        },
                        FieldType::Struct(name) => {
                            Some(&spec_metadata.identifier_to_struct[name.as_str()].fields)
                        }
                        _ => None,
                    },
                    _ => None,
                })
            })
            .flatten();

        self.fields.iter().chain(flattened_fields_iter)
    }
}

#[derive(Debug, Default)]
pub(crate) struct SpecMetadata<'s> {
    pub identifier_to_struct: HashMap<&'s str, &'s Struct>,
}

#[derive(Debug)]
pub(crate) struct Spec<'s> {
    pub structs: Vec<Struct>,
    pub source: &'s str,
}
