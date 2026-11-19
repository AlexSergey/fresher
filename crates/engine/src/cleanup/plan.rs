//! The plan builder, pure over the findings: what a selection stands for, in the order the confirmation shows it,
//! and what is left out of it before the rehearsal, each with its reason.

use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use fresher_core::{
    CategoryId, CleanupSelection, CommandError, Finding, FindingId, LeftOut, MAX_PLAN_ITEMS, PlanItem, Rehearsal,
    Removal, SkipReason, Subject,
};

use crate::error::EngineError;

/// One finding of the selection, after the checks the builder makes alone.
#[derive(Debug)]
pub(crate) enum Sorted<'a> {
    /// It goes to the rehearsal.
    Kept(&'a Finding),
    Left(LeftOut),
}

/// The findings `selection` stands for: the findings picked one by one, and each picked category's findings minus
/// its exceptions, each once. `categories` are the registered categories in the overview's order; the answer is in
/// the order the confirmation shows the items: by category, then by how they go (moved, emptied, by a tool), then
/// largest first.
///
/// Refused: an empty selection or one that stands for nothing (`NothingToClean`), an id the scan did not produce
/// (`UnknownFinding`, also among the exceptions), a category nobody registered (`UnknownCategory`), more than
/// [`MAX_PLAN_ITEMS`] findings (`TooManyItems`).
pub(crate) fn expand<'a>(
    selection: &CleanupSelection,
    findings: &'a [Finding],
    categories: &[CategoryId],
) -> Result<Vec<&'a Finding>, CommandError> {
    if selection.findings().is_empty() && selection.categories().is_empty() {
        return Err(CommandError::NothingToClean);
    }
    let known = |id: FindingId| {
        let index = findings.binary_search_by_key(&id, Finding::id);
        index.map(|index| &findings[index]).map_err(|_| CommandError::UnknownFinding { id })
    };
    let mut picked: BTreeSet<FindingId> = BTreeSet::new();
    for &id in selection.findings() {
        picked.insert(known(id)?.id());
    }
    for pick in selection.categories() {
        let category = categories
            .iter()
            .find(|category| category.as_str() == pick.category())
            .ok_or_else(|| CommandError::UnknownCategory { id: pick.category().to_owned() })?;
        let except = pick.except().iter().map(|&id| known(id).map(Finding::id)).collect::<Result<HashSet<_>, _>>()?;
        let of_category = findings.iter().filter(|finding| finding.finding().category() == *category);
        picked.extend(of_category.map(Finding::id).filter(|id| !except.contains(id)));
    }
    if picked.is_empty() {
        return Err(CommandError::NothingToClean);
    }
    if picked.len() > MAX_PLAN_ITEMS as usize {
        return Err(CommandError::TooManyItems { limit: MAX_PLAN_ITEMS });
    }
    let mut selected = picked.into_iter().map(known).collect::<Result<Vec<_>, _>>()?;
    let rank = |category: CategoryId| categories.iter().position(|&known| known == category);
    selected.sort_by_key(|finding| {
        let new = finding.finding();
        (rank(new.category()), action_order(new.removal()), Reverse(new.sizes().allocated), finding.id())
    });
    Ok(selected)
}

/// Leaves out of `selected` (in its order), with the first reason that holds: a finding the app cannot remove, a
/// removal that is not about the finding's subject, a tool told to force, an item inside another item that may go
/// (it goes with that one; of two items of one path the first stays), an item on the user's `ignored` paths, inside
/// one or an ancestor of one.
pub(crate) fn sort_out<'a>(selected: &[&'a Finding], ignored: &[PathBuf]) -> Vec<Sorted<'a>> {
    let is_ignored = |path: &Path| ignored.iter().any(|place| path.starts_with(place) || place.starts_with(path));
    let early: Vec<Option<SkipReason>> = selected.iter().map(|finding| refused(finding)).collect();
    // The items that may go, by path: only they can cover another.
    let mut first: HashMap<&Path, usize> = HashMap::new();
    for (index, finding) in selected.iter().enumerate() {
        if early[index].is_none()
            && let Some(path) = removal_path(finding.finding().removal())
            && !is_ignored(path)
        {
            first.entry(path).or_insert(index);
        }
    }
    let covered = |index: usize, path: &Path| {
        first.get(path).is_some_and(|&kept| kept != index)
            || path.ancestors().skip(1).any(|ancestor| first.contains_key(ancestor))
    };
    selected
        .iter()
        .zip(early)
        .enumerate()
        .map(|(index, (&finding, early))| {
            let path = removal_path(finding.finding().removal());
            let reason = early.or_else(|| match path {
                Some(path) if covered(index, path) => Some(SkipReason::Covered),
                Some(path) if is_ignored(path) => Some(SkipReason::Ignored),
                _ => None,
            });
            reason.map_or(Sorted::Kept(finding), |reason| Sorted::Left(LeftOut::new(finding, reason)))
        })
        .collect()
}

