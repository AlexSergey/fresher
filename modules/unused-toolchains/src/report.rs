//! What the module tells the engine about a version it decided: a finding to the Trash, or a left-out line in its
//! category with its sizes. The text keys are built once, with the module.

use fresher_core::{
    CategoryDescriptor, FindingSink, LeftOutItem, NewFinding, Removal, Subject, Text, TextArg, TextKey, TextKeyError,
};

use crate::decide::{self, Reason, Verdict};
use crate::gathered::Candidate;
use crate::toolchains::{Manager, Row};

/// The keys of a manager's finding texts.
#[derive(Debug)]
struct Texts {
    title: TextKey,
    reason: TextKey,
}

impl Texts {
    fn new(title: &str, reason: &str) -> Result<Self, TextKeyError> {
        Ok(Self { title: TextKey::new(title)?, reason: TextKey::new(reason)? })
    }
}

/// The module's categories and the keys of every text a finding or a left-out carries.
#[derive(Debug)]
pub(crate) struct Report {
    categories: Vec<CategoryDescriptor>,
    rust: Texts,
    node: Texts,
    /// One per reason, in the order of `Reason::ALL`.
    left_out: Vec<(Reason, TextKey)>,
}

impl Report {
    /// The texts of the module whose categories are `categories`.
    pub(crate) fn new(categories: Vec<CategoryDescriptor>) -> Result<Self, TextKeyError> {
        let left_out = Reason::ALL
            .into_iter()
            .map(|reason| Ok((reason, TextKey::new(reason.key())?)))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            categories,
            rust: Texts::new("unused-toolchains.title.rust", "unused-toolchains.reason.rust")?,
            node: Texts::new("unused-toolchains.title.node", "unused-toolchains.reason.node")?,
            left_out,
        })
    }

    /// The keys of the left-out reasons, in the order the module declares them.
    pub(crate) fn problems(&self) -> Vec<TextKey> {
        self.left_out.iter().map(|(_, key)| key.clone()).collect()
    }

    /// Reports `verdict` on `candidate`, a version folder of `row`: a finding, a left-out in the row's category, or
    /// nothing.
    pub(crate) fn tell(&self, row: &Row, candidate: &Candidate, verdict: Verdict, out: &mut dyn FindingSink) {
        let Some(category) = self.categories.iter().find(|category| category.id().as_str() == row.category) else {
            return;
        };
        match verdict {
            Verdict::NotClaimed => {}
            Verdict::LeftOut(reason) => {
                if let Some((_, key)) = self.left_out.iter().find(|(known, _)| *known == reason) {
                    out.left_out(LeftOutItem { category: category.id(), reason: key.clone(), sizes: candidate.sizes });
                }
            }
            Verdict::Offered => {
                let Some(name) = candidate.path.file_name().and_then(|name| name.to_str()) else { return };
                let Some(seen) = candidate.inside.as_ref().and_then(decide::last_seen) else { return };
                let (texts, command) = match row.manager {
                    Manager::Rustup => (&self.rust, format!("rustup toolchain install {name}")),
                    Manager::Nvm => (&self.node, format!("nvm install {}", name.strip_prefix('v').unwrap_or(name))),
                };
                let title = Text::new(texts.title.clone()).with("name", TextArg::Text(name.to_owned()));
                let reason = Text::new(texts.reason.clone())
                    .with("date", TextArg::Date(seen))
                    .with("size", TextArg::Size(candidate.sizes.logical))
                    .with("command", TextArg::Text(command));
                let identity = format!("{}/{name}", row.id);
                let subject = Subject::Path(candidate.path.clone());
                let removal = Removal::MoveToTrash(candidate.path.clone());
                out.push(NewFinding::new(category, identity, subject, candidate.sizes, title, reason, removal));
            }
        }
    }
}
