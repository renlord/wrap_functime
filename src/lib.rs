use proc_macro::{TokenStream, TokenTree};
use syn::{parse_macro_input, parse_quote, Error, Item, ItemFn};
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
            time(modpath, &mut newf)
        },
        _ => return Error::new_spanned(ast, "timeit attribute can only be used on functions.").to_compile_error().into(),
    }
}

#[proc_macro]
pub fn init(input: TokenStream) -> TokenStream {
    let _ = input;
    let mut arg = input.into_iter();
    if arg.clone().count() != 1 {
        panic!();
    }
    let name;
    match arg.next().unwrap() {
        TokenTree::Literal(lit) => {
            name = lit.to_string();
        },
        t => return Error::new(t.span().into(), "must be string literal").to_compile_error().into(),
    };
    let out = quote! {
        // Load the crate
        use lazy_static::lazy_static;
        use statsd::Client;
        // IP:port of your statsd daemon.
        lazy_static! {
            [pub] static mut statd_client: Result<Client,statsd::client::StatsdError> = Client::new("127.0.0.1:8125", #name);
        }
    };
    dbg!(out.to_token_stream().into())
}

fn time(modpath : syn::LitStr, f : &mut ItemFn) -> TokenStream {
    let fname;
    {
        fname = &f.sig.ident;
    }
    let modpath_s = modpath.value();
    f.block.stmts.insert(0, parse_quote!{
        use std::time::Instant;
        let statsd_client = #modpath_s::statsd_client.unwrap();
        let _starttime = Instant::now();
    });
    f.block.stmts.push(parse_quote!{
        let _difftime = _starttime.elapsed().as_secs_f64();
        statsd_client.timer(#fname, _difftime);
    });
    f.to_token_stream().into()
}
