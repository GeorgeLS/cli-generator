use crate::types::{AttributeType, FieldType};
use logos::Logos;

#[derive(Debug, Logos, Copy, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub(crate) enum Tokens {
    // Symbols
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("#")]
    Pound,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("<")]
    LAngleBracket,
    #[token(">")]
    RAngleBracket,
    #[token("[")]
    LSquareBracket,
    #[token("]")]
    RSquareBracket,
    #[token("=")]
    Equals,

    // Attributes
    #[token("struct")]
    Struct,
    #[token("short")]
    Short,
    #[token("long")]
    Long,
    #[token("alias")]
    Alias,
    #[token("flatten")]
    Flatten,
    #[token("main")]
    Main,
    #[token("subcommand")]
    SubCommand,

    // Types
    #[token("string")]
    String,
    #[token("i16")]
    I16,
    #[token("u16")]
    U16,
    #[token("i32")]
    I32,
    #[token("u32")]
    U32,
    #[token("i64")]
    I64,
    #[token("u64")]
    U64,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("Vec")]
    Vec,
    #[token("Optional")]
    Optional,
    #[token("bool")]
    Bool,

    // Generic
    #[regex("[a-zA-Z_]+")]
    Identifier,
}

impl Tokens {
    pub const fn attribute_tokens() -> &'static [Self] {
        &[
            Tokens::Short,
            Tokens::Long,
            Tokens::Alias,
            Tokens::Equals,
            Tokens::Comma,
            Tokens::Flatten,
            Tokens::Main,
            Tokens::SubCommand,
        ]
    }

    pub const fn type_tokens() -> &'static [Self] {
        &[
            Tokens::String,
            Tokens::I16,
            Tokens::U16,
            Tokens::I32,
            Tokens::U32,
            Tokens::I64,
            Tokens::U64,
            Tokens::F32,
            Tokens::F64,
            Tokens::Vec,
            Tokens::Optional,
            Tokens::Bool,
            Tokens::Identifier,
        ]
    }

    pub fn as_attribute_type(&self) -> AttributeType {
        match self {
            Tokens::Short => AttributeType::Short,
            Tokens::Long => AttributeType::Long,
            Tokens::Alias => AttributeType::Alias,
            Tokens::Flatten => AttributeType::Flatten,
            Tokens::Main => AttributeType::Main,
            Tokens::SubCommand => AttributeType::SubCommand,
            _ => unreachable!(),
        }
    }

    pub fn as_field_type(&self) -> FieldType {
        match self {
            Tokens::String => FieldType::String,
            Tokens::I16 => FieldType::I16,
            Tokens::U16 => FieldType::U16,
            Tokens::I32 => FieldType::I32,
            Tokens::U32 => FieldType::U32,
            Tokens::I64 => FieldType::I64,
            Tokens::U64 => FieldType::U64,
            Tokens::F32 => FieldType::F32,
            Tokens::F64 => FieldType::F64,
            Tokens::Bool => FieldType::Bool,
            Tokens::Vec => FieldType::Vec(Box::new(FieldType::I16)),
            Tokens::Optional => FieldType::Optional(Box::new(FieldType::I16)),
            Tokens::Identifier => FieldType::Struct(String::new()),
            _ => unreachable!(),
        }
    }

    pub const fn as_token_literal(&self) -> &'static str {
        match self {
            Tokens::LBrace => "{",
            Tokens::RBrace => "}",
            Tokens::Pound => "#",
            Tokens::Colon => ":",
            Tokens::Comma => ",",
            Tokens::LAngleBracket => "<",
            Tokens::RAngleBracket => ">",
            Tokens::LSquareBracket => "[",
            Tokens::RSquareBracket => "]",
            Tokens::Equals => "=",
            Tokens::Struct => "struct",
            Tokens::Short => "short",
            Tokens::Long => "long",
            Tokens::Alias => "alias",
            Tokens::Flatten => "flatten",
            Tokens::Main => "main",
            Tokens::SubCommand => "subcommand",
            Tokens::String => "string",
            Tokens::I16 => "i16",
            Tokens::U16 => "u16",
            Tokens::I32 => "i32",
            Tokens::U32 => "u32",
            Tokens::I64 => "i64",
            Tokens::U64 => "u64",
            Tokens::F32 => "f32",
            Tokens::F64 => "f64",
            Tokens::Vec => "Vec",
            Tokens::Optional => "Optional",
            Tokens::Bool => "bool",
            Tokens::Identifier => "regex: [a-z,A-Z_]+",
        }
    }
}
