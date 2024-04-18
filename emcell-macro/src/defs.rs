use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, ToTokens};
use sha2::{Digest, Sha256};
use sha2::digest::typenum::private::IsNotEqualPrivate;
use syn::{Data, DataStruct, DeriveInput, ExprMacro, Field, Fields, FieldValue, ItemStruct, LitInt, Member, Meta, parse2, parse_macro_input, parse_quote, Type, Visibility};
use syn::parse::{Parse, Parser, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;

struct EmcellDef {
    strukt: ItemStruct,
    is_primary: bool,

    ram_region: RamRegion,
    flash_region: FlashRegion,
    struct_sha256: [u8; 32],
}

impl ToTokens for EmcellDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
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

        let hash = self.struct_sha256;


        tokens.extend(quote! {
            emcell::meta::CellDefMeta {
                name: #name,
                cell_type: #cell_type,
                ram_range_start_offs: #ram_region_start,
                ram_range_end_offs: #ram_region_end,
                flash_range_start_offs: #flash_region_start,
                flash_range_end_offs: #flash_region_end,
                struct_sha256: [#(#hash),*],
            }
        });
    }
}

struct EmcellDeviceConfiguration {
    initial_stack_pointer: usize,
    ram_region: RamRegion,
    flash_region: FlashRegion,
}

impl ToTokens for EmcellDeviceConfiguration {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let initial_stack_pointer = self.initial_stack_pointer;
        let ram_region_start = self.ram_region.start;
        let ram_region_end = self.ram_region.end;
        let flash_region_start = self.flash_region.start;
        let flash_region_end = self.flash_region.end;

        tokens.extend(quote! {
            emcell::meta::DeviceConfigMeta {
                initial_stack_ptr: #initial_stack_pointer,
                ram_range_start: #ram_region_start,
                ram_range_end: #ram_region_end,
                flash_range_start: #flash_region_start,
                flash_range_end: #flash_region_end,
            }
        });
    }

}

struct DeviceMacroParams(Punctuated<FieldValue, Comma>);

impl Parse for DeviceMacroParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let device_config = input.parse_terminated(FieldValue::parse, Comma)?;

        Ok(DeviceMacroParams(device_config))
    }
}

impl Parse for EmcellDeviceConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let device_config_macro: ExprMacro = input.parse()?;
        if !device_config_macro.mac.path.is_ident("device") {
            return Err(syn::Error::new(device_config_macro.span(), "Expected device! macro"));
        }

        //parse macro content as struct fields list
        let device_config: DeviceMacroParams = parse2(device_config_macro.mac.tokens)?;
        let device_config = device_config.0;

        let mut initial_stack_pointer = None;
        let mut ram_region_start = None;
        let mut ram_region_end = None;
        let mut flash_region_start = None;
        let mut flash_region_end = None;

        for field in device_config.iter() {
            match &field.member {
                Member::Named(ident) => {
                    fn expr_into_lit_int(expr: &syn::Expr) -> syn::Result<usize> {
                        match expr {
                            syn::Expr::Lit(lit) => {
                                match &lit.lit {
                                    syn::Lit::Int(int) => {
                                        parse_integer_lit(int)
                                    }
                                    _ => Err(syn::Error::new(lit.span(), "Expected integer literal"))
                                }
                            }
                            _ => Err(syn::Error::new(expr.span(), "Expected integer literal"))
                        }
                    }

                    match ident.to_string().as_str() {
                        "initial_stack_ptr" => {
                            initial_stack_pointer = Some(expr_into_lit_int(&field.expr)?);
                        }
                        "ram_range_start" => {
                            ram_region_start = Some(expr_into_lit_int(&field.expr)?);
                        }
                        "ram_range_end" => {
                            ram_region_end = Some(expr_into_lit_int(&field.expr)?);
                        }
                        "flash_range_start" => {
                            flash_region_start = Some(expr_into_lit_int(&field.expr)?);
                        }
                        "flash_range_end" => {
                            flash_region_end = Some(expr_into_lit_int(&field.expr)?);
                        }
                        _ => {}
                    }

                }
                _ => {
                    return Err(syn::Error::new(field.span(), "Expected named fields"));
                }
            }
        }
        let Some(initial_stack_pointer) = initial_stack_pointer else {
            return Err(syn::Error::new(device_config.span(), "initial_stack_ptr field required"));
        };
        let Some(ram_region_start) = ram_region_start else {
            return Err(syn::Error::new(device_config.span(), "ram_range_start field required"));
        };
        let Some(ram_region_end) = ram_region_end else {
            return Err(syn::Error::new(device_config.span(), "ram_range_end field required"));
        };
        let Some(flash_region_start) = flash_region_start else {
            return Err(syn::Error::new(device_config.span(), "flash_range_start field required"));
        };
        let Some(flash_region_end) = flash_region_end else {
            return Err(syn::Error::new(device_config.span(), "flash_range_end field required"));
        };

        Ok(EmcellDeviceConfiguration {
            ram_region: RamRegion { start: ram_region_start, end: ram_region_end },
            flash_region: FlashRegion { start: flash_region_start, end: flash_region_end },
            initial_stack_pointer
        })
    }
}

