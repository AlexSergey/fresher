use std::path::{Path, PathBuf};

use fresher_core::{Access, AccessProbe, AccessStatus, GuardedPlace, expand_home};
use fresher_testing::FakeAccess;

/// The access a sandbox's world states, as sandbox mode and the autotest ask it: the world's answers, with the scan's
/// roots read as the walk reads them, so a root `~` is the sandbox's home and its stated folders are decided there.
#[derive(Debug)]
pub struct StatedAccess {
    access: FakeAccess,
    home: PathBuf,
}

impl StatedAccess {
    pub(crate) fn new(access: FakeAccess, home: &Path) -> Self {
        Self { access, home: home.to_path_buf() }
    }
}

impl AccessProbe for StatedAccess {
    fn status(&self) -> AccessStatus {
        self.access.status()
    }

    fn guarded_places(&self, roots: &[PathBuf]) -> Vec<GuardedPlace> {
        let roots: Vec<PathBuf> = roots.iter().map(|root| expand_home(root, &self.home)).collect();
        self.access.guarded_places(&roots)
    }

    fn try_place(&self, place: &GuardedPlace) -> Access {
        self.access.try_place(place)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use fresher_core::{AccessProbe, Capability, GuardedFolder, GuardedPlace};
    use fresher_testing::FakeAccess;

    use super::StatedAccess;

    const DOCUMENTS: Capability = Capability::Folder(GuardedFolder::Documents);

    fn access() -> StatedAccess {
        let home = Path::new("/box/home");
        StatedAccess::new(FakeAccess::with(&[]).home(home).guard("Documents", DOCUMENTS), home)
    }

    mod negative {
        use super::{AccessProbe, PathBuf, access};

        #[test]
        fn a_root_beside_the_home_guards_nothing() {
            for root in ["/box/other", "~other", "/elsewhere/~"] {
                assert_eq!(access().guarded_places(&[PathBuf::from(root)]), [], "{root}");
            }
        }
    }

    mod positive {
        use super::{AccessProbe, DOCUMENTS, GuardedPlace, PathBuf, access};

        #[test]
        fn a_root_written_with_a_tilde_is_the_sandbox_s_home() {
            let guarded = [GuardedPlace { path: PathBuf::from("/box/home/Documents"), capability: DOCUMENTS }];

            for root in ["~", "/box/home", "~/Documents"] {
                assert_eq!(access().guarded_places(&[PathBuf::from(root)]), guarded, "{root}");
            }
        }
    }
}
