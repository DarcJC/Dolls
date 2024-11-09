

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitInt};

#[proc_macro_attribute]
pub fn packet_processor(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析属性参数，将其视为一个整数字面量
    let packet_id = parse_macro_input!(attr as LitInt);

    // 解析函数项
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;

    // 生成注册处理器的代码
    let expanded = quote! {
        use crate::register_packet_processor;
        use crate::prelude::PacketProcessorRegistration;
        use crate::prelude::PacketProcessorFn;

        #func

        register_packet_processor!(#packet_id, #func_name as PacketProcessorFn);
    };

    TokenStream::from(expanded)
}