struct EmcellConfiguration {
    device: EmcellDeviceConfiguration,
    cells: Vec<EmcellDef>
}

impl Parse for EmcellConfiguration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        //parse device! macro
        let device: EmcellDeviceConfiguration = input.parse()?;

        let mut cells = Vec::new();
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
                        ram_region = Some(syn::parse2::<RamRegion>(meta.tokens.clone())?);
                    }
                    _ if name.is_ident("flash_region") => {
                        let meta = meta.require_list()?;
                        flash_region = Some(syn::parse2::<FlashRegion>(meta.tokens.clone())?);
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


            let mut hasher = Sha256::new();
            let fields = &strukt.fields;
            hasher.update(fields.to_token_stream().to_string().as_bytes());
            let hash = hasher.finalize();
            let struct_sha256 = hash.as_slice().try_into().unwrap();

            cells.push(EmcellDef {
                strukt,
                is_primary,
                ram_region,
                flash_region,
                struct_sha256
            });
        }

        if primary_count > 1 {
            return Err(syn::Error::new(Span::call_site(), "Only one primary cell is allowed"));
        }
        if primary_count == 0 {
            return Err(syn::Error::new(Span::call_site(), "No primary cell found. At least one cell must be marked as #[cell(primary)]"));
        }
        Ok(EmcellConfiguration{
            cells,
            device
        })
    }
}

pub fn emcell_configuration(input: TokenStream) -> TokenStream {
    let output = input.clone();
    let emcell_configuration = parse_macro_input!(input as EmcellConfiguration);

    let mut cell_names = Vec::new();
    let mut cell_idents = Vec::new();
    let mut cell_indices = Vec::new();

    let mut non_primary_cell_idents = Vec::new();
    let mut primary_cell = None;

    for (i, cell) in emcell_configuration.cells.iter().enumerate() {
        let cell_name = cell.strukt.ident.to_string();
        cell_names.push(cell_name.clone());
        cell_idents.push(cell.strukt.ident.clone());
        cell_indices.push(i);

        if cell.is_primary {
            primary_cell = Some(cell);
        }
        else {
            non_primary_cell_idents.push(cell.strukt.ident.clone());
        }
    }
    let primary_cell = primary_cell.unwrap();
    let primary_cell_ident = &primary_cell.strukt.ident;

    let cell_count = cell_names.len();

    let emcell_defs = &emcell_configuration.cells;
    let emcell_device = emcell_configuration.device;
    let output = proc_macro2::TokenStream::from(output);
    let output = quote! {
        #output

        pub type PrimaryCell = #primary_cell_ident;

        #(unsafe impl emcell::Cell for #cell_idents {
            const CUR_META: emcell::meta::CellDefMeta = META.cell_defs[#cell_indices];
            const DEVICE_CONFIG: emcell::meta::DeviceConfigMeta = META.device_configuration;
            const CELLS_META: &'static [emcell::meta::CellDefMeta] = &META.cell_defs;
            fn check_signature(&self, init_memory: bool) -> bool {
                if self.signature != <Self as emcell::WithSignature>::VALID_SIGNATURE { // a little silly :3
                    return false;
                }

                let known_sha256 = Self::CUR_META.struct_sha256;
                let sha_ok = unsafe {(self.init)(known_sha256, init_memory)};
                return sha_ok;
            }
        }

        impl #cell_idents {
            pub const fn get_cell_start_flash_addr() -> usize {
                <Self as emcell::Cell>::CUR_META.absolute_flash_start(&META.device_configuration)
            }
            pub const fn get_cell_end_flash_addr() -> usize {
                <Self as emcell::Cell>::CUR_META.absolute_flash_end(&META.device_configuration)
            }
        })*

        #(unsafe impl emcell::WithSignature for #non_primary_cell_idents {
            const VALID_SIGNATURE: u32 = 0xdeadbeef;
        })*

        unsafe impl emcell::WithSignature for #primary_cell_ident {
            const VALID_SIGNATURE: u32 = 0xbeefdead;
        }

        pub const META: emcell::meta::CellDefsMeta::<#cell_count> = emcell::meta::CellDefsMeta {
            cell_defs: [#(#emcell_defs),*],
            device_configuration: #emcell_device
        };

        pub const CELL_COUNT: usize = #cell_count;
    };

    TokenStream::from(output)
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
struct RamRegion {
    start: usize,
    end: usize,
}

