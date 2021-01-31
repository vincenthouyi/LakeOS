#![no_std]

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let inputs = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let vis = &input.vis;

    if name != "main" {
        return TokenStream::from(quote_spanned! { name.span() =>
            compile_error!("only the main function can be tagged with #[async_std::main]"),
        });
    }

    if input.sig.asyncness.is_none() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("the async keyword is missing from the function declaration"),
        });
    }

    let result = quote! {
        #[no_mangle]
        #vis fn main() #ret {
            #(#attrs)*
            async fn main(#inputs) #ret {
                #body
            }

            use core::future::Future;
            fn worker_thread() -> ! {
                naive::task::spawn(main());
                naive::task::global_executor().run();

                loop {}
            }

            use rustyl4api::object::{EndpointObj};
            use naive::ep_server::{EpServer, EP_SERVER};
            use naive::space_manager::gsm;

            let ep = gsm!().alloc_object::<EndpointObj>(12).unwrap();

            let ep_server = EP_SERVER.try_get_or_init(|| EpServer::new(ep)).unwrap();

            naive::thread::spawn(worker_thread);

            ep_server.run();

            loop {}
            // async_std::task::block_on(async {
            //     main().await
            // })
        }

    };

    result.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
