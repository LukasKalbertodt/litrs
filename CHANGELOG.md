# Changelog

All notable changes to this project will be documented in this file.


## [Unreleased]

## [0.2.3] - 2021-06-09
### Changed
- Minor internal code change to bring MSRV from 1.52 to 1.42

## [0.2.2] - 2021-06-09
### Changed
- Fixed (byte) string literal parsing by:
    - Correctly handling "string continue" sequences
    - Correctly converting `\n\r` into `\n`

## [0.2.1] - 2021-06-04
### Changed
- Fixed the `expected` value of the error returned from `TryFrom<TokenTree>` impls in some cases

## [0.2.0] - 2021-05-28
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
- `From<*Lit> for litrs::Literal` impls
- `TryFrom<proc_macro[2]::Literal> for *Lit`
- `TryFrom<TokenTree> for *Lit`
- `InvalidToken` error type for all new `TryFrom` impls


## [0.1.1] - 2021-05-25
### Added
- `From` impls to create a `Literal` from references to proc-macro literal types:
    - `From<&proc_macro::Literal>`
    - `From<&proc_macro2::Literal>`
- Better examples in README and repository

## 0.1.0 - 2021-05-24
### Added
- Everything


[Unreleased]: https://github.com/LukasKalbertodt/litrs/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/LukasKalbertodt/litrs/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/LukasKalbertodt/litrs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/LukasKalbertodt/litrs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/LukasKalbertodt/litrs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/LukasKalbertodt/litrs/compare/v0.1.0...v0.1.1