impl Parse for RamRegion {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literals = input.parse_terminated(LitInt::parse, Comma)?;
        let literals = literals.into_iter().collect::<Vec<_>>();

        if literals.len() != 2 {
            return Err(syn::Error::new(input.span(), "Expected two integer literals, separated by comma"));
        }

        let start = &literals[0];
        let end = &literals[1];

        let start = parse_integer_lit(start)?;
        let end = parse_integer_lit(end)?;

        Ok(RamRegion {
            start,
            end
        })
    }
}

// flash_region(start,end) attribute parsing
struct FlashRegion {
    start: usize,
    end: usize,
}

impl Parse for FlashRegion {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literals = input.parse_terminated(LitInt::parse, Comma)?;
        let literals = literals.into_iter().collect::<Vec<_>>();

        if literals.len() != 2 {
            return Err(syn::Error::new(input.span(), "Expected two integer literals, separated by comma"));
        }

        let start = &literals[0];
        let end = &literals[1];

        let start = parse_integer_lit(start)?;
        let end = parse_integer_lit(end)?;

        Ok(FlashRegion {
            start,
            end,
        })
    }
}

pub fn cell(_cell_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut header_struct = parse_macro_input!(item as DeriveInput);

    // enforce C abi
    header_struct.attrs.push(parse_quote! { #[repr(C)] });

    // Extract the struct fields
    let mut fields = match &mut header_struct.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields,
        _ => {
            return TokenStream::from(
                syn::Error::new(Span::call_site(), "Expected a struct with named fields")
                    .to_compile_error(),
            );
        }
    };

    let mut switch_vectors_fn_ident = None;
    for field in fields.named.iter_mut() {
        for (i, attr) in field.attrs.iter().enumerate() {
            if attr.meta.path().is_ident("switch_vectors") {
                field.attrs.remove(i);
                //check signature to be fn() -> !
                let sig = &mut field.ty;
                let Some(ident) = field.ident.as_mut() else {
                    return TokenStream::from(
                        syn::Error::new(field.span(), "Expected named field")
                            .to_compile_error(),
                    );
                };

                let true_sig: Type = parse_quote! { fn() -> ! };
                if sig.to_token_stream().to_string() != true_sig.to_token_stream().to_string() {
                    return TokenStream::from(
                        syn::Error::new(ident.span(), "Expected function signature fn() -> !")
                            .to_compile_error(),
                    );
                }

                switch_vectors_fn_ident = Some(ident.clone());
                break;
            }
        }
    }

    // signature helps us ensure that our abi is indeed located at the correct address
    //
    // Also we assume that if signature is present and valid, we can call init() function safely
    // because it is guaranteed to preserve memory location (with repr C) even if header fields were changed
    let signature_field = Field::parse_named
        .parse2(quote! { pub signature: u32 })
        .unwrap();
    fields.named.insert(0, signature_field);

    let init_field = Field::parse_named
        .parse2(quote! { pub init: unsafe fn([u8; 32], bool) -> bool })
        .unwrap();
    fields.named.insert(1, init_field);


    let header_ident = &header_struct.ident;

    let impl_decl = if let Some(switch_vectors_fn_ident) = switch_vectors_fn_ident {
        let switch_vectors = Field::parse_named
            .parse2(quote! { pub _emcell_internal_switch_vectors: unsafe fn() })
            .unwrap();
        fields.named.insert(2, switch_vectors);

        quote! {

            impl #header_ident {
                pub fn switch_vectors_and_run(&self) -> ! {
                    unsafe {(self._emcell_internal_switch_vectors)()};
                    (self.#switch_vectors_fn_ident)()
                }
            }
        }
    } else {
        quote! {}
    };


    let output = quote! {
        #header_struct

        #impl_decl
    };

    TokenStream::from(output)
}

/// switch_vectors macro attribute is a way of declaring function in a cell header with a signature () -> !
///
/// This function provides additional code generation for interrupt vector switching to ones declared in other cell
///
/// # Warning
/// Do not forget to deinitialize everything you don't need before calling this function!
/// E.g. disabling unused interrupts, resetting peripheral to default state, etc.
///
/// If run is the only function to be called in other cell, it is allowed to overlap ram regions, considering full
/// deinitialization and resetting all the peripherals before calling run().
pub fn switch_vectors(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

//dummy ram_region
pub fn ram_region(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

//dummy flash_region
pub fn flash_region(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn device(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}