use proc_macro::{TokenStream};
use std::collections::BTreeMap;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, ToTokens};
use syn::{Data, DataStruct, DeriveInput, ExprStruct, Field, Fields, ItemStruct, LitInt, Meta, parse_macro_input, Token};
use syn::__private::TokenStream2;
use syn::parse::{Parse, Parser, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;

struct EmcellDef {
    strukt: ItemStruct,
    is_primary: bool,

    ram_region: RamRegionAttrib,
    flash_region: FlashRegionAttrib,
}

impl ToTokens for EmcellDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // generate fields for the struct
        let fields = self.strukt.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! {
                #ident: core::option::Option<#ty>
            }
        });

        let name = self.strukt.ident.to_string();
        let ram_region_start = self.ram_region.start;
        let ram_region_end = self.ram_region.end;
        let flash_region_start = self.flash_region.start;
        let flash_region_end = self.flash_region.end;

        let cell_type = if self.is_primary {
            quote! { emcell::CellType::Primary }
        } else {
            quote! { emcell::CellType::NonPrimary }
        };

        tokens.extend(quote! {
            emcell::CellDefMeta {
                name: #name,
                cell_type: #cell_type,
                ram_range_start: #ram_region_start,
                ram_range_end: #ram_region_end,
                flash_range_start: #flash_region_start,
                flash_range_end: #flash_region_end,
            }
        });
    }
}
struct EmcellDefsList(Vec<EmcellDef>);

impl Parse for EmcellDefsList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        let mut primary_count = 0;
        while !input.is_empty() {
            let strukt: ItemStruct = input.parse()?;


            let mut is_primary = None;
            let mut ram_region = None;
            let mut flash_region = None;

            for attr in &strukt.attrs {
                let meta = &attr.meta;
                let name = meta.path();

                match name {
                    _ if name.is_ident("ram_region") => {
                        let meta = meta.require_list()?;
                        ram_region = Some(syn::parse2::<RamRegionAttrib>(meta.tokens.clone())?);
                    }
                    _ if name.is_ident("flash_region") => {
                        let meta = meta.require_list()?;
                        flash_region = Some(syn::parse2::<FlashRegionAttrib>(meta.tokens.clone())?);
                    }
                    _ if name.is_ident("cell") => {
                        match meta {
                            Meta::Path(_) => {
                                is_primary = Some(false);
                            } // #[cell]
                            Meta::List(inner) => {
                                let attr_params = syn::parse2::<CellAttribParams>(inner.tokens.clone())?;
                                if attr_params.is_primary {
                                    is_primary = Some(true);
                                    primary_count += 1;
                                }
                            }
                            _ => {
                                return Err(syn::Error::new(attr.span(), "Expected either #[cell] or #[cell(primary)]"));
                            }
                        }
                    }
                    _ => {}
                }
            }

            let Some(ram_region) = ram_region else {
                return Err(syn::Error::new(strukt.span(), "Attribute #[ram_region(start, end)] required for each struct definition"));
            };
            let Some(flash_region) = flash_region else {
                return Err(syn::Error::new(strukt.span(), "Attribute #[flash_region(start, end)] required for each struct definition"));
            };
            let Some(is_primary) = is_primary else {
                return Err(syn::Error::new(strukt.span(), "Required attribute #[cell] or #[cell(primary)] missing for struct definition"));
            };

            items.push(EmcellDef {
                strukt,
                is_primary,
                ram_region,
                flash_region,
            });
        }

        if primary_count > 1 {
            return Err(syn::Error::new(Span::call_site(), "Only one primary cell is allowed"));
        }
        if primary_count == 0 {
            return Err(syn::Error::new(Span::call_site(), "No primary cell found. At least one cell must be marked as #[cell(primary)]"));
        }
        Ok(EmcellDefsList(items))
    }
}

#[proc_macro]
pub fn emcell_configuration(input: TokenStream) -> TokenStream {
    let output = input.clone();
    let emcell_defs = parse_macro_input!(input as EmcellDefsList).0;

    let mut cell_names = Vec::new();
    let mut non_primary_cell_idents = Vec::new();
    let mut primary_cell = None;
    for cell in &emcell_defs {
        let cell_name = cell.strukt.ident.to_string();
        cell_names.push(cell_name);

        if cell.is_primary {
            primary_cell = Some(cell.strukt.ident.clone());
        }
        else {
            non_primary_cell_idents.push(cell.strukt.ident.clone());
        }
    }
    let primary_cell = primary_cell.unwrap();

    let cell_defs_count = cell_names.len();
    let cell_defs_array = quote! {
        pub const CELL_NAMES: [&'static str; #cell_defs_count] = [#(#cell_names),*];
    };

    let output = proc_macro2::TokenStream::from(output);
    let output = quote! {
        #output
        #cell_defs_array

        pub type PrimaryCell = #primary_cell;

        #(unsafe impl emcell::Cell for #non_primary_cell_idents {
            fn check_signature(&self) -> bool {
                let sig_valid = self.signature == 0xdeadbeef;

                if sig_valid {
                    unsafe {(self.init_memory)()}
                }
                sig_valid
            }
        })*

        unsafe impl emcell::Cell for #primary_cell {
            fn check_signature(&self) -> bool {
                true
            }
        }

        pub static META: emcell::CellDefsMeta::<#cell_defs_count> = emcell::CellDefsMeta {
            cell_defs: [#(#emcell_defs),*],
        };

        pub fn meta_for_cell(cell_name: &str) -> Option<&'static emcell::CellDefMeta> {
            META.cell_defs.iter().find(|cell| cell.name == cell_name)
        }
    };

    TokenStream::from(output)
}

