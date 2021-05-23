use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, Error, Item, ItemFn};
use quote::{quote, ToTokens};

#[proc_macro_attribute]
pub fn timeit(args: TokenStream, input: TokenStream) -> TokenStream {
    let ts_args = parse_macro_input!(args as syn::AttributeArgs);
    let mut ts_args_iter = ts_args.iter();
    let modpath : syn::LitStr;
    match ts_args_iter.next() {
        Some(syn::NestedMeta::Lit(syn::Lit::Str(s))) => {
            modpath = s.clone();
        },
        _ => return Error::new_spanned(ts_args.first(), "timeit attribute requires origin module path").to_compile_error().into(),
    };
    let ast = parse_macro_input!(input as Item);
    match ast {
        Item::Fn(f)  => {
            let mut newf = f.clone();
            time(modpath, &mut newf)
        },
        _ => return Error::new_spanned(ast, "timeit attribute can only be used on functions.").to_compile_error().into(),
    }
}

#[proc_macro]
pub fn init(input: TokenStream) -> TokenStream {
    let _ = input;
    let out = quote! {
        // Load the crate
        use statsd::Client;
        use lazy_static::lazy_static;
        // IP:port of your statsd daemon.
        lazy_static! {
            pub static ref STATSD_CLIENT:Client = {
                Client::new("127.0.0.1:8000", "test").unwrap()
            };
        }
    };
    out.to_token_stream().into()
}

fn time(modpath : syn::LitStr, f : &mut ItemFn) -> TokenStream {
    let fname = f.sig.ident.clone().to_string();
    let _modpath_s = modpath.value();
    f.block.stmts.insert(0, parse_quote! {
        let _starttime = std::time::Instant::now();
    });
    f.block.stmts.push(parse_quote! {
        STATSD_CLIENT.timer(#fname, _starttime.elapsed().as_secs_f64());
    });
    f.to_token_stream().into()
}
