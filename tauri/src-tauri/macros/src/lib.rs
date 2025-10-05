use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Type, TypePath, Path, PathSegment};

#[proc_macro_derive(FromSqliteRow, attributes(sqlite))]
pub fn derive_from_sqlite_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("FromSqliteRow can only be used with structs"),
    };

    let field_assignments: Vec<_> = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let field_name_str = field_name.to_string();

        // 检查 Option 类型
        let is_option = match field_type {
            Type::Path(type_path) => {
                type_path.path.segments.first().map(|seg| seg.ident == "Option").unwrap_or(false)
            }
            _ => false,
        };

        // 检查是否有 #[sqlite(from_str)]
        let mut is_from_str = false;
        for attr in &field.attrs {
            if attr.path().is_ident("sqlite") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("from_str") {
                        is_from_str = true;
                    }
                    Ok(())
                });
            }
        }

        if is_from_str {
            // 使用 row.get 直接获取实现了 FromSql trait 的枚举类型
            quote! {
                #field_name: row.get(#field_name_str)?,
            }
        } else if is_option {
            quote! {
                #field_name: row.get(#field_name_str)?,
            }
        } else {
            quote! {
                #field_name: row.get(#field_name_str)?,
            }
        }
    }).collect();

    let expanded = quote! {
        impl std::convert::TryFrom<&rusqlite::Row<'_>> for #name {
            type Error = rusqlite::Error;
            fn try_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
                Ok(Self {
                    #(#field_assignments)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