#[proc_macro]
pub fn define_abi(item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ExprStruct);
    let ident = input.path;
    let ident_str = ident.to_token_stream().to_string().trim_matches('"').to_string();

    let fields = input.fields;

    let link_section = String::from(".emcell.") +
        &ident_str.to_uppercase();

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
pub fn define_primary_abi(item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ExprStruct);
    let ident = input.path;
    let ident_str = ident.to_token_stream().to_string().trim_matches('"').to_string();

    let fields = input.fields;

    let link_section = String::from(".emcell.") +
        &ident_str.to_uppercase();

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

struct MyMapping {
    name: Ident,
    typez: Ident,
}

impl Parse for MyMapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let right: Ident = input.parse()?;

        Ok(MyMapping { name: left, typez: right })
    }
}

#[proc_macro]
pub fn extern_abi(item: TokenStream) -> TokenStream {
    let MyMapping { name, typez } = parse_macro_input!(item as MyMapping);

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

// params for #[cell] attribute
struct CellAttribParams {
    is_primary: bool,
}

impl Parse for CellAttribParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // check if input is empty
        if input.is_empty() {
            return Ok(CellAttribParams {
                is_primary: false
            });
        }
        let name: Ident = input.parse()?;

        let is_primary = match name.to_string().as_str() {
            "primary" => true,
            "" => false,
            _ => return Err(syn::Error::new(name.span(), "Invalid attribute! Expected #[cell(primary)] or #[cell]")),
        };

        Ok(CellAttribParams {
            is_primary
        })
    }
}
fn parse_integer_lit(input: &LitInt) -> syn::Result<usize> {
    match input.suffix() {
        "" => input.base10_parse::<usize>(),
        "0x" => usize::from_str_radix(&input.to_string(), 16).map_err(|_| syn::Error::new(input.span(), "Invalid hex literal")),
        _ => Err(syn::Error::new(input.span(), "Invalid integer literal")),
    }
}
// ram_region(start,end) attribute parsing
struct RamRegionAttrib {
    start: usize,
    end: usize,
}

impl Parse for RamRegionAttrib {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literals: Punctuated<LitInt, Comma> = input.parse_terminated(LitInt::parse, Comma)?;
        let literals = literals.into_iter().collect::<Vec<_>>();

        if literals.len() != 2 {
            return Err(syn::Error::new(input.span(), "Expected two integer literals, separated by comma"));
        }

        let start = &literals[0];
        let end = &literals[1];

        let start = parse_integer_lit(start)?;
        let end = parse_integer_lit(end)?;

        Ok(RamRegionAttrib {
            start,
            end
        })
    }
}

// flash_region(start,end) attribute parsing
struct FlashRegionAttrib {
    start: usize,
    end: usize,
}

impl Parse for FlashRegionAttrib {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literals: Punctuated<LitInt, Comma> = input.parse_terminated(LitInt::parse, Comma)?;
        let literals = literals.into_iter().collect::<Vec<_>>();

        if literals.len() != 2 {
            return Err(syn::Error::new(input.span(), "Expected two integer literals, separated by comma"));
        }

        let start = &literals[0];
        let end = &literals[1];

        let start = parse_integer_lit(start)?;
        let end = parse_integer_lit(end)?;

        Ok(FlashRegionAttrib {
            start,
            end,
        })
    }
}

#[proc_macro_attribute]
pub fn cell(cell_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    let attr_list = parse_macro_input!(cell_attr as CellAttribParams);

    // Extract the struct fields
    let fields = match &mut input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields,
        _ => {
            return TokenStream::from(
                syn::Error::new(Span::call_site(), "Expected a struct with named fields")
                    .to_compile_error(),
            );
        }
    };

    // Add the "init_memory" and "signature" fields if "primary" is not present
    if !attr_list.is_primary {
        let init_memory_field = Field::parse_named
            .parse2(quote! { pub init_memory: unsafe fn() })
            .unwrap();
        fields.named.push(init_memory_field);

        let signature_field = Field::parse_named
            .parse2(quote! { pub signature: u32 })
            .unwrap();
        fields.named.push(signature_field);
    }

    let output: TokenStream2 = quote! {
        #input
    };

    TokenStream::from(output)
}

//dummy ram_region
#[proc_macro_attribute]
pub fn ram_region(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

//dummy flash_region
#[proc_macro_attribute]
pub fn flash_region(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}