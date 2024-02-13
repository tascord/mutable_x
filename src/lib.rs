use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, Lit, LitInt,
};

static ALPHA: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

struct MxInput {
    number: Expr,
}

impl Parse for MxInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            number: input.parse()?,
        })
    }
}

fn make_generic(is_upper: bool, number: usize) -> Vec<Ident> {
    let mut idents = Vec::new();
    for i in 1..number + 1 {
        let mut ident = String::new();
        for j in 0..(i / 26) + 1 {
            let letter = ALPHA[(i + j) % 26];
            let letter = if is_upper {
                letter.to_ascii_uppercase()
            } else {
                letter
            };

            ident.push(letter as char);
        }

        let ident = format_ident!("{}", ident);
        idents.push(ident);
    }

    idents
}

#[proc_macro]
pub fn mutable_x(input: TokenStream) -> TokenStream {
    let number = parse_macro_input!(input as MxInput).number;
    let number = match number {
        Expr::Lit(expr_lit) => match expr_lit.lit {
            syn::Lit::Int(lit_int) => lit_int.base10_parse::<usize>().unwrap(),
            _ => panic!("Expected a number"),
        },
        _ => panic!("Expected a number"),
    };

    let generics = make_generic(true, number);
    let variables = make_generic(false, number);
    let numbers = (0..number).collect::<Vec<usize>>();
    let mut_name = format_ident!("Mutable{}", number);
    let number = Lit::Int(LitInt::new(&number.to_string(), Span::call_site()));

    let expanded = quote! {
        pub struct #mut_name<#(#generics),*>(
            #((futures_signals::signal::MutableSignalCloned<#generics>, futures_signals::signal::Mutable<#generics>)),*
        )
        where
            #(#generics: Clone),*;
        impl<#(#generics),*> #mut_name<#(#generics),*>
        where
            #(#generics: Clone),*
        {
            pub fn new(#(#variables: futures_signals::signal::Mutable<#generics>),*) -> Self {
                Self(
                    #((#variables.signal_cloned(), #variables)),*
                )
            }
        }
        impl<#(#generics),*> futures_signals::signal::Signal for #mut_name<#(#generics),*>
        where
            #(#generics: Clone),*
        {
            type Item = (#(#generics),*);
            fn poll_change(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> std::task::Poll<Option<Self::Item>> {
                #(let #variables = std::pin::Pin::new(&mut self.#numbers .0).poll_change(cx);)*
                let mut changed = false;
                let mut complete = 0;

                #(
                    match #variables {
                        std::task::Poll::Ready(None) => {
                            complete += 1;
                        }
                        std::task::Poll::Ready(Some(_)) => {
                            changed = true;
                        }
                        std::task::Poll::Pending => {}
                    }
                )*

                if changed {
                    std::task::Poll::Ready(
                        Some((
                            #(self.#numbers .1.get_cloned()),*
                        ))
                    )
                } else if complete == #number {
                    std::task::Poll::Ready(None)
                } else {
                    std::task::Poll::Pending
                }
            }
        }
    };

    TokenStream::from(expanded)
}
