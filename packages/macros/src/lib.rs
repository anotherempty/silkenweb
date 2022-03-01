use std::{env, path::PathBuf};

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::{
    bracketed,
    parse::{Lookahead1, Parse, ParseStream, Peek},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Colon, Comma, CustomToken},
    Ident, LitStr, Visibility,
};

mod parser;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(path);
    custom_keyword!(visibility);
    custom_keyword!(prefix);
    custom_keyword!(include_prefixes);
    custom_keyword!(exclude_prefixes);
}

/// Define `&str` constants for each class in a SASS file.
///
/// For a CSS class called `my-css-class`, a constant called `MY_CSS_CLASS` will
/// be defined.
///
/// The macro takes two forms. Firstly it can take a single string literal which
/// is the path to the CSS/SCSS/SASS file. The path is relative to the
/// `$CARGO_MANIFEST_DIR` environment variable.
///
/// Alternatively, named parameters can be specified:
/// - `path` (mandatory) is the path to the CSS /SCSS/SASS file.
/// - `visibility` (optional) is any visibility modifier, and controls the
///   visibility of class constants.
/// - `prefix` (optional) specifies that only classes starting with `prefix`
///   should be included. Their Rust names will have the prefix stripped.
/// - `include_prefixes` (optional) specifies a list of prefixes to include,
///   without stripping the prefix. Rust constants will only be defined for
///   classes starting with one or more of these prefixes.
/// - `exclude_prefixes` (optional) specifies a list of prefixes to exclude. No
///   Rust constants will be defined for a class starting with any of these
///   prefixes. `exclude_prefixes` takes precedence over `include_prefixes`.
///
/// # Examples
///
/// Define private constants for all CSS classes:
///
///  ```
/// # use silkenweb_macros::css_classes;
/// css_classes!("my-sass-file.scss");
/// assert_eq!(MY_CLASS, "my-class");
/// ```
/// 
/// Include classes starting with `border-`, except classes starting with `border-excluded-`:
/// ```
/// mod border {
///     # use silkenweb_macros::css_classes;
///     css_classes!(
///         visibility: pub,
///         path: "my-sass-file.scss",
///         prefix:"border-",
///         exclude_prefixes: ["border-excluded-"]
///     );
/// }
///
/// assert_eq!(border::SMALL, "border-small");
/// ```
/// 
/// This won't compile because `exclude_prefixes` takes precedence over
/// `include_prefixes`:
/// ```compile_fail
///     # use silkenweb_macros::css_classes;
///     css_classes!(
///         path: "my-sass-file.scss",
///         include_prefixes: ["border-"]
///         exclude_prefixes: ["border-excluded-"]
///     );
///
///     assert_eq!(BORDER_EXCLUDED_HUGE, "border-excluded-huge");
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn css_classes(input: TokenStream) -> TokenStream {
    let Input {
        visibility,
        path,
        prefix,
        include_prefixes,
        exclude_prefixes,
    } = parse_macro_input!(input);

    let root_dir = env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| abort_call_site!("Unable to read {}", CARGO_MANIFEST_DIR));
    let path = PathBuf::from(root_dir)
        .join(path)
        .into_os_string()
        .into_string()
        .expect("Expected path to be convertible to string");

    let classes = parser::class_names(&path)
        .unwrap_or_else(|e| abort_call_site!("'{}': {}", path, e.to_string()))
        .filter(|class| {
            let include = if let Some(include_prefixes) = include_prefixes.as_ref() {
                any_prefix_matches(class, include_prefixes)
            } else {
                true
            };

            let exclude = any_prefix_matches(class, &exclude_prefixes);

            include && !exclude
        });

    if let Some(prefix) = prefix {
        code_gen(
            visibility,
            &path,
            classes.filter_map(|class| {
                let class_ident = class.strip_prefix(&prefix).map(str::to_string);
                class_ident.map(|class_ident| {
                    println!("{}, {}", class_ident, class);
                    (class_ident, class)
                })
            }),
        )
    } else {
        code_gen(
            visibility,
            &path,
            classes.map(|class| (class.clone(), class)),
        )
    }
}

