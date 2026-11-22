use std::path::{Path, PathBuf};

use fresher_core::{ByteSize, CategoryId, ModuleId, NodeId, NodeInfo, NodeKind, Offer, Sizes, StateId, Withheld};

use super::{Above, Ignored, Judged, judge};

const LIBRARY: &str = "/home/Library";
const TRASH: &str = "/home/.Trash";

/// A scanned node `name` of `kind` at `path`, of 10 complete bytes, claimed by no one.
fn node(path: &str, kind: NodeKind) -> (NodeInfo, PathBuf) {
    let path = PathBuf::from(path);
    let name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let info = NodeInfo {
        id: NodeId::new(StateId::new(1), 1),
        name,
        kind,
        sizes: Sizes::complete(ByteSize::new(10), ByteSize::new(10)),
        files: 0,
        folders: 0,
        owner: None,
        modified: None,
        not_scanned: None,
    };
    (info, path)
}

/// The judgement of `node` at `path` under the home `/home`, protection letting it go, nothing explained or ignored,
/// after `change` set what the test needs.
fn judged(node: &NodeInfo, path: &Path, change: impl FnOnce(&mut Judged<'_>)) -> (Offer, bool) {
    let mut input = Judged {
        node,
        path,
        above: Above::of(Path::new("/home"), path),
        library: Some(Path::new(LIBRARY)),
        trash: Some(Path::new(TRASH)),
        protection: Ok(()),
        holds_claimed: false,
        ignored: Ignored::No,
    };
    change(&mut input);
    judge(&input)
}

fn withheld(reason: Withheld, opens: bool) -> (Offer, bool) {
    (Offer::Withheld(reason), opens)
}

mod negative {
    use std::path::Path;

    use fresher_core::{Completeness, NodeKind, NotScanned, Protected, Withheld};

    use super::{LIBRARY, TRASH, category, judged, node, withheld};
    use crate::largest::judge::{Above, Ignored};

    #[test]
    fn a_claimed_node_is_explained() {
        let (mut info, path) = node("/home/Library", NodeKind::Folder);
        info.owner = Some(category());

        assert_eq!(judged(&info, &path, |_| {}), withheld(Withheld::Explained, false));
    }

    #[test]
    fn what_is_inside_a_claimed_folder_is_explained() {
        let (folder, folder_path) = node("/home/Caches/inner", NodeKind::Folder);
        let (file, file_path) = node("/home/Caches/inner/blob.bin", NodeKind::File);
        let claimed_above = |input: &mut super::Judged<'_>| input.above = Above::Claimed;

        assert_eq!(judged(&folder, &folder_path, claimed_above), withheld(Withheld::Explained, false));
        assert_eq!(judged(&file, &file_path, claimed_above), withheld(Withheld::Explained, false));
    }

    #[test]
    fn a_file_inside_a_package_is_withheld() {
        let (file, file_path) = node("/home/Pictures/Trip.photoslibrary/sub/x.heic", NodeKind::File);
        let (folder, folder_path) = node("/home/Pictures/Trip.photoslibrary/sub", NodeKind::Folder);

        assert_eq!(judged(&file, &file_path, |_| {}), withheld(Withheld::Hidden, false));
        assert_eq!(judged(&folder, &folder_path, |_| {}), withheld(Withheld::Hidden, false));
    }

    #[test]
    fn the_library_is_withheld_and_does_not_open() {
        let (info, path) = node(LIBRARY, NodeKind::Folder);
        let (inside, inner) = node("/home/Library/Caches/big.bin", NodeKind::File);

        let standard = judged(&info, &path, |input| input.protection = Err(Protected::WholeFolder));

        assert_eq!(standard, withheld(Withheld::Library, false));
        assert_eq!(judged(&inside, &inner, |_| {}), withheld(Withheld::Library, false));
    }

    #[test]
    fn the_trash_is_withheld() {
        let (info, path) = node(TRASH, NodeKind::Folder);

        assert_eq!(judged(&info, &path, |_| {}), withheld(Withheld::Trash, false));
    }

    #[test]
    fn a_hidden_folder_and_everything_in_it_are_withheld() {
        let (folder, folder_path) = node("/home/.gradle", NodeKind::Folder);
        let (inside, inside_path) = node("/home/.gradle/caches/big.jar", NodeKind::File);

        assert_eq!(judged(&folder, &folder_path, |_| {}), withheld(Withheld::Hidden, false));
        assert_eq!(judged(&inside, &inside_path, |_| {}), withheld(Withheld::Hidden, false));
    }

    #[test]
    fn an_app_is_withheld() {
        let (app, app_path) = node("/home/Applications/Tool.app", NodeKind::Folder);
        let (inside, inside_path) = node("/home/Applications/Tool.app/Contents/big.bin", NodeKind::File);

        assert_eq!(judged(&app, &app_path, |_| {}), withheld(Withheld::App, false));
        assert_eq!(judged(&inside, &inside_path, |_| {}), withheld(Withheld::App, false));
    }

    #[test]
    fn a_link_is_withheld() {
        let (info, path) = node("/home/Movies/latest.mov", NodeKind::Link);

        assert_eq!(judged(&info, &path, |_| {}), withheld(Withheld::Link, false));
    }

    #[test]
    fn a_folder_not_scanned_is_not_seen() {
        let (mut info, path) = node("/home/Movies/TV", NodeKind::Folder);
        info.not_scanned = Some(NotScanned::Avoided);

        assert_eq!(judged(&info, &path, |_| {}), withheld(Withheld::NotSeen, false));
    }

    #[test]
    fn a_standard_folder_opens_but_is_withheld() {
        let (info, path) = node("/home/Movies", NodeKind::Folder);

        let judgement = judged(&info, &path, |input| input.protection = Err(Protected::WholeFolder));

        assert_eq!(judgement, withheld(Withheld::StandardFolder, true));
    }

    #[test]
    fn a_protected_place_is_withheld() {
        let (info, path) = node("/home/Work", NodeKind::Folder);

        for protection in [Protected::Never, Protected::Ancestor] {
            let judgement = judged(&info, &path, |input| input.protection = Err(protection));

            assert_eq!(judgement, withheld(Withheld::Protected, false), "{protection:?}");
        }
    }

    #[test]
    fn a_folder_holding_explained_bytes_opens_but_is_withheld() {
        let (folder, folder_path) = node("/home/Projects", NodeKind::Folder);
        let (package, package_path) = node("/home/Projects/Song.logicx", NodeKind::Folder);

        assert_eq!(
            judged(&folder, &folder_path, |input| input.holds_claimed = true),
            withheld(Withheld::HoldsExplained, true)
        );
        assert_eq!(
            judged(&package, &package_path, |input| input.holds_claimed = true),
            withheld(Withheld::HoldsExplained, false)
        );
    }

    #[test]
    fn a_folder_read_in_part_is_withheld() {
        let (mut info, path) = node("/home/Shared", NodeKind::Folder);
        info.sizes.completeness = Completeness::Partial;

        assert_eq!(judged(&info, &path, |_| {}), withheld(Withheld::ReadInPart, true));
    }

    #[test]
    fn an_ignored_place_is_withheld() {
        let (info, path) = node("/home/Archive/old.zip", NodeKind::File);
        let (folder, folder_path) = node("/home/Archive", NodeKind::Folder);

        assert_eq!(
            judged(&info, &path, |input| input.ignored = Ignored::AtOrAbove),
            withheld(Withheld::Ignored, false)
        );
        assert_eq!(
            judged(&folder, &folder_path, |input| input.ignored = Ignored::AtOrAbove),
            withheld(Withheld::Ignored, false)
        );
        assert_eq!(
            judged(&folder, &folder_path, |input| input.ignored = Ignored::Inside),
            withheld(Withheld::Ignored, true)
        );
        assert_eq!(
            judged(&info, &path, |input| input.protection = Err(Protected::Ignored)),
            withheld(Withheld::Ignored, false)
        );
    }

    #[test]
    fn the_folders_above_a_path_outside_the_home_say_nothing() {
        assert_eq!(Above::of(Path::new("/home"), Path::new("/elsewhere/.x/y")), Above::Plain);
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{NodeKind, Offer};

    use super::{Above, judged, node};

    #[test]
    fn the_folders_above_name_what_they_make_of_a_row() {
        let above = |path: &str| Above::of(Path::new("/home"), Path::new(path));

        assert_eq!(above("/home/a/b.txt"), Above::Plain);
        assert_eq!(above("/home/Tool.app/Contents/x"), Above::App);
        assert_eq!(above("/home/Tool.app/.git/x"), Above::App);
        assert_eq!(above("/home/Tool.app/Contents/Song.logicx/x"), Above::App);
        assert_eq!(above("/home/.tools/Song.logicx/x"), Above::Hidden);
        assert_eq!(above("/home/Song.logicx/Media/x.wav"), Above::Hidden);
        assert_eq!(above("/home/Song.logicx"), Above::Plain);
    }

    #[test]
    fn a_plain_file_is_offered() {
        let (info, path) = node("/home/Downloads/movie.mov", NodeKind::File);

        assert_eq!(judged(&info, &path, |_| {}), (Offer::Offered, false));
    }

    #[test]
    fn a_plain_folder_is_offered_and_opens() {
        let (info, path) = node("/home/Projects/old", NodeKind::Folder);

        assert_eq!(judged(&info, &path, |_| {}), (Offer::Offered, true));
    }

    #[test]
    fn a_package_is_offered_and_does_not_open() {
        let (info, path) = node("/home/Pictures/Trip.photoslibrary", NodeKind::Folder);

        assert_eq!(judged(&info, &path, |_| {}), (Offer::Offered, false));
    }
}

fn category() -> CategoryId {
    CategoryId::new(ModuleId::new("alpha").unwrap(), "alpha.caches").unwrap()
}
