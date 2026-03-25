use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Data, DeriveInput, Expr, ExprLit, Field, Fields, GenericArgument, Ident, Lit, LitStr, Meta,
    MetaNameValue, PathArguments, Type,
};

#[proc_macro_derive(LoggableEvent, attributes(log_event, log))]
pub fn derive_loggable_event(input: TokenStream) -> TokenStream {
    derive_loggable_event_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(LogValue, attributes(log_value))]
pub fn derive_log_value(input: TokenStream) -> TokenStream {
    derive_log_value_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_loggable_event_impl(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let enum_ident = input.ident.clone();
    let data = match input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "LoggableEvent can only be derived for enums",
            ));
        }
    };

    let enum_meta = parse_enum_log_event_meta(&input.attrs)?;
    let variants = data
        .variants
        .into_iter()
        .map(|variant| build_variant_arm(&enum_ident, &enum_meta, variant))
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        impl crate::logging::event::sealed::Sealed for #enum_ident {}

        impl crate::logging::event::LoggableEvent for #enum_ident {
            fn into_log_event(self) -> crate::logging::event::LogEventDto {
                match self {
                    #(#variants),*
                }
            }
        }
    })
}

fn derive_log_value_impl(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let enum_ident = input.ident.clone();
    let rename_all = parse_log_value_meta(&input.attrs)?;
    let data = match input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "LogValue can only be derived for enums",
            ));
        }
    };

    let rename_all = rename_all.ok_or_else(|| {
        syn::Error::new(
            enum_ident.span(),
            "LogValue requires #[log_value(rename_all = \"snake_case\")] or #[log_value(rename_all = \"lowercase\")]",
        )
    })?;
    if rename_all != "snake_case" && rename_all != "lowercase" {
        return Err(syn::Error::new(
            enum_ident.span(),
            format!(
                "unsupported log_value rename_all `{rename_all}`; accepted values: snake_case, lowercase"
            ),
        ));
    }

    let arms = data
        .variants
        .into_iter()
        .map(|variant| {
            if !matches!(variant.fields, Fields::Unit) {
                return Err(syn::Error::new(
                    variant.ident.span(),
                    "LogValue only supports unit enum variants",
                ));
            }
            let variant_ident = variant.ident;
            let value = match rename_all.as_str() {
                "snake_case" => variant_ident.to_string().to_snake_case(),
                "lowercase" => variant_ident.to_string().to_ascii_lowercase(),
                _ => {
                    return Err(syn::Error::new(
                        variant_ident.span(),
                        "unsupported log_value rename_all",
                    ));
                }
            };
            Ok(quote! {
                Self::#variant_ident => crate::logging::event::LogFieldValue::String(#value.to_string())
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        impl crate::logging::event::LogValue for #enum_ident {
            fn into_log_field_value(self) -> crate::logging::event::LogFieldValue {
                match self {
                    #(#arms),*
                }
            }
        }
    })
}

#[derive(Clone)]
struct EnumLogEventMeta {
    producer: Option<String>,
    transport: Option<String>,
    parser: Option<String>,
}

#[derive(Clone)]
enum VariantLogEventMeta {
    Static {
        name: String,
        severity: String,
        result: String,
        message: String,
    },
    Computed {
        name: String,
    },
}

#[derive(Clone)]
struct FieldLogMeta {
    key: Option<String>,
    skip: bool,
    flatten_prefix: Option<String>,
}

fn parse_enum_log_event_meta(attrs: &[syn::Attribute]) -> syn::Result<EnumLogEventMeta> {
    let metas = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("log_event"))
        .map(parse_meta_list)
        .collect::<syn::Result<Vec<_>>>()?;

    let mut producer = None::<String>;
    let mut transport = None::<String>;
    let mut parser = None::<String>;

    for nested in metas.into_iter().flatten() {
        match nested {
            Meta::NameValue(name_value) if name_value.path.is_ident("producer") => {
                producer = Some(parse_string_value(&name_value)?);
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("transport") => {
                transport = Some(parse_string_value(&name_value)?);
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("parser") => {
                parser = Some(parse_string_value(&name_value)?);
            }
            Meta::NameValue(name_value)
                if name_value.path.is_ident("name")
                    || name_value.path.is_ident("severity")
                    || name_value.path.is_ident("result")
                    || name_value.path.is_ident("message")
                    || name_value.path.is_ident("meta") => {}
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "unsupported enum-level log_event key; accepted keys: producer, transport, parser",
                ));
            }
        }
    }

    Ok(EnumLogEventMeta {
        producer,
        transport,
        parser,
    })
}

