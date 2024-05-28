mod defs;

use proc_macro::{TokenStream};
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::{ExprStruct, parse_macro_input, parse_quote, Path};
use syn::parse::{Parse, ParseStream};
use syn::token::{Colon};


#[proc_macro]
pub fn define_header(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ExprStruct);
    let ident = input.path;
    let ident_str = ident.to_token_stream().to_string().trim_matches('"').to_string();

    let fields = input.fields;

    let link_section = ".emcell.cur_header";
    let static_ident = format_ident!("_emcell_{}_internal", ident_str);


    let output: proc_macro2::TokenStream = {
        quote!(
            #[cortex_m_rt::entry]
            fn _emcell_internal_main() -> ! {
                let reset_vec_addr = unsafe { (<#ident as emcell::Cell>::DEVICE_CONFIG.flash_range_start as *const u32)
                .offset(1)} ;
                let reset_vec = unsafe { reset_vec_addr.read_volatile() } as *const u32;
                let reset_vec = unsafe { core::mem::transmute::<_, fn() -> !>(reset_vec) };
                unsafe {reset_vec()}
            }

            #[no_mangle]
            #[link_section = #link_section]
            pub static #static_ident : #ident = #ident {
                signature: 0xdeadbeef,
                init: unsafe { __emcell_init },
                #fields
            };

            unsafe fn __emcell_init(known_sha: [u8; 32], init_memory: bool) -> bool {
                if known_sha != <#ident as emcell::Cell>::CUR_META.struct_sha256 {
                    return false;
                }

                if (init_memory) {
                    emcell::device::init();
                }
                true
            }
        )
    };

    proc_macro::TokenStream::from(output)
}



#[proc_macro]
pub fn define_primary_header(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ExprStruct);
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
                signature: 0xbeef_dead,
                init: unsafe { __emcell_init_primary },
                #fields
            };

            unsafe fn __emcell_init_primary(known_sha: [u8; 32], _init_memory: bool) -> bool {
                if known_sha != <#ident as emcell::Cell>::CUR_META.struct_sha256 {
                    return false;
                }
                true
            }
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
        // let cells_meta_ident: Ident = input.parse()?;
        // let _: Comma = input.parse()?;
        let left: Ident = input.parse()?;
        let _: Colon = input.parse()?;
        let right: Ident = input.parse()?;

        Ok(ExternHeader { name: left, typez: right })
    }
}


fn generate_extern_header(cell_name: Ident, cell_type: Ident, forward_or_backward: Path) -> TokenStream {
    let internal_ident = format_ident!("_emcell_{}_internal", cell_type);

    let output: proc_macro2::TokenStream = {
        quote!(
            extern crate emcell;
            extern "Rust" {
                pub static #internal_ident: #cell_type;
            }

            pub struct #cell_name {
                inner: emcell::CellWrapper<#cell_type, #forward_or_backward>
            }

            impl #cell_name {
                /// Construct CellWrapper for this cell with signature check and memory initialization
                ///
                /// #Safety
                /// CellWrapper can be constructed ONLY if this cell is used by exactly one other cell project
                pub fn new() -> Option<Self> {
                    let cell = unsafe { & #internal_ident };
                    unsafe { emcell::CellWrapper::<#cell_type, #forward_or_backward>::_new_init(cell)}.map(|inner| Self { inner })
                }

                /// Construct constant CellWrapper for this cell without signature check and memory initialization
                /// Actual initialization will be performed later with ensure_init or automatically on first header access
                ///
                /// !Warning! Cell access will panic if initialization fails.
                ///
                /// #Safety
                /// CellWrapper can be constructed ONLY if this cell is used by exactly one other cell project
                pub const fn new_uninit() -> Self {
                    let cell = unsafe { & #internal_ident };
                    let inner = unsafe { emcell::CellWrapper::_new_uninit(cell) };
                    Self {
                        inner
                    }
                }

                /// Construct CellWrapper with user-provided header
                pub const fn new_dummy(dummy_header: &'static #cell_type) -> Self {
                    Self {
                        inner: emcell::CellWrapper::new_dummy(dummy_header)
                    }
                }

                /// Try to initialize cell's header in runtime. Designed to be used with new_uninit
                pub fn ensure_init(&self) -> Option<()> {
                    self.inner.ensure_init()
                }

                /// Check if this cell wrapper was created with `new_dummy` method
                pub fn is_dummy(&self) -> bool {
                    self.inner.is_dummy()
                }
            }

            impl core::ops::Deref for #cell_name {
                type Target = #cell_type;
                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }
        )
    };

    proc_macro::TokenStream::from(output)
}

/// Generate header wrapper for external cell
///
/// Cell specified in this macro **should not** be referenced with extern_header_forward in other cell projects.
/// Cell **should not** be a primary cell.
///
/// # Example
/// ```
/// use emcell_macro::extern_header_forward;
/// extern_header_forward! {
///    Cell1Wrapper: Cell1
/// }
///
///
#[proc_macro]
pub fn extern_header_forward(item: TokenStream) -> TokenStream {
    let ExternHeader { name: cell_name, typez: cell_type } = parse_macro_input!(item as ExternHeader);
    let forward_backward = parse_quote!(emcell::Forward);

    generate_extern_header(cell_name, cell_type, forward_backward)
}

/// Generate header wrapper for parent external cell
///
/// It is only allowed to extern header, which use extern_header_forward with this cell. This cell is called parent cell.
///
/// # Example
/// ```
/// use emcell_macro::extern_header_backward;
///
/// extern_header_backward! {
///    Cell1Wrapper: Cell1
/// }
///
#[proc_macro]
pub fn extern_header_backward(item: TokenStream) -> TokenStream {
    let ExternHeader { name: cell_name, typez: cell_type } = parse_macro_input!(item as ExternHeader);
    let forward_backward = parse_quote!(emcell::Backward);

    generate_extern_header(cell_name, cell_type, forward_backward)
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

/// Declare header function with signature fn() -> !, which use additional generated code for
/// switching interrupt vectors to the ones from the cell
#[proc_macro_attribute]
pub fn switch_vectors(attr: TokenStream, item: TokenStream) -> TokenStream {
    defs::switch_vectors(attr, item)
}

#[proc_macro]
pub fn device(item: TokenStream) -> TokenStream {
    defs::device(item)
}

#[proc_macro]
pub fn emcell_configuration(input: TokenStream) -> TokenStream {
    defs::emcell_configuration(input)
}