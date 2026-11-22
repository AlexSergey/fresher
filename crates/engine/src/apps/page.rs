//! An app's page: every file of its footprint, what each uninstall choice takes, and its categories on Scan.

use fresher_core::{
    Access, AppDetail, AppOffer, AppSummary, CategoryId, ChoiceView, FootprintItem, FootprintKind, InstalledApp,
    UninstallChoice,
};

use super::list::Listed;
use crate::results::add;

/// The page of `listed`, `app`'s row told as `summary`; `categories` are the categories of the findings on Scan that
/// belong to the app, in the registry's order. An app offered for uninstall carries the line about App Management
/// in state `management`; one not offered carries none.
pub(crate) fn detail(
    listed: &Listed,
    app: &InstalledApp,
    summary: AppSummary,
    categories: Vec<CategoryId>,
    management: Access,
) -> AppDetail {
    let notice = match summary.offer {
        AppOffer::Offered => AppDetail::management_notice(management),
        AppOffer::NotOffered { .. } => None,
    };
    let bundle = FootprintItem::new(FootprintKind::Bundle, app.path.clone(), listed.bundle, None);
    let parts = listed.parts.iter().map(|part| FootprintItem::new(part.kind, part.path.clone(), part.sizes, part.kept));
    let items: Vec<FootprintItem> = std::iter::once(bundle).chain(parts).collect();
    let choices = UninstallChoice::ALL
        .into_iter()
        .map(|choice| {
            let going = items.iter().filter(|item| goes(item, choice));
            let (sizes, count) = going.fold((fresher_core::Sizes::ZERO, 0_u32), |(sizes, count), item| {
                (add(sizes, item.sizes), count.saturating_add(1))
            });
            ChoiceView::new(choice, sizes, count)
        })
        .collect();
    AppDetail {
        summary,
        items,
        choices,
        categories,
        command: listed.installed_by.as_ref().map(fresher_core::InstalledBy::command),
        notice,
    }
}

const fn goes(item: &FootprintItem, choice: UninstallChoice) -> bool {
    match choice {
        UninstallChoice::KeepData => item.keep_data,
        UninstallChoice::Everything => item.everything,
    }
}

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;