fn parse_variant_log_event_meta(attrs: &[syn::Attribute]) -> syn::Result<VariantLogEventMeta> {
    let mut name = None::<String>;
    let mut severity = None::<String>;
    let mut result = None::<String>;
    let mut message = None::<String>;
    let mut meta = None::<String>;

    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("log_event"))
    {
        for nested in parse_meta_list(attr)? {
            match nested {
                Meta::NameValue(name_value) if name_value.path.is_ident("name") => {
                    name = Some(parse_string_value(&name_value)?);
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("severity") => {
                    severity = Some(parse_string_value(&name_value)?);
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("result") => {
                    result = Some(parse_string_value(&name_value)?);
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("message") => {
                    message = Some(parse_string_value(&name_value)?);
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("meta") => {
                    meta = Some(parse_string_value(&name_value)?);
                }
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "unsupported variant-level log_event key; accepted keys: name, severity, result, message, meta",
                    ));
                }
            }
        }
    }

    let name = name.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "variant #[log_event(...)] must include name",
        )
    })?;

    match meta.as_deref() {
        Some("computed") => {
            if severity.is_some() || result.is_some() || message.is_some() {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "meta = \"computed\" cannot be combined with severity, result, or message",
                ));
            }
            Ok(VariantLogEventMeta::Computed { name })
        }
        Some(other) => Err(syn::Error::new(
            Span::call_site(),
            format!("unsupported meta `{other}`; accepted values: computed"),
        )),
        None => Ok(VariantLogEventMeta::Static {
            name,
            severity: severity.ok_or_else(|| {
                syn::Error::new(
                    Span::call_site(),
                    "static log_event variants must include severity",
                )
            })?,
            result: result.ok_or_else(|| {
                syn::Error::new(
                    Span::call_site(),
                    "static log_event variants must include result",
                )
            })?,
            message: message.ok_or_else(|| {
                syn::Error::new(
                    Span::call_site(),
                    "static log_event variants must include message",
                )
            })?,
        }),
    }
}

fn parse_field_log_meta(field: &Field) -> syn::Result<FieldLogMeta> {
    let mut key = None::<String>;
    let mut skip = false;
    let mut flatten = false;
    let mut prefix = None::<String>;

    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("log"))
    {
        for nested in parse_meta_list(attr)? {
            match nested {
                Meta::Path(path) if path.is_ident("skip") => {
                    skip = true;
                }
                Meta::Path(path) if path.is_ident("flatten") => {
                    flatten = true;
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("key") => {
                    key = Some(parse_string_value(&name_value)?);
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("prefix") => {
                    prefix = Some(parse_string_value(&name_value)?);
                }
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "unsupported #[log(...)] key; accepted keys: key, skip, flatten, prefix",
                    ));
                }
            }
        }
    }

    if skip && (flatten || key.is_some() || prefix.is_some()) {
        return Err(syn::Error::new(
            field.span(),
            "#[log(skip)] cannot be combined with key, flatten, or prefix",
        ));
    }

    if flatten {
        let flatten_prefix = prefix.ok_or_else(|| {
            syn::Error::new(
                field.span(),
                "#[log(flatten, prefix = \"...\")] requires prefix",
            )
        })?;
        if key.is_some() {
            return Err(syn::Error::new(
                field.span(),
                "#[log(flatten, prefix = \"...\")] cannot be combined with key",
            ));
        }
        if !is_supported_flatten_type(&field.ty) {
            return Err(syn::Error::new(
                field.ty.span(),
                "unsupported flatten type; accepted types: BTreeMap<String, serde_json::Value>",
            ));
        }
        return Ok(FieldLogMeta {
            key: None,
            skip: false,
            flatten_prefix: Some(flatten_prefix),
        });
    }

    if prefix.is_some() {
        return Err(syn::Error::new(
            field.span(),
            "prefix is only valid together with flatten",
        ));
    }

    Ok(FieldLogMeta {
        key,
        skip,
        flatten_prefix: None,
    })
}

