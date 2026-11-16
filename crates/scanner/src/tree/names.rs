//! Every name of the tree in one byte buffer.

use crate::tree::TreeError;

/// Where a name lies in [`Names`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NameRef {
    pub(crate) offset: u32,
    pub(crate) len: u16,
}

/// The names of the tree, back to back. Names are bytes: a file name is not always UTF-8.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Names {
    bytes: Vec<u8>,
}

impl Names {
    /// Stores `name`. A name longer than `u16::MAX` bytes cannot exist on APFS and is refused; so is a buffer past
    /// 4 GiB.
    pub(crate) fn push(&mut self, name: &[u8]) -> Result<NameRef, TreeError> {
        let len = u16::try_from(name.len()).map_err(|_| TreeError::NameTooLong { len: name.len() })?;
        let offset = u32::try_from(self.bytes.len()).map_err(|_| TreeError::Full)?;
        offset.checked_add(u32::from(len)).ok_or(TreeError::Full)?;
        self.bytes.extend_from_slice(name);
        Ok(NameRef { offset, len })
    }

    /// The name stored at `name`; empty for a reference this buffer did not give.
    pub(crate) fn get(&self, name: NameRef) -> &[u8] {
        let start = name.offset as usize;
        self.bytes.get(start..start + usize::from(name.len)).unwrap_or_default()
    }

    pub(crate) fn heap_bytes(&self) -> usize {
        self.bytes.capacity()
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.bytes.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::tree::TreeError;
        use crate::tree::names::{NameRef, Names};

        #[test]
        fn a_name_too_long_is_refused() {
            let mut names = Names::default();
            let long = vec![b'a'; usize::from(u16::MAX) + 1];
            assert_eq!(names.push(&long), Err(TreeError::NameTooLong { len: long.len() }));
            assert_eq!(names.heap_bytes(), 0);
        }

        #[test]
        fn a_foreign_reference_reads_as_empty() {
            let names = Names::default();
            assert_eq!(names.get(NameRef { offset: 10, len: 3 }), b"");
        }
    }

    mod positive {
        use crate::tree::names::Names;

        #[test]
        fn names_are_kept_as_bytes() {
            let mut names = Names::default();
            let first = names.push(b"Library").unwrap();
            let second = names.push(b"\xffnot utf-8").unwrap();
            let longest = names.push(&vec![b'a'; usize::from(u16::MAX)]).unwrap();
            assert_eq!(names.get(first), b"Library");
            assert_eq!(names.get(second), b"\xffnot utf-8");
            assert_eq!(names.get(longest).len(), usize::from(u16::MAX));
        }
    }
}
