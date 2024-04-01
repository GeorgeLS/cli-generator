use crate::generate::left_pad;
use crate::types::{AttributeType, Field, FieldType, Spec, SpecMetadata, Struct};
use std::collections::HashMap;
use std::fmt::Write;

fn field_type_to_cpp_type(field_type: &FieldType) -> String {
    match field_type {
        FieldType::String => "std::string".to_string(),
        FieldType::I16 => "int16_t".to_string(),
        FieldType::U16 => "uint16_t".to_string(),
        FieldType::I32 => "int32_t".to_string(),
        FieldType::U32 => "uint32_t".to_string(),
        FieldType::I64 => "int64_t".to_string(),
        FieldType::U64 => "uint64_t".to_string(),
        FieldType::F32 => "float".to_string(),
        FieldType::F64 => "double".to_string(),
        FieldType::Bool => "bool".to_string(),
        FieldType::Vec(inner) => format!("std::vector<{}>", field_type_to_cpp_type(inner)),
        FieldType::Optional(inner) => format!("std::optional<{}>", field_type_to_cpp_type(inner)),
        FieldType::Struct(strukt) => strukt.to_string(),
    }
}

#[derive(Debug, Default)]
struct CppSourceBuilder {
    buffer: String,
    indentation: usize,
}

macro_rules! cpp_source_builder_writeln {
    ($self:expr) => {{
        writeln!($self.buffer).unwrap();
    }};
    ($self:expr, $($arg:tt)*) => {{
        if $self.indentation != 0 {
            left_pad($self.indentation, &mut $self.buffer).unwrap();
        }
        writeln!($self.buffer, $($arg)*).unwrap();
    }};
}

macro_rules! cpp_source_builder_write {
    ($self:expr) => {{
        write!($self.buffer).unwrap()
    }};
    ($self:expr, $($arg:tt)*) => {{
        if $self.indentation != 0 {
            left_pad($self.indentation, &mut $self.buffer).unwrap();
        }
        write!($self.buffer, $($arg)*).unwrap()
    }};
}

impl CppSourceBuilder {
    #[inline]
    pub fn push_indentation_level(&mut self) {
        self.indentation += 4;
    }

    #[inline]
    pub fn pop_indentation_level(&mut self) {
        if self.indentation >= 4 {
            self.indentation -= 4;
        }
    }

    #[inline]
    pub fn get_indentation_level(&self) -> usize {
        self.indentation
    }

    #[inline]
    pub fn set_indentation_level(&mut self, indentation: usize) {
        self.indentation = indentation;
    }
}

impl CppSourceBuilder {
    #[inline]
    pub fn result(self) -> String {
        self.buffer
    }

    #[inline]
    pub fn write_header_guard_start(&mut self) {
        cpp_source_builder_writeln!(self, "#ifndef _CLI_H_");
        cpp_source_builder_writeln!(self, "#define _CLI_H_");
        cpp_source_builder_writeln!(self);
    }

    #[inline]
    pub fn write_header_guard_end(&mut self) {
        cpp_source_builder_writeln!(self, "#endif // _CLI_H_");
    }

    #[inline]
    pub fn write_include_headers(&mut self) {
        cpp_source_builder_writeln!(self, "#include <cstdint>");
        cpp_source_builder_writeln!(self, "#include <cstdlib>");
        cpp_source_builder_writeln!(self, "#include <cstring>");
        cpp_source_builder_writeln!(self, "#include <cstdio>");
        cpp_source_builder_writeln!(self, "#include <cerrno>");
        cpp_source_builder_writeln!(self, "#include <string>");
        cpp_source_builder_writeln!(self, "#include <vector>");
        cpp_source_builder_writeln!(self);
    }

    #[inline]
    pub fn write_struct_start(&mut self, struct_name: &str) {
        cpp_source_builder_writeln!(self, "struct {struct_name} {{");
    }

    #[inline]
    pub fn write_struct_end(&mut self) {
        cpp_source_builder_writeln!(self, "}};\n");
    }

