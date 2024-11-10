

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Expr};

#[proc_macro_attribute]
pub fn packet_processor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let packet_id = parse_macro_input!(attr as Expr);

    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;

    let expanded = quote! {
        use crate::register_packet_processor;
        use crate::prelude::PacketProcessorRegistration;
        use crate::prelude::PacketProcessorFn;

        #func

        register_packet_processor!(#packet_id, #func_name as PacketProcessorFn);
    };

    TokenStream::from(expanded)
}
