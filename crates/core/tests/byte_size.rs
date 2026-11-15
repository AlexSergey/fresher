//! The public API of `fresher-core`, as other crates see it.

mod negative {
    use fresher_core::ByteSize;

    #[test]
    fn a_sum_past_the_maximum_is_refused() {
        let almost_full = ByteSize::new(u64::MAX - 1);

        assert_eq!(almost_full.checked_add(ByteSize::new(2)), None);
    }
}

mod positive {
    use fresher_core::ByteSize;

    #[test]
    fn sizes_add_up_across_findings() {
        let total = [10, 20, 30].into_iter().map(ByteSize::new).try_fold(ByteSize::ZERO, ByteSize::checked_add);

        assert_eq!(total, Some(ByteSize::new(60)));
    }
}
