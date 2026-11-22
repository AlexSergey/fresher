//! A runtime's card: the design's section 4, from its row, its state and what the walk saw of its disk file.

use fresher_core::{
    ByteSize, DangerLevel, Figure, IdError, Insight, InsightError, InsightPart, ShownCommand, Text, TextArg, TextKey,
    TextKeyError, ToolId,
};

use crate::answer::{Kind, Usage};
use crate::decide::{Disk, State};
use crate::runtimes::{Maximum, Runtime};

/// Why a card could not be built: only a bug in the module's own table or keys, which its tests catch.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum ReportError {
    #[error(transparent)]
    Insight(#[from] InsightError),
    #[error(transparent)]
    Key(#[from] TextKeyError),
    #[error(transparent)]
    Id(#[from] IdError),
}

/// A part of what Docker keeps, in the card's order.
#[derive(Clone, Copy, Debug)]
enum Part {
    BuildCache,
    Images,
    Containers,
    Volumes,
}

impl Part {
    const ALL: [Self; 4] = [Self::BuildCache, Self::Images, Self::Containers, Self::Volumes];

    const fn id(self) -> &'static str {
        match self {
            Self::BuildCache => "build-cache",
            Self::Images => "images",
            Self::Containers => "containers",
            Self::Volumes => "volumes",
        }
    }

    const fn danger(self) -> DangerLevel {
        match self {
            Self::BuildCache => DangerLevel::Safe,
            Self::Images => DangerLevel::Low,
            Self::Containers => DangerLevel::Medium,
            Self::Volumes => DangerLevel::High,
        }
    }

    /// The Docker command that frees the part, after `docker --context <context>`.
    const fn command(self) -> &'static [&'static str] {
        match self {
            Self::BuildCache => &["builder", "prune", "--all"],
            Self::Images => &["image", "prune", "--all"],
            Self::Containers => &["container", "prune"],
            Self::Volumes => &["volume", "prune"],
        }
    }

    /// Whether its detail counts how many are unused; the build cache's does not.
    const fn counted(self) -> bool {
        !matches!(self, Self::BuildCache)
    }

    const fn of(self, usage: &Usage) -> &Kind {
        match self {
            Self::BuildCache => &usage.build_cache,
            Self::Images => &usage.images,
            Self::Containers => &usage.containers,
            Self::Volumes => &usage.volumes,
        }
    }
}

/// The card of `runtime` in `state`, with what the walk saw of its disk file.
pub(crate) fn insight(runtime: &Runtime, state: &State, disk: &Disk) -> Result<Insight, ReportError> {
    let name = || TextArg::Text(runtime.name.clone());
    let title = text("docker.title")?.with("runtime", name());
    let mut summary = text(state.key())?.with("runtime", name());
    let mut figures = Vec::new();
    if let Disk::Seen(sizes) = disk {
        if runtime.maximum == Maximum::Shown {
            figures.push(Figure { label: text("docker.figure.maximum")?, size: sizes.logical });
        }
        figures.push(Figure { label: text("docker.figure.on-disk")?, size: sizes.allocated });
    }
    let mut parts = Vec::new();
    if let State::Answered(usage) = state {
        let inside = sum(Part::ALL.map(|part| part.of(usage).size));
        let unused = sum(Part::ALL.map(|part| part.of(usage).unused));
        summary = summary.with("inside", TextArg::Size(inside)).with("unused", TextArg::Size(unused));
        figures.push(Figure { label: text("docker.figure.inside")?, size: inside });
        for part in Part::ALL {
            parts.push(insight_part(runtime, part, part.of(usage))?);
        }
    }
    let mut notes = Vec::new();
    if *disk == Disk::Unseen {
        notes.push(text("docker.note.disk-unseen")?);
    }
    if *state != State::Left {
        notes.push(text(&format!("docker.runtime.{}.gives-back", runtime.id))?);
    }
    if parts.iter().any(|part| part.command.is_some()) {
        notes.push(text("docker.note.never-runs")?);
    }
    Ok(Insight::new(&runtime.id, title, summary, figures, parts, notes)?)
}

/// One part of an answered card: its unused size, what it is, what freeing it costs and, when something is unused,
/// its command.
fn insight_part(runtime: &Runtime, part: Part, kind: &Kind) -> Result<InsightPart, ReportError> {
    let key = |field: &str| text(&format!("docker.part.{}.{field}", part.id()));
    let nothing = kind.unused == ByteSize::ZERO;
    let detail = match (nothing, part.counted()) {
        (true, _) => key("none")?,
        (false, true) => key("detail")?.with("count", TextArg::Count(kind.unused_count())),
        (false, false) => key("detail")?,
    };
    let command = if nothing {
        None
    } else {
        let args = ["--context", runtime.context.as_str()].into_iter().chain(part.command().iter().copied());
        Some(ShownCommand::new(ToolId::new("docker")?, args.map(str::to_owned).collect())?)
    };
    Ok(InsightPart {
        label: key("label")?,
        size: kind.unused,
        detail,
        effect: key("effect")?,
        danger: part.danger(),
        command,
    })
}

fn text(key: &str) -> Result<Text, TextKeyError> {
    TextKey::new(key).map(Text::new)
}

/// The sum of `sizes`, at most the largest size.
fn sum(sizes: [ByteSize; 4]) -> ByteSize {
    ByteSize::new(sizes.iter().fold(0_u64, |total, size| total.saturating_add(size.bytes())))
}
