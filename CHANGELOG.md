# Changelog

All notable changes to this project will be documented in this file.


## [Unreleased]

### Changed
- **Breaking**: rename `Error` to `ParseError`. That describes its purpose more
    closely and is particular useful now that other error types exist in the library.

### Removed
- **Breaking**: remove `proc-macro` feature and instead offer the corresponding
    `impl`s unconditionally. Since the feature didn't enable/disable a
    dependency (`proc-macro` is a compiler provided crate) and since apparently
    it works fine in `no_std` environments, I dropped this feature. I don't
    currently see a reason why the corresponding impls should be conditional.

### Added
- `TryFrom<TokenTree> for litrs::Literal` impls
- `From<*Lit> for litrs::Literal` impls (e.g. `From<StringLit> for Literal`)
- `TryFrom<proc_macro[2]::Literal> for *Lit` (e.g. `for IntegerLit`)


## [0.1.1] - 2021-05-25
### Added
- `From` impls to create a `Literal` from references to proc-macro literal types:
    - `From<&proc_macro::Literal>`
    - `From<&proc_macro2::Literal>`
- Better examples in README and repository

## 0.1.0 - 2021-05-24
### Added
- Everything


[Unreleased]: https://github.com/LukasKalbertodt/litrs/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/LukasKalbertodt/litrs/compare/v0.1.0...v0.1.1
