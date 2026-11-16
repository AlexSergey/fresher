//! A list of `u32` that holds up to four values without allocating: the interests of one name, the hits of one entry.

/// Up to four values inline; more move to the heap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SmallList {
    Inline { len: u8, items: [u32; 4] },
    Heap(Vec<u32>),
}

impl Default for SmallList {
    fn default() -> Self {
        Self::Inline { len: 0, items: [0; 4] }
    }
}

impl SmallList {
    pub(crate) fn push(&mut self, value: u32) {
        match self {
            Self::Inline { len, items } => {
                if let Some(slot) = items.get_mut(usize::from(*len)) {
                    *slot = value;
                    *len += 1;
                } else {
                    let mut spilled = items.to_vec();
                    spilled.push(value);
                    *self = Self::Heap(spilled);
                }
            }
            Self::Heap(values) => values.push(value),
        }
    }

    pub(crate) fn extend_from_slice(&mut self, values: &[u32]) {
        for &value in values {
            self.push(value);
        }
    }

    pub(crate) fn as_slice(&self) -> &[u32] {
        match self {
            Self::Inline { len, items } => items.get(..usize::from(*len)).unwrap_or_default(),
            Self::Heap(values) => values,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    /// Sorts the values and drops repeats.
    pub(crate) fn sort_dedup(&mut self) {
        match self {
            Self::Inline { len, items } => {
                let Some(values) = items.get_mut(..usize::from(*len)) else { return };
                values.sort_unstable();
                let mut kept = 0;
                for read in 0..values.len() {
                    if kept == 0 || values.get(read) != values.get(kept - 1) {
                        values.swap(kept, read);
                        kept += 1;
                    }
                }
                *len = u8::try_from(kept).unwrap_or(*len);
            }
            Self::Heap(values) => {
                values.sort_unstable();
                values.dedup();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::matcher::small_list::SmallList;

        #[test]
        fn an_empty_list_holds_nothing() {
            let mut list = SmallList::default();
            list.sort_dedup();
            assert!(list.is_empty());
            assert_eq!(list.as_slice(), &[] as &[u32]);
        }
    }

    mod positive {
        use crate::matcher::small_list::SmallList;

        #[test]
        fn four_values_stay_inline_and_a_fifth_moves_to_the_heap() {
            let mut list = SmallList::default();
            list.extend_from_slice(&[4, 3, 2, 1]);
            assert!(matches!(list, SmallList::Inline { len: 4, .. }));
            list.push(0);
            assert!(matches!(list, SmallList::Heap(_)));
            assert_eq!(list.as_slice(), &[4, 3, 2, 1, 0]);
        }

        #[test]
        fn sort_dedup_sorts_and_drops_repeats_inline_and_on_the_heap() {
            let mut inline = SmallList::default();
            inline.extend_from_slice(&[3, 1, 3, 1]);
            inline.sort_dedup();
            assert_eq!(inline.as_slice(), &[1, 3]);
            let mut heap = SmallList::default();
            heap.extend_from_slice(&[5, 1, 5, 2, 1]);
            heap.sort_dedup();
            assert_eq!(heap.as_slice(), &[1, 2, 5]);
        }
    }
}