/// The plan's items and its left-out list: each kept finding with the identity its rehearsal read, or left out with
/// the rehearsal's reason; `answers` are the rehearsal's, one per kept finding in order. The left-out list keeps the
/// order of `sorted`.
pub(crate) fn freeze(
    sorted: Vec<Sorted<'_>>,
    answers: Vec<Rehearsal>,
) -> Result<(Vec<PlanItem>, Vec<LeftOut>), EngineError> {
    let kept = sorted.iter().filter(|sorted| matches!(sorted, Sorted::Kept(_))).count();
    if answers.len() != kept {
        return Err(EngineError::Rehearsal { items: kept, answers: answers.len() });
    }
    let mut answers = answers.into_iter();
    let (mut items, mut left_out) = (Vec::new(), Vec::new());
    for sorted in sorted {
        let finding = match sorted {
            Sorted::Left(left) => {
                left_out.push(left);
                continue;
            }
            Sorted::Kept(finding) => finding,
        };
        match answers.next() {
            Some(Rehearsal::Passed(identity)) => items.push(PlanItem::new(finding).with_identity(identity)),
            Some(Rehearsal::Skipped(reason)) => left_out.push(LeftOut::new(finding, reason)),
            None => return Err(EngineError::Rehearsal { items: kept, answers: kept }),
        }
    }
    Ok((items, left_out))
}

