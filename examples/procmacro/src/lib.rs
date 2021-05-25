use proc_macro::{Spacing, TokenStream, TokenTree};
use litrs::Literal;


/// Concatinates all input string and char literals into a single output string
/// literal.
#[proc_macro]
pub fn concat(input: TokenStream) -> TokenStream {
    let mut out = String::new();

    for tt in input {
        let lit = match tt {
            // Here we use the `From` impl of `Literal`.
            TokenTree::Literal(lit) => Literal::from(lit),
            _ => panic!("only literals allowed as input"),
        };

        // Here we can match over the literal to inspect it. All literal kinds
        // have a `value` method to return the represented value.
        println!("{:?}", lit);
        match lit {
            Literal::String(s) => out.push_str(s.value()),
            Literal::Char(c) => out.push(c.value()),
            _ => panic!("input has to be char or string literals, but this is not: {}", lit),
        }
    }

    TokenTree::Literal(proc_macro::Literal::string(&out)).into()
}

/// Repeats a given string a given number of times. Example: `repeat!
/// (3 * "foo")` will result int `"foofoofoo"`.
#[proc_macro]
pub fn repeat(input: TokenStream) -> TokenStream {
    // Validate input
    let (int, string) = match &*input.into_iter().collect::<Vec<_>>() {
        [TokenTree::Literal(int), TokenTree::Punct(p), TokenTree::Literal(string)] => {
            if p.as_char() != '*' || p.spacing() != Spacing::Alone {
                panic!("second token has to be a single `*`");
            }

            let int = match Literal::from(int) {
                Literal::Integer(i) => i,
                _ => panic!("first token has to be an integer"),
            };
            let string = match Literal::from(string) {
                Literal::String(s) => s,
                _ => panic!("third token has to be a string!"),
            };

            (int, string)
        }
        _ => panic!("expected three input tokens: `<int> * <string>`"),
    };

    // Create the output string
    let times = int.value::<u32>().expect("integer value too large :(");
    let out = (0..times).map(|_| string.value()).collect::<String>();
    TokenTree::Literal(proc_macro::Literal::string(&out)).into()
}