fn build_variant_arm(
    enum_ident: &Ident,
    enum_meta: &EnumLogEventMeta,
    variant: syn::Variant,
) -> syn::Result<proc_macro2::TokenStream> {
    let variant_ident = variant.ident.clone();
    let variant_meta = parse_variant_log_event_meta(&variant.attrs)?;
    let computed = matches!(variant_meta, VariantLogEventMeta::Computed { .. });
    if computed && !is_allowed_computed_variant(enum_ident, &variant_ident) {
        return Err(syn::Error::new(
            variant_ident.span(),
            format!(
                "unsupported computed metadata variant `{}::{}`; accepted variants: PgInfoLogEvent::SqlTransition, SubprocessLogEvent::Line, PostgresLineLogEvent::Json, PostgresLineLogEvent::Plain, PostgresLineLogEvent::Unparsed",
                enum_ident, variant_ident
            ),
        ));
    }

    let (pattern, field_inits) = build_field_tokens(&variant.fields)?;
    let event_name = LitStr::new(variant_name(&variant_meta), Span::call_site());
    let fields_expr = quote! {{
        let mut fields = ::std::vec::Vec::new();
        #(#field_inits)*
        fields
    }};

    match variant_meta {
        VariantLogEventMeta::Static {
            severity,
            result,
            message,
            ..
        } => {
            let source = static_source_tokens(enum_meta)?;
            let severity = severity_tokens(&severity)?;
            let result = result_tokens(&result)?;
            let message = LitStr::new(&message, Span::call_site());
            Ok(quote! {
                Self::#variant_ident #pattern => {
                    let fields = #fields_expr;
                    crate::logging::event::LogEventDto {
                        severity: #severity,
                        event_name: #event_name,
                        result: #result,
                        message: ::std::borrow::Cow::Borrowed(#message),
                        source: #source,
                        fields,
                    }
                }
            })
        }
        VariantLogEventMeta::Computed { .. } => {
            let meta_expr = computed_meta_tokens(enum_ident, &variant_ident)?;
            Ok(quote! {
                Self::#variant_ident #pattern => {
                    let meta = #meta_expr;
                    let fields = #fields_expr;
                    crate::logging::event::LogEventDto {
                        severity: meta.severity,
                        event_name: #event_name,
                        result: meta.result,
                        message: meta.message,
                        source: meta.source,
                        fields,
                    }
                }
            })
        }
    }
}

