//! The guarantees of the cleanup types, held by the compiler: every case in `compile_fail/` must fail to compile, with
//! the error in its `.stderr` file. `TRYBUILD=overwrite` rewrites the `.stderr` files; review the diff like a golden.

mod negative {
    #[test]
    fn code_outside_the_core_cannot_get_past_the_confirmation() {
        let cases = trybuild::TestCases::new();
        cases.compile_fail("tests/compile_fail/*.rs");
    }
}
