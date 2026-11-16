//! Where a folder stands in the trie of anchored paths.

use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path};

use crate::matcher::Matcher;
use crate::matcher::trie::{TrieIndex, TrieNode};

/// The trie node of a folder, or none: a small `Copy` value that travels with each folder job. The trie is
/// deterministic, so a folder stands at one node at most.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TrieState(TrieIndex);

impl TrieState {
    /// Off the trie: nothing anchored can match below.
    pub(crate) const DEAD: Self = Self(TrieIndex::MAX);

    /// On the way to an anchored interest: the walk reads such a folder first.
    pub(crate) fn is_live(self) -> bool {
        self != Self::DEAD
    }
}

impl Matcher {
    /// The state of a scan root, walked down from the file system root: a root above an anchor stands on the way to
    /// it, a root below it inside its pattern.
    pub(crate) fn root_state(&self, root: &Path) -> TrieState {
        let mut components = root.components();
        if components.next() != Some(Component::RootDir) || self.trie.is_empty() {
            return TrieState::DEAD;
        }
        components.fold(TrieState(0), |state, component| match component {
            Component::Normal(name) => self.child_state(state, name.as_bytes()),
            _ => TrieState::DEAD,
        })
    }

    /// The state of the entry `name` in a folder at `state`: one lookup.
    pub(crate) fn child_state(&self, state: TrieState, name: &[u8]) -> TrieState {
        let Some(node) = self.trie_node(state) else { return TrieState::DEAD };
        match node.literal.get(name) {
            Some(&next) => TrieState(next),
            None => match node.any {
                Some(next) if std::str::from_utf8(name).is_ok() => TrieState(next),
                _ => TrieState::DEAD,
            },
        }
    }

    pub(super) fn trie_node(&self, state: TrieState) -> Option<&TrieNode> {
        usize::try_from(state.0).ok().and_then(|index| self.trie.get(index))
    }
}
