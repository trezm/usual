// #![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use proc_macro2::{
    Ident, Literal, Span as Span2, TokenStream as TokenStream2, TokenTree as TokenTree2,
};
use quote::quote;
use regex::{Match, Regex};
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn query(items: TokenStream) -> TokenStream {
    let items = proc_macro2::TokenStream::from(items);
    let mut item_iter = items.clone().into_iter();

    let text = match item_iter.next() {
        Some(TokenTree2::Literal(literal)) => {
            let stream = TokenStream2::from(TokenTree2::Literal(literal)).into();
            parse_macro_input!(stream as LitStr).value()
        }
        _ => panic!("The first argument of `query!` must be a string literal."),
    };

    let re = Regex::new(r"\{([^\}:\s]+)(?:::([\w,]+))?\s*(?:as (\w+))?\}").unwrap();

    let mut matches = re.find_iter(&text).into_iter().collect::<Vec<Match>>();
    matches.reverse();
    let mut output_string = text.clone();
    let mut value_injections = vec![];

    for m in matches {
        for cap in re.captures_iter(m.as_str()) {
            let mut cap_iter = cap.iter();
            let _full = cap_iter.next().unwrap().unwrap().as_str();

            let model_name = cap_iter.next().unwrap().unwrap().as_str();
            let field_names = match cap_iter.next() {
                Some(Some(inner_match)) => {
                    let value = inner_match.as_str();

                    value.split(",").collect::<Vec<&str>>()
                }
                _ => vec![],
            };

            let table_name = match cap_iter.next() {
                Some(Some(inner_match)) => Some(inner_match.as_str()),
                _ => None,
            };

            if field_names.len() > 0 {
                let mut fields = vec![];

                &field_names.into_iter().for_each(|f| {
                    let model_ident = Ident::new(model_name, Span2::call_site());
                    let column_literal = Literal::string(f);

                    match table_name {
                        Some(table_name) => {
                            let table_name = Literal::string(table_name);
                            fields.push(quote! { <#model_ident>::column_with_prefix_and_table(#column_literal, Some(<#model_ident>::prefix()), Some(#table_name)) });
                        }
                        None => {
                            fields.push(quote! { <#model_ident>::column_with_prefix_and_table(#column_literal, Some(<#model_ident>::prefix()), None) });
                        }
                    }
                });

                // Reverse the fields order since we're going back to front for the matches -- they'll get switched when we reverse the whole array.
                fields.reverse();
                value_injections.append(&mut fields);

                output_string.replace_range(
                    m.range(),
                    &std::iter::repeat("{}")
                        .take(value_injections.len())
                        .collect::<Vec<&str>>()
                        .join(", "),
                );
            } else {
                let ident = Ident::new(model_name, Span2::call_site());
                let initial_injection_count = value_injections.len();

                match table_name {
                    Some(table_name) => {
                        let table_name = Literal::string(table_name);
                        value_injections.push(quote! { <#ident>::columns_with_table(#table_name) });
                    }
                    None => {
                        value_injections.push(quote! { <#ident>::columns() });
                    }
                }

                output_string.replace_range(
                    m.range(),
                    &std::iter::repeat("{}")
                        .take(value_injections.len() - initial_injection_count)
                        .collect::<Vec<&str>>()
                        .join(", "),
                );
            }
        }
    }

    value_injections.reverse();

    let gen = quote! {
        format!(#output_string, #( #value_injections,)*)
    };

    // proc_macro::Span::call_site()
    //     .note("Thruster code output")
    //     .note(gen.to_string())
    //     .emit();

    gen.into()
}

struct Field {
    name: String,
    ty: String,
}

#[proc_macro]
pub fn partial(items: TokenStream) -> TokenStream {
    let items = proc_macro2::TokenStream::from(items);
    let mut item_iter = items.clone().into_iter();

    let model = match item_iter.next() {
        Some(TokenTree2::Ident(ident)) => ident,
        _ => panic!("The first argument of `query!` must be a string literal."),
    };
    let model_name = model.to_string();
    let partial_ident = Ident::new(&format!("Partial{}", model.to_string()), Span2::call_site());
    let partial_ident_name = partial_ident.to_string();

    // Accept more arguments -- really not sure if we need this.
    let mut fields = vec![];

    item_iter.next(); // skip punctuation

    while let Some(ident) = item_iter.next() {
        let field_name = match ident {
            TokenTree2::Ident(i) => i,
            _ => panic!("All arguments after the first must be identifiers."),
        };

        match item_iter.next() {
            Some(TokenTree2::Ident(i)) => if i.to_string() != "as" { panic!("A field identifier must be followed by `as Type`, for example, `content as String`.") },
            _ => panic!("A field identifier must be followed by `as Type`, for example, `content as String`."),
        }; // skip punctuation

        let ty = match item_iter.next() {
            Some(TokenTree2::Ident(i)) => {
                i
            },
            _ => panic!("A field identifier must be followed by `as Type`, for example, `content as String`."),
        };

        fields.push(Field {
            name: field_name.to_string(),
            ty: ty.to_string(),
        });

        item_iter.next(); // skip punctuation
    }

    let field_declarations = fields
        .iter()
        .map(|f| {
            let field_name = Ident::new(&f.name, Span2::call_site());
            let field_type = Ident::new(&f.ty, Span2::call_site());
            quote! {
                pub #field_name: #field_type
            }
        })
        .collect::<Vec<_>>();

    let field_initializers = fields
        .iter()
        .map(|f| {
            let field_name = Ident::new(&f.name, Span2::call_site());
            let field_key = field_name.to_string();
            quote! {
                #field_name: row.try_get(format!("{}{}", Self::prefix(), #field_key).as_str())
                .expect(&format!("You messed up while trying to get {} ({}{}) from {}", #field_key, Self::prefix(), #field_key, #partial_ident_name))
            }
        })
        .collect::<Vec<_>>();

    let field_keys = fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>();

    let gen = quote! {
        |r| {
            #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            struct #partial_ident {
                #(
                    #field_declarations
                ),*
            };

            impl Model for #partial_ident {
                fn prefix() -> &'static str {
                    concat!(#model_name, "__")
                }

                fn from_row_starting_index(_index: usize, row: &impl TryGetRow) -> Self {
                    #partial_ident {
                        #( #field_initializers ),*
                    }
                }

                fn columns_list() -> Vec<&'static str> {
                    vec![#( #field_keys ),*]
                }
            }

            #partial_ident::from_row_starting_index(0, r)
        }
    };

    // proc_macro::Span::call_site()
    //     .note("Thruster code output")
    //     .note(gen.to_string())
    //     .emit();

    gen.into()
}