fn build_field_tokens(
    fields: &Fields,
) -> syn::Result<(proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>)> {
    match fields {
        Fields::Unit => Ok((quote! {}, Vec::new())),
        Fields::Named(fields) => {
            let bindings = fields
                .named
                .iter()
                .map(|field| {
                    field.ident.clone().ok_or_else(|| {
                        syn::Error::new(
                            field.span(),
                            "named LoggableEvent fields require identifiers",
                        )
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?;

            let pattern = quote! { { #(#bindings),* } };
            let field_inits = fields
                .named
                .iter()
                .map(build_field_init)
                .collect::<syn::Result<Vec<_>>>()?;
            Ok((pattern, field_inits))
        }
        Fields::Unnamed(_) => Err(syn::Error::new(
            fields.span(),
            "LoggableEvent only supports unit and struct variants",
        )),
    }
}

fn build_field_init(field: &Field) -> syn::Result<proc_macro2::TokenStream> {
    let ident = field.ident.clone().ok_or_else(|| {
        syn::Error::new(
            field.span(),
            "named LoggableEvent fields require identifiers",
        )
    })?;
    let meta = parse_field_log_meta(field)?;
    if meta.skip {
        return Ok(quote! {});
    }

    if let Some(prefix) = meta.flatten_prefix {
        let prefix = LitStr::new(&prefix, Span::call_site());
        if option_inner(&field.ty).is_some() {
            return Ok(quote! {
                if let Some(value) = #ident {
                    crate::logging::event::push_flattened_json_fields(&mut fields, #prefix, value);
                }
            });
        }
        return Ok(quote! {
            crate::logging::event::push_flattened_json_fields(&mut fields, #prefix, #ident);
        });
    }

    let key = meta.key.unwrap_or_else(|| ident.to_string());
    let key = LitStr::new(&key, Span::call_site());
    if option_inner(&field.ty).is_some() {
        return Ok(quote! {
            if let Some(value) = #ident {
                fields.push(crate::logging::event::LogField::new(
                    #key,
                    crate::logging::event::LogValue::into_log_field_value(value),
                ));
            }
        });
    }

    Ok(quote! {
        fields.push(crate::logging::event::LogField::new(
            #key,
            crate::logging::event::LogValue::into_log_field_value(#ident),
        ));
    })
}

fn computed_meta_tokens(
    enum_ident: &Ident,
    variant_ident: &Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let enum_name = enum_ident.to_string();
    let variant_name = variant_ident.to_string();

    match (enum_name.as_str(), variant_name.as_str()) {
        ("PgInfoLogEvent", "SqlTransition") => Ok(quote! {{
            let became_unreachable =
                previous.as_ref() == Some(&crate::pginfo::state::SqlStatus::Healthy)
                    && next == crate::pginfo::state::SqlStatus::Unreachable;
            let recovered =
                previous.as_ref() == Some(&crate::pginfo::state::SqlStatus::Unreachable)
                    && next == crate::pginfo::state::SqlStatus::Healthy;
            let severity = if became_unreachable {
                crate::logging::LogSeverity::Warn
            } else if recovered {
                crate::logging::LogSeverity::Info
            } else {
                crate::logging::LogSeverity::Debug
            };
            let result = if became_unreachable {
                crate::logging::LogEventResult::Failed
            } else if recovered {
                crate::logging::LogEventResult::Recovered
            } else {
                crate::logging::LogEventResult::Ok
            };
            crate::logging::event::LogComputedMeta {
                severity,
                result,
                message: ::std::borrow::Cow::Borrowed("pginfo sql status transition"),
                source: crate::logging::LogSource {
                    producer: crate::logging::LogProducer::App,
                    transport: crate::logging::LogTransport::Internal,
                    parser: crate::logging::LogParser::App,
                },
            }
        }}),
        ("SubprocessLogEvent", "Line") => Ok(quote! {{
            let severity = match stream {
                crate::process::log_event::CapturedStream::Stdout => crate::logging::LogSeverity::Info,
                crate::process::log_event::CapturedStream::Stderr => crate::logging::LogSeverity::Warn,
            };
            let transport = match stream {
                crate::process::log_event::CapturedStream::Stdout => crate::logging::LogTransport::ChildStdout,
                crate::process::log_event::CapturedStream::Stderr => crate::logging::LogTransport::ChildStderr,
            };
            crate::logging::event::LogComputedMeta {
                severity,
                result: crate::logging::LogEventResult::Ok,
                message: ::std::borrow::Cow::Owned(line.clone()),
                source: crate::logging::LogSource {
                    producer: crate::logging::LogProducer::PgTool,
                    transport,
                    parser: crate::logging::LogParser::Raw,
                },
            }
        }}),
        ("PostgresLineLogEvent", "Json") => Ok(quote! {{
            crate::logging::event::LogComputedMeta {
                severity,
                result: crate::logging::LogEventResult::Ok,
                message: ::std::borrow::Cow::Owned(message.clone()),
                source: crate::logging::LogSource {
                    producer: source.producer,
                    transport: source.transport,
                    parser: crate::logging::LogParser::PostgresJson,
                },
            }
        }}),
        ("PostgresLineLogEvent", "Plain") => Ok(quote! {{
            crate::logging::event::LogComputedMeta {
                severity,
                result: crate::logging::LogEventResult::Ok,
                message: ::std::borrow::Cow::Owned(message.clone()),
                source: crate::logging::LogSource {
                    producer: source.producer,
                    transport: source.transport,
                    parser: crate::logging::LogParser::PostgresPlain,
                },
            }
        }}),
        ("PostgresLineLogEvent", "Unparsed") => Ok(quote! {{
            crate::logging::event::LogComputedMeta {
                severity: crate::logging::LogSeverity::Info,
                result: crate::logging::LogEventResult::Ok,
                message: ::std::borrow::Cow::Owned(raw_line.clone()),
                source: crate::logging::LogSource {
                    producer: source.producer,
                    transport: source.transport,
                    parser: crate::logging::LogParser::Raw,
                },
            }
        }}),
        _ => Err(syn::Error::new(
            variant_ident.span(),
            "unsupported computed metadata variant",
        )),
    }
}

fn static_source_tokens(enum_meta: &EnumLogEventMeta) -> syn::Result<proc_macro2::TokenStream> {
    let producer = enum_meta.producer.as_deref().ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "static LoggableEvent variants require enum-level producer",
        )
    })?;
    let transport = enum_meta.transport.as_deref().ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "static LoggableEvent variants require enum-level transport",
        )
    })?;
    let parser = enum_meta.parser.as_deref().ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "static LoggableEvent variants require enum-level parser",
        )
    })?;

    let producer = producer_tokens(producer)?;
    let transport = transport_tokens(transport)?;
    let parser = parser_tokens(parser)?;
    Ok(quote! {
        crate::logging::LogSource {
            producer: #producer,
            transport: #transport,
            parser: #parser,
        }
    })
}

