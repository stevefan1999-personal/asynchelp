use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::{parse_quote, Ident, ImplItemMethod, ReturnType, AttributeArgs};
use darling::FromMeta;

#[derive(Debug, FromMeta)]
struct StackfutureArgs {
    #[darling(default)]
    size: usize,
}

#[allow(unused)]
fn snake_to_camel(ident_str: &str) -> String {
    let mut camel_ty = String::with_capacity(ident_str.len());

    let mut last_char_was_underscore = true;
    for c in ident_str.chars() {
        match c {
            '_' => last_char_was_underscore = true,
            c if last_char_was_underscore => {
                camel_ty.extend(c.to_uppercase());
                last_char_was_underscore = false;
            }
            c => camel_ty.extend(c.to_lowercase()),
        }
    }

    camel_ty.shrink_to_fit();
    camel_ty
}

#[allow(unused)]
fn associated_type_for_rpc(method: &ImplItemMethod) -> String {
    snake_to_camel(&method.sig.ident.unraw().to_string()) + "Fut"
}

#[cfg(feature = "stackfuture")]
#[proc_macro_attribute]
pub fn tarpc_stackfuture(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = syn::parse_macro_input!(attr as AttributeArgs);
    let mut method = syn::parse_macro_input!(input as ImplItemMethod);

    let StackfutureArgs { size } = match StackfutureArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => { return TokenStream::from(e.write_errors()); }
    };


    method.sig.asyncness = None;

    // get either the return type or ().
    let ret = match &method.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ret) => quote!(#ret),
    };

    let fut_name = associated_type_for_rpc(&method);
    let fut_name_ident = Ident::new(&fut_name, method.sig.ident.span());

    // generate the updated return signature.
    method.sig.output = parse_quote! {
        -> ::stackfuture::StackFuture<'static, #ret, #size>
    };

    // transform the body of the method into Box::pin(async move { body }).
    let block = method.block.clone();
    method.block = parse_quote! [{
        ::stackfuture::StackFuture::from(async move
            #block
        )
    }];

    // generate and return type declaration for return type.
    TokenStream::from(quote! {
        type #fut_name_ident = ::stackfuture::StackFuture<'static, #ret, #size>;
        #method
    })
}
