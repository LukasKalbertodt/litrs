# `litrs`: parsing and inspecting Rust literals

[<img alt="CI status of master" src="https://img.shields.io/github/workflow/status/LukasKalbertodt/litrs/CI/master?label=CI&logo=github&logoColor=white&style=for-the-badge" height="23">](https://github.com/LukasKalbertodt/litrs/actions?query=workflow%3ACI+branch%3Amaster)
[<img alt="Crates.io Version" src="https://img.shields.io/crates/v/litrs?logo=rust&style=for-the-badge" height="23">](https://crates.io/crates/litrs)
[<img alt="docs.rs" src="https://img.shields.io/crates/v/litrs?color=blue&label=docs&style=for-the-badge" height="23">](https://docs.rs/litrs)

`litrs` offers functionality to parse Rust literals, i.e. tokens in the Rust programming language that represent fixed values.
This is particularly useful for proc macros, but can also be used outside of a proc-macro context.

**Why this library?**
Unfortunately, the `proc_macro` API shipped with the compiler offers no easy way to inspect literals.
There are mainly two libraries for this purpose:
[`syn`](https://github.com/dtolnay/syn) and [`literalext`](https://github.com/mystor/literalext).
The latter is deprecated.
And `syn` is oftentimes overkill for the task at hand, especially when developing function like proc-macros (e.g. `foo!(..)`).
This crate is a lightweight alternative (it compiles very quickly).
Also, when it comes to literals, `litrs` offers a bit more flexibility and a few more features compared to `syn`.

While this library is fairly young, it is extensively tested and I think the number of parsing bugs should already be very low.
I'm interested in community feedback!
If you consider using this, please speak your mind [in this issue](https://github.com/LukasKalbertodt/litrs/issues/1).

## Example

### In proc macro

```rust
use proc_macro::{TokenStream, TokenTree};
use litrs::Literal;

#[proc_macro]
pub fn foo(input: TokenStream) -> TokenStream {
    let lit = match input.into_iter().next() {
        // Use `From` impl to get a `litrs::Literal` from `proc_macro::Literal`
        Some(TokenTree::Literal(lit)) => Literal::from(lit),
        _ => panic!("invalid input"),
    };

    // You can now inspect the literal!
    match lit {
        Literal::Integer(i) => {
            println!("Got an integer specified in base {:?}", i.base());

            let value = i.value::<u64>().expect("integer literal too large");
            println!(
                "Is your integer even? {}",
                if value % 2 == 0 { "yes" } else { "no" },
            );
        }
        Literal::String(s) if s.is_raw_string() => println!("A raw string literal!"),
        _ => println!("Got some other literal kind"),
    }

    TokenStream::new() // dummy output
}
```

### Parsing from a `&str`

```rust
use litrs::Literal;

let lit = Literal::parse("3.14f32").expect("failed to parse literal");
match lit {
    Literal::Float(lit) => println!("{:?}", lit.type_suffix()),
    Literal::Integer(lit) => println!("{:?}", lit.base()),
    Literal::Bool(lit) => { /* ... */ }
    Literal::Char(lit) => { /* ... */ }
    Literal::String(lit) => { /* ... */ }
    Literal::Byte(lit) => { /* ... */ }
    Literal::ByteString(lit) => { /* ... */ }
}
```

See [**the documentation**](https://docs.rs/litrs) or the `examples/` directory for more information.


<br />

---

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