struct Input {
    path: String,
    visibility: Option<Visibility>,
    prefix: Option<String>,
    include_prefixes: Option<Vec<String>>,
    exclude_prefixes: Vec<String>,
}

impl Input {
    fn parameter<Keyword, KeywordToken, T>(
        keyword: Keyword,
        lookahead: &Lookahead1,
        input: ParseStream,
        exists: bool,
    ) -> syn::Result<bool>
    where
        Keyword: Peek + FnOnce(T) -> KeywordToken,
        KeywordToken: Parse + CustomToken,
    {
        Ok(if lookahead.peek(keyword) {
            if exists {
                abort!(
                    input.span(),
                    "{} is defined multiple times",
                    KeywordToken::display()
                );
            }

            input.parse::<KeywordToken>()?;
            input.parse::<Colon>()?;

            true
        } else {
            false
        })
    }

    fn parse_prefix_list(input: &syn::parse::ParseBuffer) -> Result<Vec<String>, syn::Error> {
        let list;
        bracketed!(list in input);
        Ok(Punctuated::<LitStr, Comma>::parse_terminated(&list)?
            .iter()
            .map(|prefix| prefix.value())
            .collect())
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            return Ok(Self {
                path: input.parse::<LitStr>()?.value(),
                visibility: None,
                prefix: None,
                include_prefixes: None,
                exclude_prefixes: Vec::new(),
            });
        }

        let mut path = None;
        let mut visibility = None;
        let mut prefix = None;
        let mut include_prefixes = None;
        let mut exclude_prefixes = Vec::new();
        let mut trailing_comma = true;

        while !input.is_empty() {
            if !trailing_comma {
                abort!(input.span(), "Expected ','");
            }

            let lookahead = input.lookahead1();

            if Self::parameter(kw::path, &lookahead, input, path.is_some())? {
                path = Some(input.parse::<LitStr>()?.value());
            } else if Self::parameter(kw::visibility, &lookahead, input, visibility.is_some())? {
                visibility = Some(input.parse()?);
            } else if Self::parameter(kw::prefix, &lookahead, input, prefix.is_some())? {
                prefix = Some(input.parse::<LitStr>()?.value());
            } else if Self::parameter(
                kw::include_prefixes,
                &lookahead,
                input,
                include_prefixes.is_some(),
            )? {
                include_prefixes = Some(Self::parse_prefix_list(input)?);
            } else if Self::parameter(
                kw::exclude_prefixes,
                &lookahead,
                input,
                !exclude_prefixes.is_empty(),
            )? {
                exclude_prefixes = Self::parse_prefix_list(input)?;
            } else {
                return Err(lookahead.error());
            }

            trailing_comma = input.peek(Comma);

            if trailing_comma {
                input.parse::<Comma>()?;
            }
        }

        if let Some(path) = path {
            Ok(Self {
                visibility,
                path,
                prefix,
                include_prefixes,
                exclude_prefixes,
            })
        } else {
            abort_call_site!("Missing 'path' parameter");
        }
    }
}

fn any_prefix_matches(x: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| x.starts_with(prefix))
}

fn code_gen(
    visibility: Option<Visibility>,
    path: &str,
    classes: impl Iterator<Item = (String, String)>,
) -> TokenStream {
    let classes = classes.map(|(class_ident, class_name)| {
        if !class_ident.starts_with(char::is_alphabetic) {
            abort_call_site!(
                "Identifier '{}' doesn't start with an alphabetic character",
                class_ident
            );
        }

        let class_ident = Ident::new(
            &class_ident
                .replace(|c: char| !c.is_alphanumeric(), "_")
                .to_uppercase(),
            Span::call_site(),
        );
        quote!(#visibility const #class_ident: &str = #class_name;)
    });

    quote!(
        const _: &[u8] = ::std::include_bytes!(#path);
        #(#classes)*
    )
    .into()
}

const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";
