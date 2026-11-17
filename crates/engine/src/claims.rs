//! Claims: one owner per path among a scan's findings, so no byte is counted twice in the category totals.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use fresher_core::{ByteSize, ClaimedPath, Finding, FindingId, ModuleId, Sizes, Subject};

/// What `resolve` decided: each path finding's counted size, and the owner of each claimed path.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Claims {
    counted: HashMap<FindingId, Sizes>,
    owners: BTreeMap<PathBuf, Finding>,
}

impl Claims {
    /// The size `finding` adds to its category's total: its own sizes minus the findings nested in it, nothing when
    /// another finding owns its path. A finding that is not about a path keeps its own sizes.
    pub(crate) fn counted(&self, finding: &Finding) -> Sizes {
        self.counted.get(&finding.id()).copied().unwrap_or_else(|| finding.finding().sizes())
    }

    /// Forgets the claims of `findings`; the others keep their counted sizes.
    pub(crate) fn forget(&mut self, findings: &HashSet<FindingId>) {
        self.counted.retain(|id, _| !findings.contains(id));
        self.owners.retain(|_, owner| !findings.contains(&owner.id()));
    }

    /// Every claimed path with the category that owns it, by path.
    pub(crate) fn paths(&self) -> Vec<ClaimedPath> {
        self.owners
            .iter()
            .map(|(path, owner)| ClaimedPath { path: path.clone(), category: owner.finding().category() })
            .collect()
    }
}

/// Resolves the claims of `findings`; `order` is the modules in registration order. Pure: the result does not
/// depend on the order of `findings`.
///
/// - Only findings about a path take part (a tool object, a Docker image say, does not).
/// - Two findings of the same path: the higher danger level owns it, then the module registered first, then the
///   category id, then the lower finding id. The other counts nothing.
/// - The deepest finding owns its subtree: an outer finding counts its sizes minus those of the owners nearest
///   inside it, compared by path components, never by string prefix.
pub(crate) fn resolve(findings: &[Finding], order: &[ModuleId]) -> Claims {
    let rank = |finding: &Finding| {
        let module = order.iter().position(|&module| module == finding.module()).unwrap_or(usize::MAX);
        (
            finding.finding().danger(),
            Reverse(module),
            Reverse(finding.finding().category().as_str().to_owned()),
            Reverse(finding.id()),
        )
    };
    let mut owners: BTreeMap<PathBuf, Finding> = BTreeMap::new();
    let mut counted = HashMap::new();
    for finding in findings {
        let Subject::Path(path) = finding.finding().subject() else { continue };
        counted.insert(finding.id(), Sizes::ZERO);
        match owners.get(path.as_path()) {
            Some(owner) if rank(owner) >= rank(finding) => {}
            _ => {
                owners.insert(path.clone(), finding.clone());
            }
        }
    }
    for owner in owners.values() {
        counted.insert(owner.id(), owner.finding().sizes());
    }
    for (path, owner) in &owners {
        if let Some(outer) = nearest_owner_above(&owners, path) {
            let less = counted.get(&outer.id()).copied().unwrap_or(Sizes::ZERO);
            counted.insert(outer.id(), minus(less, owner.finding().sizes()));
        }
    }
    Claims { counted, owners }
}

fn nearest_owner_above<'a>(owners: &'a BTreeMap<PathBuf, Finding>, path: &Path) -> Option<&'a Finding> {
    path.ancestors().skip(1).find_map(|ancestor| owners.get(ancestor))
}