    #[inline]
    pub fn write_struct_field(&mut self, field: &Field) {
        let field_type = field_type_to_cpp_type(&field.ty);
        let field_name = &field.name;
        self.push_indentation_level();
        cpp_source_builder_writeln!(self, "{field_type} {field_name};");
        self.pop_indentation_level();
    }

    pub fn write_parse_numeric_field(&mut self, field_type: &FieldType) {
        let cpp_type = field_type_to_cpp_type(field_type);
        let conversion_function = match field_type {
            FieldType::I16
            | FieldType::U16
            | FieldType::I32
            | FieldType::U32
            | FieldType::I64
            | FieldType::U64 => "std::strtoll(arg_value, nullptr, 10)",
            FieldType::F32 => "std::strtof(arg_value, nullptr)",
            FieldType::F64 => "std::strtod(arg_value, nullptr)",
            _ => unreachable!(),
        };

        cpp_source_builder_writeln!(self, "char* arg_value = args[0];");
        cpp_source_builder_writeln!(
            self,
            "{cpp_type} arg_res = static_cast<{cpp_type}>({conversion_function});"
        );
        cpp_source_builder_writeln!(self);

        cpp_source_builder_writeln!(self, "if (errno == ERANGE) {{");
        self.push_indentation_level();
        cpp_source_builder_writeln!(
            self,
            r#"printf("Value '%s' of option '%s' out of range for integer type", arg_value, arg);"#
        );
        cpp_source_builder_writeln!(self, "exit(1);");
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");

        cpp_source_builder_writeln!(self, r#"if (arg_res == 0 && strcmp(arg, "0") != 0) {{"#);
        self.push_indentation_level();
        cpp_source_builder_writeln!(
            self,
            r#"printf("Value '%s' of option '%s' is not a valid integer", arg_value, arg);"#
        );
        cpp_source_builder_writeln!(self, "exit(1);");
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");
    }

    pub fn write_parse_field_type(&mut self, struct_name: &str, field_type: &FieldType) {
        fn write_parse_value_option_preamble(
            self_: &mut CppSourceBuilder,
            struct_name: &str,
            is_string: bool,
        ) {
            cpp_source_builder_writeln!(self_, "++args;");
            cpp_source_builder_writeln!(self_, "++i;");

            cpp_source_builder_write!(self_, "if (i == argc");
            let indentation_level = self_.get_indentation_level();
            self_.set_indentation_level(0);
            if is_string {
                cpp_source_builder_writeln!(self_, ") {{");
            } else {
                cpp_source_builder_writeln!(self_, " || {struct_name}::is_option(args[0])) {{");
            }
            self_.set_indentation_level(indentation_level);
            self_.push_indentation_level();

            cpp_source_builder_writeln!(
                self_,
                r#"printf("Expected value for option '%s' but no value was provided", arg);"#
            );
            cpp_source_builder_writeln!(self_, "exit(1);");

            self_.pop_indentation_level();
            cpp_source_builder_writeln!(self_, "}}");
        }

        match field_type {
            FieldType::Vec(_) | FieldType::Bool => {}
            _ => write_parse_value_option_preamble(
                self,
                struct_name,
                matches!(field_type, FieldType::String),
            ),
        }

        match field_type {
            FieldType::String => {
                cpp_source_builder_writeln!(self, "std::string arg_res = args[0];");
            }
            FieldType::I16
            | FieldType::U16
            | FieldType::I32
            | FieldType::U32
            | FieldType::I64
            | FieldType::U64
            | FieldType::F32
            | FieldType::F64 => {
                self.write_parse_numeric_field(field_type);
            }
            FieldType::Bool => {
                cpp_source_builder_writeln!(self, "bool arg_res = true;");
            }
            FieldType::Struct(struct_name) => {
                cpp_source_builder_writeln!(
                    self,
                    "{struct_name} arg_res = {struct_name}::parse(argc - i, args);"
                );
            }
            FieldType::Vec(inner) => {
                self.write_parse_field_type(struct_name, inner);
            }
            FieldType::Optional(inner) => {
                self.write_parse_field_type(struct_name, inner);
            }
        }
    }

    fn write_parse_fields_r(
        &mut self,
        struct_name: &str,
        fields: &[Field],
        spec_metadata: &SpecMetadata,
        parents: &mut Vec<String>,
        mandatory_field_to_index: &HashMap<&str, usize>,
    ) {
        let mut match_fields_buffer = Vec::new();

        for field in fields {
            for attr in &field.attributes {
                match attr.ty {
                    AttributeType::Short => {
                        let arg_match = format!("-{}", field.short_value().unwrap());
                        match_fields_buffer.push(arg_match);
                    }
                    AttributeType::Long => {
                        let arg_match = format!("--{}", field.long_value().unwrap());
                        match_fields_buffer.push(arg_match);
                    }
                    AttributeType::Alias => {
                        let value = attr.value.as_ref().unwrap();
                        let arg_match = format!("--{}", value.replace('_', "-"));
                        match_fields_buffer.push(arg_match);
                    }
                    AttributeType::Flatten => {
                        let flatten_type = match &field.ty {
                            FieldType::Vec(inner) => match inner.as_ref() {
                                FieldType::Struct(name) => {
                                    spec_metadata.identifier_to_struct[name.as_str()]
                                }
                                _ => unreachable!(),
                            },
                            FieldType::Struct(name) => {
                                spec_metadata.identifier_to_struct[name.as_str()]
                            }
                            _ => unreachable!(),
                        };
                        parents.push(field.name.clone());
                        self.write_parse_fields_r(
                            struct_name,
                            &flatten_type.fields,
                            spec_metadata,
                            parents,
                            mandatory_field_to_index,
                        );
                    }
                    _ => unreachable!(),
                }
            }

            if !match_fields_buffer.is_empty() {
                let field_matcher = match_fields_buffer
                    .drain(..)
                    .map(|arg_match| format!(r#"strcmp(arg, "{arg_match}") == 0"#))
                    .collect::<Vec<_>>()
                    .join(" || ");

                let indentation_level = self.get_indentation_level();
                self.set_indentation_level(1);
                cpp_source_builder_writeln!(self, "else if ({field_matcher}) {{");
                self.set_indentation_level(indentation_level);

                self.push_indentation_level();

                self.write_parse_field_type(struct_name, &field.ty);

                let destination = parents.join(".");

                match &field.ty {
                    FieldType::Vec(_) => {
                        cpp_source_builder_writeln!(
                            self,
                            "{destination}.{}.push_back(arg_res);",
                            field.name
                        );
                    }
                    _ => {
                        cpp_source_builder_writeln!(
                            self,
                            "{destination}.{} = arg_res;",
                            field.name
                        );
                    }
                }

                if let Some(index) = mandatory_field_to_index.get(field.name.as_str()) {
                    cpp_source_builder_writeln!(self, "mandatory_fields_seen[{index}] = true;")
                }

                self.pop_indentation_level();
                cpp_source_builder_write!(self, "}}");
            }
        }
        parents.pop();
    }

    pub fn write_parse_fields(
        &mut self,
        struct_name: &str,
        fields: &[Field],
        spec_metadata: &SpecMetadata,
        mandatory_field_to_index: &HashMap<&str, usize>,
    ) {
        let mut parents = vec!["res".to_string()];
        self.write_parse_fields_r(
            struct_name,
            fields,
            spec_metadata,
            &mut parents,
            mandatory_field_to_index,
        )
    }

    pub fn write_struct_parse_method(&mut self, strukt: &Struct, spec_metadata: &SpecMetadata) {
        cpp_source_builder_writeln!(self);

        let struct_name = &strukt.name;

        self.push_indentation_level();
        cpp_source_builder_writeln!(
            self,
            "static {struct_name} parse (int argc, char *args[]) {{"
        );

        self.push_indentation_level();

        if strukt
            .attributes
            .iter()
            .any(|attr| matches!(attr.ty, AttributeType::Main))
        {
            cpp_source_builder_writeln!(self, "--argc;");
            cpp_source_builder_writeln!(self, "++args;\n");
        }

        cpp_source_builder_write!(self, "const char* mandatory_field_names[] = {{");

        let indentation_level = self.get_indentation_level();
        self.set_indentation_level(1);

        let mut mandatory_field_name_to_index = HashMap::new();

        for (i, field) in strukt
            .fields
            .iter()
            .filter(|f| !matches!(f.ty, FieldType::Optional(_)))
            .enumerate()
        {
            cpp_source_builder_write!(self, r#""{}","#, field.name);
            mandatory_field_name_to_index.insert(field.name.as_str(), i);
        }

        cpp_source_builder_writeln!(self, "}};");
        self.set_indentation_level(indentation_level);

        cpp_source_builder_writeln!(
            self,
            "bool mandatory_fields_seen[sizeof(mandatory_field_names)/sizeof(mandatory_field_names[0])] = {{ false }};\n"
        );

        cpp_source_builder_writeln!(self, "{struct_name} res = {{}};");
        cpp_source_builder_writeln!(self, "for (int i = 0; i != argc; ++i, ++args) {{");

        self.push_indentation_level();
        cpp_source_builder_writeln!(self, "char *arg = args[0];");
        cpp_source_builder_writeln!(
            self,
            r#"if (strcmp("-h", arg) == 0 || strcmp("--help", arg) == 0) {{"#
        );
        self.push_indentation_level();
        cpp_source_builder_writeln!(self, "{struct_name}::help();");
        self.pop_indentation_level();
        cpp_source_builder_write!(self, "}}");

        self.write_parse_fields(
            strukt.name.as_str(),
            &strukt.fields,
            spec_metadata,
            &mandatory_field_name_to_index,
        );

        let indentation_level = self.get_indentation_level();
        self.set_indentation_level(1);
        cpp_source_builder_writeln!(self, "else {{");
        self.set_indentation_level(indentation_level);
        self.push_indentation_level();
        cpp_source_builder_writeln!(self, r#"printf("Unknown option '%s'\n", arg);"#);
        cpp_source_builder_writeln!(self, "exit(1);");
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");

        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}\n");

        cpp_source_builder_writeln!(self, "bool not_seen_any = false;");
        cpp_source_builder_writeln!(
            self,
            "for (size_t i = 0; i != sizeof(mandatory_field_names)/sizeof(mandatory_field_names[0]); ++i) {{"
        );
        self.push_indentation_level();

        cpp_source_builder_writeln!(self, "if (!mandatory_fields_seen[i]) {{");
        self.push_indentation_level();
        cpp_source_builder_writeln!(
            self,
            r#"printf("--%s was required but it was not provided\n", mandatory_field_names[i]);"#
        );
        cpp_source_builder_writeln!(self, "not_seen_any = true;");
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");

        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");

        cpp_source_builder_writeln!(self, "if (not_seen_any) {{");
        self.push_indentation_level();
        cpp_source_builder_writeln!(self, "exit(1);");
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");

        cpp_source_builder_writeln!(self, "return res;");

        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");
        self.pop_indentation_level();
    }

    pub fn write_struct_help_method(&mut self, strukt: &Struct, spec_metadata: &SpecMetadata) {
        cpp_source_builder_writeln!(self);

        self.push_indentation_level();
        cpp_source_builder_writeln!(self, "static void help() {{");
        self.push_indentation_level();
        cpp_source_builder_writeln!(self, r#"printf("Usage: {} [OPTIONS]\n""#, strukt.name);
        cpp_source_builder_writeln!(self, r#""\n""#);
        cpp_source_builder_writeln!(self, r#""Options:\n""#);
        cpp_source_builder_writeln!(self, r#""    -h, --help\n""#);

        let identation_level = self.get_indentation_level();
        for field in strukt.get_fields(spec_metadata) {
            cpp_source_builder_write!(self, "\"    ");
            self.set_indentation_level(0);
            if let Some(short_value) = field.short_value() {
                cpp_source_builder_write!(self, "-{short_value}");
            }

            if let Some(long_value) = field.long_value() {
                if field.short_value().is_some() {
                    cpp_source_builder_write!(self, ", ")
                }
                cpp_source_builder_write!(self, "--{long_value}");
            }

            if !matches!(field.ty, FieldType::Bool) {
                cpp_source_builder_write!(self, " <{}>", field.name.to_uppercase());
            }

            cpp_source_builder_writeln!(self, r#"\n""#);
            self.set_indentation_level(identation_level);
        }
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, ");");

        cpp_source_builder_writeln!(self, "exit(0);");
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");
        self.pop_indentation_level();
    }

    pub fn write_is_option_method(&mut self, strukt: &Struct, spec_metadata: &SpecMetadata) {
        cpp_source_builder_writeln!(self);

        self.push_indentation_level();
        cpp_source_builder_writeln!(self, "static bool is_option(char* arg) {{");

        self.push_indentation_level();
        cpp_source_builder_writeln!(self, "static const char* valid_options[] = {{");
        self.push_indentation_level();

        let mut num_fields = 0;
        for field in strukt.get_fields(spec_metadata) {
            if let Some(short_value) = field.short_value() {
                cpp_source_builder_writeln!(self, r#""-{short_value}","#);
                num_fields += 1;
            }

            if let Some(long_value) = field.long_value() {
                cpp_source_builder_writeln!(self, r#""--{long_value}","#);
                num_fields += 1;
            }
        }
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}};");

        cpp_source_builder_writeln!(self);

        cpp_source_builder_writeln!(self, "for (size_t i = 0; i != {num_fields}; ++i) {{");
        self.push_indentation_level();

        cpp_source_builder_writeln!(self, "if (strcmp(arg, valid_options[i]) == 0) {{");
        self.push_indentation_level();

        cpp_source_builder_writeln!(self, "return true;");

        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");

        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");

        cpp_source_builder_writeln!(self);
        cpp_source_builder_writeln!(self, "return false;");

        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");
        self.pop_indentation_level();
    }

    pub fn write_debug_print_method(&mut self, strukt: &Struct) {
        fn field_to_print_statement(field: &Field) -> String {
            match &field.ty {
                FieldType::String => {
                    format!(r#"printf("\t{0}: %s\n", this->{0}.c_str());"#, field.name)
                }
                FieldType::I16 => format!(r#"printf("\t{0}: %d\n", this->{0});"#, field.name),
                FieldType::U16 => format!(r#"printf("\t{0}: %d\n", this->{0});"#, field.name),
                FieldType::I32 => format!(r#"printf("\t{0}: %d\n", this->{0});"#, field.name),
                FieldType::U32 => format!(r#"printf("\t{0}: %d\n", this->{0});"#, field.name),
                FieldType::I64 => format!(r#"printf("\t{0}: %d\n", this->{0});"#, field.name),
                FieldType::U64 => format!(r#"printf("\t{0}: %d\n", this->{0});"#, field.name),
                FieldType::F32 => format!(r#"printf("\t{0}: %f\n", this->{0});"#, field.name),
                FieldType::F64 => format!(r#"printf("\t{0}: %f\n", this->{0});"#, field.name),
                FieldType::Bool => format!(
                    r#"printf("\t{0}: %s\n", this->{0} ? "true" : "false");"#,
                    field.name
                ),
                FieldType::Vec(inner) => match inner.as_ref() {
                    FieldType::String => {
                        format!(r#"printf("\t%s,\n", this->{}[i].c_str());"#, field.name)
                    }
                    FieldType::I16 => format!(r#"printf("\t%d,\n", this->{}[i]);"#, field.name),
                    FieldType::U16 => format!(r#"printf("\t%d,\n", this->{}[i]);"#, field.name),
                    FieldType::I32 => format!(r#"printf("\t%d,\n", this->{}[i]);"#, field.name),
                    FieldType::U32 => format!(r#"printf("\t%d,\n", this->{}[i]);"#, field.name),
                    FieldType::I64 => format!(r#"printf("\t%d,\n", this->{}[i]);"#, field.name),
                    FieldType::U64 => format!(r#"printf("\t%d,\n", this->{}[i]);"#, field.name),
                    FieldType::F32 => format!(r#"printf("\t%f,\n", this->{}[i]);"#, field.name),
                    FieldType::F64 => format!(r#"printf("\t%f,\n", this->{}[i]);"#, field.name),
                    FieldType::Bool => format!(
                        r#"printf("\t%s,\n", this->{}[i] ? "true" : "false");"#,
                        field.name
                    ),
                    FieldType::Vec(_) => unreachable!(),
                    FieldType::Optional(_) => unreachable!(),
                    FieldType::Struct(_) => format!("this->{}[i].print_debug();", field.name),
                },
                FieldType::Struct(_) => format!("this->{}.print_debug();", field.name),
                FieldType::Optional(inner) => match inner.as_ref() {
                    FieldType::String => {
                        format!(r#"printf("\t%s,\n", this->{}[i].c_str());"#, field.name)
                    }
                    FieldType::I16 => {
                        format!(r#"printf("\t%d,\n", this->{}[i].value());"#, field.name)
                    }
                    FieldType::U16 => {
                        format!(r#"printf("\t%d,\n", this->{}[i].value());"#, field.name)
                    }
                    FieldType::I32 => {
                        format!(r#"printf("\t%d,\n", this->{}[i].value());"#, field.name)
                    }
                    FieldType::U32 => {
                        format!(r#"printf("\t%d,\n", this->{}[i].value());"#, field.name)
                    }
                    FieldType::I64 => {
                        format!(r#"printf("\t%d,\n", this->{}[i].value());"#, field.name)
                    }
                    FieldType::U64 => {
                        format!(r#"printf("\t%d,\n", this->{}[i].value());"#, field.name)
                    }
                    FieldType::F32 => {
                        format!(r#"printf("\t%f,\n", this->{}[i].value());"#, field.name)
                    }
                    FieldType::F64 => {
                        format!(r#"printf("\t%f,\n", this->{}[i].value());"#, field.name)
                    }
                    FieldType::Bool => format!(
                        r#"printf("\t%s,\n", this->{}[i].value() ? "true" : "false");"#,
                        field.name
                    ),
                    FieldType::Vec(_) => unreachable!(),
                    FieldType::Optional(_) => unreachable!(),
                    FieldType::Struct(_) => {
                        format!("this->{}[i].value().print_debug();", field.name)
                    }
                },
            }
        }
        cpp_source_builder_writeln!(self);
        self.push_indentation_level();

        cpp_source_builder_writeln!(self, "void print_debug() {{");
        self.push_indentation_level();

        cpp_source_builder_writeln!(self, r#"printf("{} {{\n");"#, strukt.name);
        for field in &strukt.fields {
            let print_statement = field_to_print_statement(field);
            match field.ty {
                FieldType::Vec(_) => {
                    cpp_source_builder_writeln!(self, r#"printf("\t{}: [\n");"#, field.name);
                    cpp_source_builder_writeln!(
                        self,
                        "for (size_t i = 0; i != this->{}.size(); ++i) {{",
                        field.name
                    );
                    self.push_indentation_level();
                    cpp_source_builder_writeln!(self, "{print_statement}");
                    self.pop_indentation_level();
                    cpp_source_builder_writeln!(self, "}}");
                    cpp_source_builder_writeln!(self, r#"printf("\t]\n");"#);
                }
                _ => cpp_source_builder_writeln!(self, "{print_statement}"),
            }
        }

        cpp_source_builder_writeln!(self, r#"printf("}}\n");"#);
        self.pop_indentation_level();
        cpp_source_builder_writeln!(self, "}}");

        self.pop_indentation_level();
    }
}

pub(crate) fn generate_cli(spec: &Spec, spec_metadata: &SpecMetadata) -> String {
    let mut source_builder = CppSourceBuilder::default();

    source_builder.write_header_guard_start();
    source_builder.write_include_headers();

    for strukt in &spec.structs {
        source_builder.write_struct_start(&strukt.name);

        for field in &strukt.fields {
            source_builder.write_struct_field(field);
        }

        source_builder.write_debug_print_method(strukt);
        source_builder.write_struct_help_method(strukt, spec_metadata);
        source_builder.write_is_option_method(strukt, spec_metadata);
        source_builder.write_struct_parse_method(strukt, spec_metadata);

        source_builder.write_struct_end();
    }

    source_builder.write_header_guard_end();

    source_builder.result()
}
