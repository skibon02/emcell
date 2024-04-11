use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{Data, DataStruct, DeriveInput, ExprMacro, Field, Fields, FieldValue, ItemStruct, LitInt, Member, Meta, parse2, parse_macro_input};
use syn::parse::{Parse, Parser, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;

struct EmcellDef {
    strukt: ItemStruct,
    is_primary: bool,

    ram_region: RamRegion,
    flash_region: FlashRegion,
}

impl ToTokens for EmcellDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // // generate fields for the struct
        // let fields = self.strukt.fields.iter().map(|field| {
        //     let ident = &field.ident;
        //     let ty = &field.ty;
        //     quote! {
        //         #ident: core::option::Option<#ty>
        //     }
        // });

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
            emcell::meta::CellDefMeta {
                name: #name,
                cell_type: #cell_type,
                ram_range_start_offs: #ram_region_start,
                ram_range_end_offs: #ram_region_end,
                flash_range_start_offs: #flash_region_start,
                flash_range_end_offs: #flash_region_end,
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
                                        parse_integer_lit(&int)
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

            cells.push(EmcellDef {
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
    let mut non_primary_cell_idents = Vec::new();
    let mut primary_cell = None;
    for cell in &emcell_configuration.cells {
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

    let emcell_defs = emcell_configuration.cells;
    let emcell_device = emcell_configuration.device;
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

        pub static META: emcell::meta::CellDefsMeta::<#cell_defs_count> = emcell::meta::CellDefsMeta {
            cell_defs: [#(#emcell_defs),*],
            device_configuration: #emcell_device
        };
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

    let output = quote! {
        #input
    };

    TokenStream::from(output)
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