/// The items of `sorted` to rehearse, in order.
pub(crate) fn to_rehearse(sorted: &[Sorted<'_>]) -> Vec<PlanItem> {
    let kept = sorted.iter().filter_map(|sorted| match sorted {
        Sorted::Kept(finding) => Some(PlanItem::new(finding)),
        Sorted::Left(_) => None,
    });
    kept.collect()
}

/// Why the builder leaves `finding` out before anything else: not removable, its removal not about its subject, or a
/// tool told to force.
fn refused(finding: &Finding) -> Option<SkipReason> {
    let removability = finding.removability();
    if !removability.is_removable() {
        return Some(SkipReason::NotRemovable { removability });
    }
    let new = finding.finding();
    match (new.removal(), new.subject()) {
        (Removal::MoveToTrash(path) | Removal::EmptyFolder(path), Subject::Path(subject)) if path == subject => None,
        (Removal::ToolCommand(command), Subject::External { tool, .. }) if command.tool() == *tool => {
            command.forces().then_some(SkipReason::ForcedTool)
        }
        _ => Some(SkipReason::NotTheSubject),
    }
}

fn removal_path(removal: &Removal) -> Option<&Path> {
    match removal {
        Removal::MoveToTrash(path) | Removal::EmptyFolder(path) => Some(path),
        Removal::ToolCommand(_) => None,
    }
}

/// Moved items first, then emptied folders, then a tool's items: each group of the confirmation goes one way.
const fn action_order(removal: &Removal) -> u8 {
    match removal {
        Removal::MoveToTrash(_) => 0,
        Removal::EmptyFolder(_) => 1,
        Removal::ToolCommand(_) => 2,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{
        CategoryDescriptor, CategoryId, CleanupSelection, DangerLevel, Finding, FindingId, NewFinding, Removability,
        Removal, SkipReason, Subject, ToolCommand, ToolId,
    };

    use super::Sorted;
    use crate::cleanup::testing::select;
    use crate::testing::{bytes, category, finding, text};

    fn alpha() -> CategoryDescriptor {
        category("alpha.caches", DangerLevel::Safe)
    }

    fn beta() -> CategoryDescriptor {
        category("beta.logs", DangerLevel::Low)
    }

    fn categories() -> Vec<CategoryId> {
        vec![alpha().id(), beta().id()]
    }

    fn found(id: u32, category: &CategoryDescriptor, path: &str, size: u64) -> Finding {
        Finding::new(FindingId::new(id), finding(category, path, bytes(size)), Removability::Removable)
    }

    fn removed_by(id: u32, subject: Subject, removal: Removal) -> Finding {
        let new = NewFinding::new(
            &alpha(),
            format!("{id}"),
            subject,
            bytes(1),
            text("test.finding.title"),
            text("test.finding.reason"),
            removal,
        );
        Finding::new(FindingId::new(id), new, Removability::Removable)
    }

    fn tool(id: u32, args: &[&str]) -> Finding {
        let docker = ToolId::new("docker").unwrap();
        let command = ToolCommand::new(docker, args.iter().map(|&arg| arg.to_owned()).collect()).unwrap();
        removed_by(id, Subject::External { tool: docker, id: format!("{id}") }, Removal::ToolCommand(command))
    }

    fn ids(findings: &[&Finding]) -> Vec<u32> {
        findings.iter().map(|finding| finding.id().get()).collect()
    }

    /// For each finding of `findings` sorted out, its id and its reason, `None` for one kept.
    fn sorted(findings: &[Finding], ignored: &[PathBuf]) -> Vec<(u32, Option<SkipReason>)> {
        let selected: Vec<&Finding> = findings.iter().collect();
        super::sort_out(&selected, ignored)
            .into_iter()
            .map(|sorted| match sorted {
                Sorted::Kept(finding) => (finding.id().get(), None),
                Sorted::Left(left) => (left.finding().get(), Some(left.reason().clone())),
            })
            .collect()
    }

    fn expand(selection: &CleanupSelection, findings: &[Finding]) -> Result<Vec<u32>, fresher_core::CommandError> {
        super::expand(selection, findings, &categories()).map(|selected| ids(&selected))
    }

    mod negative {
        use std::path::PathBuf;

        use fresher_core::{
            CommandError, FindingId, MAX_PLAN_ITEMS, Rehearsal, Removability, Removal, SkipReason, Subject, ToolId,
        };

        use super::{alpha, beta, expand, found, removed_by, sorted, tool};
        use crate::cleanup::testing::select;
        use crate::error::EngineError;

        #[test]
        fn an_empty_selection_is_nothing_to_clean() {
            let findings = [found(1, &alpha(), "/home/a", 1)];

            assert_eq!(expand(&select(&[], &[]), &findings), Err(CommandError::NothingToClean));
            assert_eq!(expand(&select(&[], &[("beta.logs", &[])]), &findings), Err(CommandError::NothingToClean));
            assert_eq!(expand(&select(&[], &[("alpha.caches", &[1])]), &findings), Err(CommandError::NothingToClean));
        }

        #[test]
        fn an_unknown_finding_is_refused() {
            let findings = [found(1, &alpha(), "/home/a", 1)];
            let unknown = CommandError::UnknownFinding { id: FindingId::new(9) };

            assert_eq!(expand(&select(&[1, 9], &[]), &findings), Err(unknown.clone()));
            assert_eq!(expand(&select(&[], &[("alpha.caches", &[9])]), &findings), Err(unknown));
        }

        #[test]
        fn an_unknown_category_is_refused() {
            let findings = [found(1, &alpha(), "/home/a", 1)];

            for id in ["gamma.caches", "alpha", "caches"] {
                let error = expand(&select(&[1], &[(id, &[])]), &findings);
                assert_eq!(error, Err(CommandError::UnknownCategory { id: id.to_owned() }), "{id}");
            }
        }

        #[test]
        fn more_than_the_limit_is_refused() {
            let findings: Vec<_> =
                (1..=MAX_PLAN_ITEMS + 1).map(|id| found(id, &alpha(), &format!("/home/{id}"), 1)).collect();

            let error = expand(&select(&[], &[("alpha.caches", &[])]), &findings);

            assert_eq!(error, Err(CommandError::TooManyItems { limit: MAX_PLAN_ITEMS }));
            assert_eq!(expand(&select(&[], &[("alpha.caches", &[1])]), &findings).map(|ids| ids.len()), Ok(10_000));
        }

        #[test]
        fn a_not_removable_finding_is_left_out() {
            let locked = fresher_core::Finding::new(
                FindingId::new(1),
                crate::testing::finding(&alpha(), "/home/a", crate::testing::bytes(1)),
                Removability::Locked,
            );

            let reasons = sorted(&[locked], &[]);

            assert_eq!(reasons, [(1, Some(SkipReason::NotRemovable { removability: Removability::Locked }))]);
        }

        #[test]
        fn a_category_selection_leaves_out_an_item_that_is_not_removable() {
            let admin = fresher_core::Finding::new(
                FindingId::new(2),
                crate::testing::finding(&alpha(), "/home/b", crate::testing::bytes(1)),
                Removability::NeedsAdministrator,
            );
            let findings = [found(1, &alpha(), "/home/a", 1), admin];

            let selected = expand(&select(&[], &[("alpha.caches", &[])]), &findings).unwrap();

            assert_eq!(selected, [1, 2], "the whole category is selected");
            let needs_administrator = SkipReason::NotRemovable { removability: Removability::NeedsAdministrator };
            assert_eq!(sorted(&findings, &[]), [(1, None), (2, Some(needs_administrator))]);
        }

        #[test]
        fn a_removal_that_is_not_its_subject_is_left_out() {
            let docker = ToolId::new("docker").unwrap();
            let brew = ToolId::new("brew").unwrap();
            let elsewhere = removed_by(1, Subject::Path("/home/a".into()), Removal::MoveToTrash("/home/b".into()));
            let emptied = removed_by(2, Subject::Path("/home/a".into()), Removal::EmptyFolder("/home/a/x".into()));
            let external =
                removed_by(3, Subject::External { tool: docker, id: "a".into() }, Removal::MoveToTrash("/a".into()));
            let mut other_tool = tool(4, &["image", "rm", "a"]);
            if let Removal::ToolCommand(command) = other_tool.finding().removal().clone() {
                other_tool =
                    removed_by(4, Subject::External { tool: brew, id: "a".into() }, Removal::ToolCommand(command));
            }
            let path_tool = match tool(5, &["rm"]).finding().removal().clone() {
                removal @ Removal::ToolCommand(_) => removed_by(5, Subject::Path("/home/a".into()), removal),
                _ => unreachable!(),
            };

            let reasons = sorted(&[elsewhere, emptied, external, other_tool, path_tool], &[]);

            assert!(reasons.iter().all(|(_, reason)| *reason == Some(SkipReason::NotTheSubject)), "{reasons:?}");
        }

        #[test]
        fn a_forced_tool_command_is_left_out() {
            let forced = [&["rm", "-f", "a"][..], &["rm", "--force"], &["rm", "--force=true"], &["rm", "-rf", "a"]];
            let fine = [&["rm", "a"][..], &["rm", "--filter", "x"], &["rm", "-q"], &["rm", "-"], &["rm", "--forced-x"]];

            let findings: Vec<_> = forced.iter().chain(&fine).zip(1..).map(|(args, id)| tool(id, args)).collect();
            let reasons = sorted(&findings, &[]);

            let left: Vec<_> = reasons.iter().filter(|(_, reason)| reason.is_some()).collect();
            assert_eq!(left.len(), forced.len(), "{reasons:?}");
            assert!(left.iter().all(|(id, reason)| *id <= 4 && *reason == Some(SkipReason::ForcedTool)));
        }

        #[test]
        fn an_item_inside_another_is_covered() {
            let findings = [
                found(1, &alpha(), "/home/a/inner", 1),
                found(2, &alpha(), "/home/a", 1),
                found(3, &alpha(), "/home/ab", 1),
                found(4, &beta(), "/home/a", 1),
            ];

            let reasons = sorted(&findings, &[]);

            assert_eq!(reasons, [(1, Some(SkipReason::Covered)), (2, None), (3, None), (4, Some(SkipReason::Covered))]);
        }

        #[test]
        fn an_item_under_a_left_out_one_is_not_covered_by_it() {
            let locked = fresher_core::Finding::new(
                FindingId::new(1),
                crate::testing::finding(&alpha(), "/home/a", crate::testing::bytes(1)),
                Removability::NeedsAdministrator,
            );

            let reasons = sorted(&[locked, found(2, &alpha(), "/home/a/inner", 1)], &[]);

            assert_eq!(reasons[1], (2, None));
        }

        #[test]
        fn an_ignored_path_is_left_out() {
            let findings = [
                found(1, &alpha(), "/home/kept/inner", 1),
                found(2, &alpha(), "/home", 1),
                found(3, &alpha(), "/home/keptx", 1),
            ];

            let reasons = sorted(&findings[..1], &[PathBuf::from("/home/kept")]);
            let above = sorted(&findings[1..], &[PathBuf::from("/home/kept")]);

            assert_eq!(reasons, [(1, Some(SkipReason::Ignored))]);
            assert_eq!(above, [(2, Some(SkipReason::Ignored)), (3, None)]);
        }

        #[test]
        fn a_rehearsal_that_does_not_answer_every_item_is_refused() {
            let findings = [found(1, &alpha(), "/home/a", 1), found(2, &alpha(), "/home/b", 1)];
            let selected: Vec<_> = findings.iter().collect();
            let sorted = super::super::sort_out(&selected, &[]);

            let error = super::super::freeze(sorted, vec![Rehearsal::Passed(None)]);

            assert_eq!(error.err(), Some(EngineError::Rehearsal { items: 2, answers: 1 }));
        }
    }

    mod positive {
        use fresher_core::{FindingId, ItemIdentity, PlanItem, Rehearsal, SkipReason};

        use super::{alpha, beta, expand, found, ids, select, sorted, tool};
        use crate::cleanup::plan::{freeze, sort_out, to_rehearse};

        #[test]
        fn a_category_expands_to_its_findings_minus_its_exceptions() {
            let findings = [
                found(1, &alpha(), "/home/a", 1),
                found(2, &beta(), "/home/l", 1),
                found(3, &alpha(), "/home/b", 1),
                found(4, &alpha(), "/home/c", 1),
            ];

            let selected = expand(&select(&[2], &[("alpha.caches", &[3])]), &findings);

            assert_eq!(selected, Ok(vec![1, 4, 2]));
        }

        #[test]
        fn a_finding_picked_twice_counts_once() {
            let findings = [found(1, &alpha(), "/home/a", 1), found(2, &alpha(), "/home/b", 1)];

            let selected = expand(&select(&[1, 1, 2], &[("alpha.caches", &[]), ("alpha.caches", &[2])]), &findings);

            assert_eq!(selected, Ok(vec![1, 2]));
        }

        #[test]
        fn the_items_go_by_category_then_how_they_go_then_largest_first() {
            let findings = [
                found(1, &beta(), "/home/l", 50),
                tool(2, &["image", "rm", "a"]),
                found(3, &alpha(), "/home/a", 10),
                found(4, &alpha(), "/home/b", 30),
                found(5, &alpha(), "/home/c", 30),
            ];

            let selected = expand(&select(&[1, 2, 3, 4, 5], &[]), &findings);

            assert_eq!(selected, Ok(vec![4, 5, 3, 2, 1]));
        }

        #[test]
        fn freezing_takes_the_rehearsals_identities_and_reasons_in_order() {
            let findings = [
                found(1, &alpha(), "/home/a", 1),
                found(2, &alpha(), "/home/a/in", 1),
                found(3, &alpha(), "/home/b", 1),
                found(4, &alpha(), "/home/c", 1),
            ];
            let selected: Vec<_> = findings.iter().collect();
            let sorted = sort_out(&selected, &[]);
            let rehearsed: Vec<_> = to_rehearse(&sorted).iter().map(PlanItem::finding).collect();
            let identity = ItemIdentity::new(1, 7, fresher_core::EntryKind::Folder);
            let answers = vec![
                Rehearsal::Passed(Some(identity)),
                Rehearsal::Skipped(SkipReason::Vanished),
                Rehearsal::Passed(None),
            ];

            let (items, left_out) = freeze(sorted, answers).unwrap();

            assert_eq!(rehearsed, [1, 3, 4].map(FindingId::new));
            assert_eq!(items.iter().map(PlanItem::finding).collect::<Vec<_>>(), [1, 4].map(FindingId::new));
            assert_eq!(items[0].identity(), Some(identity));
            let left: Vec<_> = left_out.iter().map(|left| (left.finding().get(), left.reason().clone())).collect();
            assert_eq!(left, [(2, SkipReason::Covered), (3, SkipReason::Vanished)]);
        }

        #[test]
        fn removable_findings_about_their_subject_are_kept() {
            let findings = [found(1, &alpha(), "/home/a", 1), tool(2, &["image", "rm", "-q", "a"])];

            assert_eq!(sorted(&findings, &[]), [(1, None), (2, None)]);
            assert_eq!(ids(&findings.iter().collect::<Vec<_>>()), [1, 2]);
        }
    }

    mod property {
        use std::collections::BTreeSet;

        use fresher_core::{CommandError, Finding};
        use proptest::prelude::*;

        use super::{alpha, beta, categories, found, select};
        use crate::cleanup::plan::{Sorted, expand, sort_out};

        /// Up to 40 findings, each of `alpha` or `beta`, at a path among a few nested ones.
        fn findings() -> impl Strategy<Value = Vec<Finding>> {
            prop::collection::vec((any::<bool>(), 0_usize..6, 1_u64..100), 1..40).prop_map(|specs| {
                let paths = ["/h/a", "/h/a/b", "/h/a/b/c", "/h/ab", "/h/x", "/h/x/y"];
                specs
                    .into_iter()
                    .zip(1..)
                    .map(|((in_alpha, path, size), id)| {
                        let category = if in_alpha { alpha() } else { beta() };
                        found(id, &category, paths[path], size)
                    })
                    .collect()
            })
        }

        proptest! {
            #[test]
            fn the_plan_never_holds_an_item_that_was_not_selected(
                findings in findings(),
                picks in prop::collection::vec(1_u32..45, 0..10),
                whole in prop::option::of(any::<bool>()),
                except in prop::collection::vec(1_u32..45, 0..10),
            ) {
                let known = |id: &u32| findings.iter().any(|finding| finding.id().get() == *id);
                let picks: Vec<u32> = picks.into_iter().filter(known).collect();
                let except: Vec<u32> = except.into_iter().filter(known).collect();
                let category = whole.map(|alpha| if alpha { "alpha.caches" } else { "beta.logs" });
                let categories_picked: Vec<(&str, &[u32])> = category.map(|id| (id, &except[..])).into_iter().collect();
                let selection = select(&picks, &categories_picked);

                let mut allowed: BTreeSet<u32> = picks.iter().copied().collect();
                if let Some(category) = category {
                    let of_category = findings.iter().filter(|finding| finding.finding().category().as_str() == category);
                    allowed.extend(of_category.map(|finding| finding.id().get()).filter(|id| !except.contains(id)));
                }
                match expand(&selection, &findings, &categories()) {
                    Ok(selected) => {
                        let kept = sort_out(&selected, &[]);
                        for sorted in kept {
                            let id = match sorted {
                                Sorted::Kept(finding) => finding.id().get(),
                                Sorted::Left(left) => left.finding().get(),
                            };
                            prop_assert!(allowed.contains(&id), "{id} was not selected");
                        }
                        prop_assert_eq!(selected.len(), allowed.len());
                    }
                    Err(error) => {
                        prop_assert_eq!(error, CommandError::NothingToClean);
                        prop_assert!(allowed.is_empty());
                    }
                }
            }
        }
    }
}