fn severity_tokens(value: &str) -> syn::Result<proc_macro2::TokenStream> {
    match value {
        "trace" => Ok(quote! { crate::logging::LogSeverity::Trace }),
        "debug" => Ok(quote! { crate::logging::LogSeverity::Debug }),
        "info" => Ok(quote! { crate::logging::LogSeverity::Info }),
        "warn" => Ok(quote! { crate::logging::LogSeverity::Warn }),
        "error" => Ok(quote! { crate::logging::LogSeverity::Error }),
        "fatal" => Ok(quote! { crate::logging::LogSeverity::Fatal }),
        other => Err(syn::Error::new(
            Span::call_site(),
            format!(
                "unsupported severity `{other}`; accepted values: trace, debug, info, warn, error, fatal"
            ),
        )),
    }
}

fn result_tokens(value: &str) -> syn::Result<proc_macro2::TokenStream> {
    match value {
        "ok" => Ok(quote! { crate::logging::LogEventResult::Ok }),
        "failed" => Ok(quote! { crate::logging::LogEventResult::Failed }),
        "recovered" => Ok(quote! { crate::logging::LogEventResult::Recovered }),
        "timeout" => Ok(quote! { crate::logging::LogEventResult::Timeout }),
        other => Err(syn::Error::new(
            Span::call_site(),
            format!(
                "unsupported result `{other}`; accepted values: ok, failed, recovered, timeout"
            ),
        )),
    }
}

fn producer_tokens(value: &str) -> syn::Result<proc_macro2::TokenStream> {
    match value {
        "app" => Ok(quote! { crate::logging::LogProducer::App }),
        "postgres" => Ok(quote! { crate::logging::LogProducer::Postgres }),
        "pg_tool" => Ok(quote! { crate::logging::LogProducer::PgTool }),
        other => Err(syn::Error::new(
            Span::call_site(),
            format!("unsupported producer `{other}`; accepted values: app, postgres, pg_tool"),
        )),
    }
}

