//! The guarantees of the cleaner's types, held by the compiler: every case in `compile_fail/` must fail to compile,
//! with the error in its `.stderr` file. `TRYBUILD=overwrite` rewrites the `.stderr` files; review the diff like a
//! golden.

mod negative {
    #[test]
    fn code_outside_the_cleaner_cannot_make_reuse_or_skip_a_check() {
        let cases = trybuild::TestCases::new();
        cases.compile_fail("tests/compile_fail/*.rs");
    }
}
