use proc_macro::TokenStream;
use syn::{parse_macro_input, Error, Item, ItemFn};
use quote::{quote, ToTokens};

#[proc_macro_attribute]
pub fn timeit(args: TokenStream, input: TokenStream) -> TokenStream {
    let ts_args = parse_macro_input!(args as syn::AttributeArgs);
    let modpath : syn::LitStr;
    match ts_args.first().unwrap() {
        syn::NestedMeta::Lit(syn::Lit::Str(s)) => {
            modpath = s.clone();
        },
        _ => return Error::new_spanned(ts_args.first(), "timeit attribute requires origin module path").to_compile_error().into(),
    };

    let ast = parse_macro_input!(input as Item);
    match ast {
        Item::Fn(f)  => {
            let mut newf = f.clone();
            time(modpath, newf)
        },
        _ => return Error::new_spanned(ast, "timeit attribute can only be used on functions.").to_compile_error().into(),
    }
}

#[proc_macro]
pub fn init(input: TokenStream) -> TokenStream {
    let _ = input;
    let out = quote! {
        // Load the crate
        #[macro_use]
        extern crate lazy_static;
        extern crate statsd;

        // Import the client object.
        use statsd::Client;

        // Get a client with the prefix of `myapp`. The host should be the
        // IP:port of your statsd daemon.
        lazy_static! {
            [pub] static ref mut Option<Client> statd_client = Client::new("127.0.0.1:8125", "parity-statsd").unwrap();
        };
    };
    out.to_token_stream().into()
}

fn time(modpath : syn::LitStr, f : &mut ItemFn) -> TokenStream {
    let fname = f.sig.ident;
    let first = quote! {
        use std::time::Instant;
        let statsd_client = $modpath::statsd_client.unwrap();
        let _starttime = Instant::now();
    };
    let last = quote! {
        let _difftime = _starttime.elapsed().as_secs_f64();
        statsd_client.timer($fname, _difftime);
    };
    let mut out = TokenStream::new();
    f.to_token_stream().into()
}
