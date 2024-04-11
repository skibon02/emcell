mod defs;

use proc_macro::{TokenStream};
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::{ExprStruct, parse_macro_input, Token};
use syn::parse::{Parse, Parser, ParseStream};
use syn::spanned::Spanned;


#[proc_macro]
pub fn define_header(item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ExprStruct);
    let ident = input.path;
    let ident_str = ident.to_token_stream().to_string().trim_matches('"').to_string();

    let fields = input.fields;

    let link_section = ".emcell.cur_header";
    let static_ident = format_ident!("_emcell_{}_internal", ident_str);


    let output: proc_macro2::TokenStream = {
        quote!(
            #[no_mangle]
            #[export_name = "Reset"]
            pub fn reset() -> ! {
                loop {}
            }

            #[no_mangle]
            #[link_section = #link_section]
            pub static #static_ident : #ident = #ident {
                init_memory: unsafe { emcell::init_memory},
                signature: 0xdeadbeef,
                #fields
            };
        )
    };

    proc_macro::TokenStream::from(output)
}



#[proc_macro]
pub fn define_primary_header(item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ExprStruct);
    let ident = input.path;
    let ident_str = ident.to_token_stream().to_string().trim_matches('"').to_string();

    let fields = input.fields;

    let link_section = ".emcell.cur_header";
    let static_ident = format_ident!("_emcell_{}_internal", ident_str);

    let output: proc_macro2::TokenStream = {
        quote!(
            #[no_mangle]
            #[link_section = #link_section]
            pub static #static_ident : #ident = #ident {
                #fields
            };
        )
    };

    proc_macro::TokenStream::from(output)
}

struct ExternHeader {
    name: Ident,
    typez: Ident,
}

impl Parse for ExternHeader {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let right: Ident = input.parse()?;

        Ok(ExternHeader { name: left, typez: right })
    }
}

#[proc_macro]
pub fn extern_header(item: TokenStream) -> TokenStream {
    let ExternHeader { name, typez } = parse_macro_input!(item as ExternHeader);

    let internal_ident = format_ident!("_emcell_{}_internal", typez);

    let output: proc_macro2::TokenStream = {
        quote!(
            extern crate emcell;
            extern "Rust" {
                pub static #internal_ident: #typez;
            }

            pub type #name = emcell::CellWrapper<#typez>;


            pub unsafe trait CellWrapperTrait {
                type CellWrapperType;
                fn new() -> Option<Self::CellWrapperType>;
                fn new_uninit() -> Self::CellWrapperType;
            }

            unsafe impl CellWrapperTrait for #name {
                type CellWrapperType = #name;

                fn new() -> Option<Self> {
                    let cell = unsafe { & #internal_ident };
                    unsafe { emcell::CellWrapper::_new_init(cell)}
                }

                fn new_uninit() -> Self {
                    let cell = unsafe { & #internal_ident };
                    unsafe { emcell::CellWrapper::_new_uninit(cell) }
                }
            }
        )
    };

    // let tokens: Vec<_> = input.into_iter().collect();
    // println!("tokens: {:?}", tokens);

    proc_macro::TokenStream::from(output)
}


// Macro, defined in defs.rs for user crate with cells definitions
#[proc_macro_attribute]
pub fn cell(cell_attr: TokenStream, item: TokenStream) -> TokenStream {
    defs::cell(cell_attr, item)
}

#[proc_macro_attribute]
pub fn ram_region(attr: TokenStream, item: TokenStream) -> TokenStream {
    defs::ram_region(attr, item)
}

#[proc_macro_attribute]
pub fn flash_region(attr: TokenStream, item: TokenStream) -> TokenStream {
    defs::flash_region(attr, item)
}

#[proc_macro]
pub fn device(item: TokenStream) -> TokenStream {
    defs::device(item)
}

#[proc_macro]
pub fn emcell_configuration(input: TokenStream) -> TokenStream {
    defs::emcell_configuration(input)
}