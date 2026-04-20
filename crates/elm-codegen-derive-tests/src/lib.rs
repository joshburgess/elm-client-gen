//! Test-only crate that hosts the trybuild harness for
//! `elm-codegen-derive`. Kept separate so the derive's test build
//! doesn't try to depend on `elm-codegen-core` with the `derive`
//! feature active (which would cycle through the derive crate itself).