fn transport_tokens(value: &str) -> syn::Result<proc_macro2::TokenStream> {
    match value {
        "internal" => Ok(quote! { crate::logging::LogTransport::Internal }),
        "file_tail" => Ok(quote! { crate::logging::LogTransport::FileTail }),
        "child_stdout" => Ok(quote! { crate::logging::LogTransport::ChildStdout }),
        "child_stderr" => Ok(quote! { crate::logging::LogTransport::ChildStderr }),
        other => Err(syn::Error::new(
            Span::call_site(),
            format!(
                "unsupported transport `{other}`; accepted values: internal, file_tail, child_stdout, child_stderr"
            ),
        )),
    }
}

fn parser_tokens(value: &str) -> syn::Result<proc_macro2::TokenStream> {
    match value {
        "app" => Ok(quote! { crate::logging::LogParser::App }),
        "postgres_json" => Ok(quote! { crate::logging::LogParser::PostgresJson }),
        "postgres_plain" => Ok(quote! { crate::logging::LogParser::PostgresPlain }),
        "raw" => Ok(quote! { crate::logging::LogParser::Raw }),
        other => Err(syn::Error::new(
            Span::call_site(),
            format!(
                "unsupported parser `{other}`; accepted values: app, postgres_json, postgres_plain, raw"
            ),
        )),
    }
}

fn parse_log_value_meta(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    let mut rename_all = None::<String>;
    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("log_value"))
    {
        for nested in parse_meta_list(attr)? {
            match nested {
                Meta::NameValue(name_value) if name_value.path.is_ident("rename_all") => {
                    rename_all = Some(parse_string_value(&name_value)?);
                }
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "unsupported log_value key; accepted keys: rename_all",
                    ));
                }
            }
        }
    }
    Ok(rename_all)
}

fn parse_meta_list(attr: &syn::Attribute) -> syn::Result<Vec<Meta>> {
    let parser = Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    match &attr.meta {
        Meta::List(list) => parser
            .parse2(list.tokens.clone())
            .map(|items| items.into_iter().collect()),
        _ => Err(syn::Error::new(
            attr.span(),
            "attribute must use list syntax",
        )),
    }
}

fn parse_string_value(name_value: &MetaNameValue) -> syn::Result<String> {
    match &name_value.value {
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Ok(value.value()),
        _ => Err(syn::Error::new(
            name_value.value.span(),
            "attribute value must be a string literal",
        )),
    }
}

fn option_inner(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(type_path) => {
            let segment = type_path.path.segments.last()?;
            if segment.ident != "Option" {
                return None;
            }
            match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => {
                    arguments.args.first().and_then(|arg| match arg {
                        GenericArgument::Type(inner) => Some(inner),
                        _ => None,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn is_supported_flatten_type(ty: &Type) -> bool {
    if let Some(inner) = option_inner(ty) {
        return is_supported_flatten_type(inner);
    }
    let Type::Path(type_path) = ty else {
        return false;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };
    if segment.ident != "BTreeMap" {
        return false;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    let mut args = arguments.args.iter();
    let Some(GenericArgument::Type(Type::Path(key_type))) = args.next() else {
        return false;
    };
    let Some(GenericArgument::Type(Type::Path(value_type))) = args.next() else {
        return false;
    };
    let key_ok = key_type
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "String");
    let value_ok = value_type
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Value");
    key_ok && value_ok && args.next().is_none()
}

fn is_allowed_computed_variant(enum_ident: &Ident, variant_ident: &Ident) -> bool {
    matches!(
        (
            enum_ident.to_string().as_str(),
            variant_ident.to_string().as_str()
        ),
        ("PgInfoLogEvent", "SqlTransition")
            | ("SubprocessLogEvent", "Line")
            | ("PostgresLineLogEvent", "Json")
            | ("PostgresLineLogEvent", "Plain")
            | ("PostgresLineLogEvent", "Unparsed")
    )
}

fn variant_name(meta: &VariantLogEventMeta) -> &str {
    match meta {
        VariantLogEventMeta::Static { name, .. } | VariantLogEventMeta::Computed { name } => name,
    }
}
