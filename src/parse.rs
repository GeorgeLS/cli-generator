use crate::lexer::Tokens;
use crate::types::{Attribute, AttributeType, Field, FieldType, Spec, Struct};
use logos::{Logos, Span, SpannedIter};
use std::iter::Peekable;

type LexerType<'s> = Peekable<SpannedIter<'s, Tokens>>;

pub(crate) struct Parser<'s> {
    source: &'s str,
    lexer: LexerType<'s>,
}

struct ParserToken {
    pub token: Tokens,
    pub span: Span,
}

impl ParserToken {
    pub fn new(token: Tokens, span: Span) -> Self {
        Self { token, span }
    }
}

impl<'s> Parser<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            lexer: Tokens::lexer(source).spanned().peekable(),
        }
    }

    #[inline]
    fn make_end_of_file_chic_error(&self) -> String {
        chic::Error::new("Parser error")
            .error(
                1,
                self.source.trim_end().len() - 1,
                self.source.trim_end().len(),
                self.source,
                "Unexpected end of file",
            )
            .to_string()
    }

    #[inline]
    fn make_chic_error_for_lexer_error(&self, span: &Span) -> String {
        chic::Error::new("Lexer error")
            .error(1, span.start, span.end, self.source, "Unknown token")
            .to_string()
    }

    #[inline]
    fn make_chic_error_for_parse_error(
        &self,
        span: &Span,
        message: &'s str,
        help: Option<&'s str>,
    ) -> String {
        let mut err =
            chic::Error::new("Parse error").error(1, span.start, span.end, self.source, message);

        if let Some(help) = help {
            err = err.help(help);
        }

        err.to_string()
    }

    #[inline]
    fn ensure_token_any_of(&self, token: &ParserToken, expected: &[Tokens]) -> Result<(), String> {
        if expected.contains(&token.token) {
            Ok(())
        } else {
            Err(chic::Error::new("Parser error")
                .error(
                    1,
                    token.span.start,
                    token.span.end,
                    self.source,
                    "Unexpected token",
                )
                .help(&format!(
                    "Tokens can be any of: {}",
                    expected
                        .iter()
                        .map(|v| v.as_token_literal())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .to_string())
        }
    }

    #[inline]
    fn ensure_next_token_any_of(&mut self, tokens: &[Tokens]) -> Result<ParserToken, String> {
        let next_token = self
            .next_token()
            .ok_or_else(|| self.make_end_of_file_chic_error())??;

        self.ensure_token_any_of(&next_token, tokens)?;

        Ok(next_token)
    }

    #[inline]
    fn ensure_next_token(&mut self, token: Tokens) -> Result<ParserToken, String> {
        self.ensure_next_token_any_of(&[token])
    }

    #[inline]
    fn next_token(&mut self) -> Option<Result<ParserToken, String>> {
        let res = self.peek_token();
        let _ = self.lexer.next();
        res
    }

    #[inline]
    fn peek_token(&mut self) -> Option<Result<ParserToken, String>> {
        let (token_res, span) = {
            let (token_res, span) = self.lexer.peek()?;
            (token_res.clone(), span.clone())
        };

        match token_res {
            Ok(token) => Some(Ok(ParserToken::new(token, span))),
            Err(_) => Some(Err(self.make_chic_error_for_lexer_error(&span))),
        }
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, String> {
        let mut res = Vec::new();

        let attributes_start = self.ensure_next_token(Tokens::Pound)?;
        self.ensure_next_token(Tokens::LSquareBracket)?;

        loop {
            let Some(next_token) = self.next_token() else {
                return Err(self.make_end_of_file_chic_error());
            };

            let next_token = next_token?;

            if matches!(next_token.token, Tokens::RSquareBracket) {
                break;
            }

            if matches!(next_token.token, Tokens::Comma) {
                continue;
            }

            self.ensure_token_any_of(&next_token, Tokens::attribute_tokens())?;

            let ty = next_token.token.as_attribute_type();

            let value = match ty {
                AttributeType::Short | AttributeType::Long => {
                    let Some(next_token) = self.peek_token() else {
                        return Err(self.make_end_of_file_chic_error());
                    };

                    let next_token = next_token?;

                    if matches!(next_token.token, Tokens::Equals) {
                        let _ = self.next_token();
                        let id_token = self.ensure_next_token(Tokens::Identifier)?;
                        Some(&self.source[id_token.span.start..id_token.span.end])
                    } else {
                        None
                    }
                }
                AttributeType::Alias => {
                    self.ensure_next_token(Tokens::Equals)?;
                    let id_token = self.ensure_next_token(Tokens::Identifier)?;
                    Some(&self.source[id_token.span.start..id_token.span.end])
                }
                _ => None,
            };

            let value = value.map(String::from);

            let attribute = Attribute {
                ty,
                value,
                span: next_token.span,
            };

            res.push(attribute)
        }

        if res.is_empty() {
            return Err(self.make_chic_error_for_parse_error(
                &attributes_start.span,
                "Attributes cannot be empty",
                None,
            ));
        }

        Ok(res)
    }

    fn parse_field(&mut self) -> Result<Field, String> {
        let id_token = self.ensure_next_token(Tokens::Identifier)?;
        let name = self.source[id_token.span.start..id_token.span.end].to_string();

        self.ensure_next_token(Tokens::Colon)?;

        let mut ty_token = self.ensure_next_token_any_of(Tokens::type_tokens())?;
        let mut ty = ty_token.token.as_field_type();

        if matches!(ty, FieldType::Vec(_) | FieldType::Optional(_)) {
            let inner = match &mut ty {
                FieldType::Vec(inner) => inner,
                FieldType::Optional(inner) => inner,
                _ => unreachable!(),
            };

            self.ensure_next_token(Tokens::LAngleBracket)?;
            let inner_ty_token = self.ensure_next_token_any_of(Tokens::type_tokens())?;
            self.ensure_next_token(Tokens::RAngleBracket)?;

            *inner.as_mut() = inner_ty_token.token.as_field_type();

            if let FieldType::Struct(inner) = inner.as_mut() {
                inner.push_str(&self.source[inner_ty_token.span.start..inner_ty_token.span.end]);
            };

            ty_token = inner_ty_token;
        } else if matches!(ty, FieldType::Struct(_)) {
            let FieldType::Struct(inner) = &mut ty else {
                unreachable!()
            };

            inner.push_str(&self.source[ty_token.span.start..ty_token.span.end]);
        }

        if let Some(token) = self.peek_token() {
            let token = token?;
            if matches!(token.token, Tokens::Comma) {
                let _ = self.next_token();
            }
        }

        let res = Field {
            name,
            attributes: Vec::new(),
            ty,
            name_span: id_token.span,
            type_span: ty_token.span,
        };

        Ok(res)
    }

    fn parse_struct(&mut self) -> Result<Struct, String> {
        self.ensure_next_token(Tokens::Struct)?;

        let id_token = self.ensure_next_token(Tokens::Identifier)?;
        let name = self.source[id_token.span.start..id_token.span.end].to_string();

        self.ensure_next_token(Tokens::LBrace)?;

        let mut fields = Vec::new();

        while let Some(token) = self.peek_token() {
            let token = token?;
            if matches!(token.token, Tokens::RBrace) {
                break;
            }

            match token.token {
                Tokens::Pound => {
                    let mut attributes = self.parse_attributes()?;
                    let mut field = self.parse_field()?;

                    for attribute in &mut attributes {
                        if matches!(attribute.ty, AttributeType::Short) && attribute.value.is_none()
                        {
                            attribute.value =
                                Some(String::from(field.name.chars().next().unwrap()));
                        } else if matches!(attribute.ty, AttributeType::Long)
                            && attribute.value.is_none()
                        {
                            attribute.value = Some(field.name.clone());
                        }
                    }

                    field.attributes = attributes;
                    fields.push(field);
                }
                Tokens::Identifier => {
                    let field = self.parse_field()?;
                    fields.push(field);
                }
                _ => unreachable!(),
            }
        }

        self.ensure_next_token(Tokens::RBrace)?;

        let strukt = Struct {
            attributes: Vec::new(),
            fields,
            name,
            name_span: id_token.span,
        };

        Ok(strukt)
    }

    pub fn parse(&mut self) -> Result<Spec, String> {
        let mut structs = Vec::new();

        while let Some(parser_token) = self.peek_token() {
            let parser_token = parser_token?;
            match parser_token.token {
                Tokens::Pound => {
                    let attributes = self.parse_attributes()?;

                    let Some(parser_token) = self.peek_token() else {
                        return Err(self.make_end_of_file_chic_error());
                    };

                    let parser_token = parser_token?;

                    match parser_token.token {
                        Tokens::Struct => {
                            let mut strukt = self.parse_struct()?;
                            strukt.attributes.extend(attributes);
                            structs.push(strukt);
                        }
                        _ => unreachable!(),
                    }
                }
                Tokens::Struct => {
                    let strukt = self.parse_struct()?;
                    structs.push(strukt);
                }
                _ => unreachable!(),
            }
        }

        let res = Spec {
            structs,
            source: self.source,
        };

        Ok(res)
    }
}