/// `total` less `part`, each size stopping at zero; `total`'s completeness stays.
pub(crate) fn minus(total: Sizes, part: Sizes) -> Sizes {
    let less = |a: ByteSize, b: ByteSize| ByteSize::new(a.bytes().saturating_sub(b.bytes()));
    Sizes {
        logical: less(total.logical, part.logical),
        allocated: less(total.allocated, part.allocated),
        reclaimable: less(total.reclaimable, part.reclaimable),
        completeness: total.completeness,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{
        CategoryDescriptor, Finding, FindingId, ModuleId, NewFinding, Removability, Removal, Subject, ToolId,
    };

    use crate::testing::{bytes, finding, text};

    pub(super) fn found(id: u32, category: &CategoryDescriptor, path: &str, size: u64) -> Finding {
        Finding::new(FindingId::new(id), finding(category, PathBuf::from(path), bytes(size)), Removability::Removable)
    }

    pub(super) fn order() -> Vec<ModuleId> {
        ["alpha", "beta"].map(|id| ModuleId::new(id).unwrap()).to_vec()
    }

    fn image(id: u32, category: &CategoryDescriptor, size: u64) -> Finding {
        let subject = Subject::External { tool: ToolId::new("docker").unwrap(), id: "sha256:1".into() };
        let new = NewFinding::new(
            category,
            "image",
            subject,
            bytes(size),
            text("test.finding.title"),
            text("test.finding.reason"),
            Removal::MoveToTrash("/nowhere".into()),
        );
        Finding::new(FindingId::new(id), new, Removability::Removable)
    }

    mod negative {
        use fresher_core::DangerLevel;

        use super::{found, image, order};
        use crate::claims::resolve;
        use crate::testing::{bytes, category};

        #[test]
        fn a_nested_finding_is_not_counted_twice() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let outer = found(1, &caches, "/home/project", 100);
            let inner = found(2, &caches, "/home/project/node_modules/.cache", 30);

            let claims = resolve(&[outer.clone(), inner.clone()], &order());

            assert_eq!((claims.counted(&outer), claims.counted(&inner)), (bytes(70), bytes(30)));
            assert_eq!(claims.paths().len(), 2);
        }

        #[test]
        fn a_tool_finding_takes_no_part_in_claims() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let docker = image(1, &caches, 500);

            let claims = resolve(std::slice::from_ref(&docker), &order());

            assert_eq!(claims.counted(&docker), bytes(500));
            assert_eq!(claims.paths(), []);
        }

        #[test]
        fn a_path_that_only_shares_a_prefix_is_not_nested() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let cache = found(1, &caches, "/a/cache", 10);
            let neighbour = found(2, &caches, "/a/cache2", 20);

            let claims = resolve(&[cache.clone(), neighbour.clone()], &order());

            assert_eq!((claims.counted(&cache), claims.counted(&neighbour)), (bytes(10), bytes(20)));
        }

        #[test]
        fn the_losing_finding_of_a_path_counts_nothing() {
            let (safe, high) = (category("alpha.caches", DangerLevel::Safe), category("beta.data", DangerLevel::High));
            let low = found(1, &safe, "/a", 10);
            let owner = found(2, &high, "/a", 10);

            let claims = resolve(&[low.clone(), owner.clone()], &order());

            assert_eq!((claims.counted(&low), claims.counted(&owner)), (bytes(0), bytes(10)));
        }
    }

    mod positive {
        use fresher_core::DangerLevel;

        use super::{found, order};
        use crate::claims::resolve;
        use crate::testing::{bytes, category};

        #[test]
        fn the_deepest_finding_owns_its_subtree() {
            let caches = category("alpha.caches", DangerLevel::Safe);
            let (top, middle, deep) =
                (found(1, &caches, "/p", 100), found(2, &caches, "/p/a", 60), found(3, &caches, "/p/a/b", 25));

            let claims = resolve(&[deep.clone(), top.clone(), middle.clone()], &order());

            assert_eq!(
                [claims.counted(&top), claims.counted(&middle), claims.counted(&deep)],
                [bytes(40), bytes(35), bytes(25)]
            );
            let paths: Vec<_> = claims.paths().into_iter().map(|claim| claim.path).collect();
            assert_eq!(paths, ["/p", "/p/a", "/p/a/b"].map(std::path::PathBuf::from));
        }

        #[test]
        fn the_same_path_goes_to_the_higher_danger_then_the_first_module() {
            let alpha = category("alpha.caches", DangerLevel::Low);
            let beta_low = category("beta.caches", DangerLevel::Low);
            let beta_high = category("beta.data", DangerLevel::High);

            let by_module = resolve(&[found(1, &beta_low, "/a", 5), found(2, &alpha, "/a", 5)], &order());
            let by_danger = resolve(&[found(1, &alpha, "/a", 5), found(2, &beta_high, "/a", 5)], &order());

            assert_eq!(by_module.paths()[0].category, alpha.id());
            assert_eq!(by_danger.paths()[0].category, beta_high.id());
        }
    }

    mod property {
        use std::collections::BTreeSet;
        use std::path::{Path, PathBuf};

        use fresher_core::{CategoryDescriptor, DangerLevel, Finding};
        use proptest::prelude::*;

        use super::{found, order};
        use crate::claims::resolve;
        use crate::testing::category;

        /// The leaves of a small tree: `/r/<a>/<b>/<c>`, each with a size.
        fn leaves() -> Vec<(PathBuf, u64)> {
            let mut leaves = Vec::new();
            for a in 0_u64..3 {
                for b in 0..3 {
                    for c in 0..2 {
                        leaves.push((PathBuf::from(format!("/r/{a}/{b}/{c}")), 1 + a * 7 + b * 3 + c));
                    }
                }
            }
            leaves
        }

        /// The folders and leaves findings may name.
        fn places() -> Vec<PathBuf> {
            let mut places: BTreeSet<PathBuf> = BTreeSet::new();
            for (leaf, _) in leaves() {
                for ancestor in leaf.ancestors().take(4) {
                    places.insert(ancestor.to_path_buf());
                }
            }
            places.into_iter().collect()
        }

        fn size_under(path: &Path) -> u64 {
            leaves().iter().filter(|(leaf, _)| leaf.starts_with(path)).map(|(_, size)| size).sum()
        }

        fn categories() -> [CategoryDescriptor; 3] {
            [
                category("alpha.caches", DangerLevel::Safe),
                category("alpha.logs", DangerLevel::Low),
                category("beta.data", DangerLevel::High),
            ]
        }

        fn findings(chosen: &[(usize, usize)]) -> Vec<Finding> {
            let (places, categories) = (places(), categories());
            (1_u32..)
                .zip(chosen)
                .map(|(id, &(place, category))| {
                    let path = &places[place % places.len()];
                    let path = path.to_str().unwrap();
                    found(id, &categories[category % 3], path, size_under(Path::new(path)))
                })
                .collect()
        }

        proptest! {
            #[test]
            fn category_totals_never_exceed_the_sum_of_distinct_paths(
                chosen in prop::collection::vec((0_usize..40, 0_usize..3), 0..12),
            ) {
                let findings = findings(&chosen);

                let claims = resolve(&findings, &order());

                let total: u64 = findings.iter().map(|finding| claims.counted(finding).logical.bytes()).sum();
                let claimed: Vec<PathBuf> = claims.paths().into_iter().map(|claim| claim.path).collect();
                let union: u64 = leaves()
                    .iter()
                    .filter(|(leaf, _)| claimed.iter().any(|path| leaf.starts_with(path)))
                    .map(|(_, size)| size)
                    .sum();
                prop_assert_eq!(total, union);
            }

            #[test]
            fn resolve_does_not_depend_on_finding_order(
                chosen in prop::collection::vec((0_usize..40, 0_usize..3), 0..12),
                rotation in 0_usize..12,
            ) {
                let findings = findings(&chosen);
                let mut shuffled = findings.clone();
                shuffled.reverse();
                let len = shuffled.len().max(1);
                shuffled.rotate_left(rotation % len);

                prop_assert_eq!(resolve(&findings, &order()), resolve(&shuffled, &order()));
            }
        }
    }
}
