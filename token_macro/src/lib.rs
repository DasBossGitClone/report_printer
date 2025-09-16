use token::*;

#[proc_macro]
pub fn token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tt: Vec<proc_macro::TokenTree> = input.into_iter().collect::<Vec<_>>();

    

    todo!()
}
