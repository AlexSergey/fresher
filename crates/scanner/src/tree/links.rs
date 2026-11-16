//! Bytes that several paths share: a hard link group's (one file, several names) and a clone group's shared blocks.
//! Each group's bytes are counted once, at the member that comes first in depth-first name order, so the totals do
//! not depend on the order the workers ran in.

use crate::tree::node::{NodeIndex, Sums};

/// One member of a group: the group's key, the node that holds the member (its folder when the file is folded, its
/// own node when kept), and the bytes the group may count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Member {
    pub(crate) device: u64,
    pub(crate) id: u64,
    pub(crate) holder: NodeIndex,
    pub(crate) bytes: Sums,
}

/// What `resolve` decided: the bytes each first member's holder gains, and the holders of every other member.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Resolved {
    pub(crate) credited: Vec<(NodeIndex, Sums)>,
    pub(crate) others: Vec<NodeIndex>,
}

/// The members of every group of one kind, keyed by `(device, id)`: the file id of a hard link group, the clone id of
/// a clone group.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct LinkSet {
    members: Vec<Member>,
}

impl LinkSet {
    pub(crate) fn add(&mut self, device: u64, id: u64, holder: NodeIndex, bytes: Sums) {
        self.members.push(Member { device, id, holder, bytes });
    }

    /// Every member, for the cache codec.
    pub(crate) fn members(&self) -> &[Member] {
        &self.members
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub(crate) fn heap_bytes(&self) -> usize {
        self.members.capacity() * size_of::<Member>()
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.members.shrink_to_fit();
    }

    /// Orders every group by `rank` (a node's position in depth-first name order) and gives each group's bytes to
    /// its first member: the largest of its members' bytes, part by part (a hard link's members are equal; a clone's
    /// shared part is the most any member shares).
    pub(crate) fn resolve(&mut self, rank: impl Fn(NodeIndex) -> u32) -> Resolved {
        self.members.sort_by_key(|member| (member.device, member.id, rank(member.holder), member.holder));
        let mut resolved = Resolved::default();
        for group in self.members.chunk_by(|left, right| (left.device, left.id) == (right.device, right.id)) {
            let Some((first, rest)) = group.split_first() else { continue };
            let bytes = group.iter().fold(Sums::ZERO, |most, member| max(most, member.bytes));
            resolved.credited.push((first.holder, bytes));
            resolved.others.extend(rest.iter().map(|member| member.holder));
        }
        resolved
    }
}

fn max(left: Sums, right: Sums) -> Sums {
    Sums {
        files: left.files.max(right.files),
        logical: left.logical.max(right.logical),
        allocated: left.allocated.max(right.allocated),
        reclaimable: left.reclaimable.max(right.reclaimable),
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::tree::links::LinkSet;
        use crate::tree::node::NodeIndex;
        use crate::tree::testing::sums;

        #[test]
        fn an_empty_set_credits_nothing() {
            let mut set = LinkSet::default();
            assert!(set.is_empty());
            assert_eq!(set.resolve(|_| 0).credited, []);
        }

        #[test]
        fn two_groups_are_not_merged() {
            let mut set = LinkSet::default();
            let holder = NodeIndex::new(1).unwrap();
            set.add(1, 10, holder, sums(0, 5, 5, 5));
            set.add(2, 10, holder, sums(0, 7, 7, 7));
            assert_eq!(set.resolve(|_| 0).credited.len(), 2);
        }
    }

    mod positive {
        use crate::tree::links::LinkSet;
        use crate::tree::node::NodeIndex;
        use crate::tree::testing::sums;

        #[test]
        fn the_first_member_by_rank_takes_the_largest_bytes() {
            let mut set = LinkSet::default();
            let (late, early) = (NodeIndex::new(1).unwrap(), NodeIndex::new(2).unwrap());
            set.add(1, 10, late, sums(0, 0, 100, 0));
            set.add(1, 10, early, sums(0, 0, 90, 0));
            let resolved = set.resolve(|holder| u32::from(holder != early));
            assert_eq!(resolved.credited, [(early, sums(0, 0, 100, 0))]);
            assert_eq!(resolved.others, [late]);
        }
    }
}
