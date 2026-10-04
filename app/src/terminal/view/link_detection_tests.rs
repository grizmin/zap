use std::iter;

use warp_util::path::CleanPathResult;

use super::super::TerminalView;
use super::{path_without_trailing_sentence_period, GridHighlightedLink};
use crate::terminal::model::grid::grid_handler::PossiblePath;
use crate::terminal::model::index::Point;
use crate::terminal::model::terminal_model::WithinModel;
use crate::util::file::LinkValidationContext;

/// Builds a single AltScreen candidate covering `token` at row 0, cols 0..len.
fn candidate_for(token: &str) -> WithinModel<PossiblePath> {
    let end_col = token.chars().count() - 1;
    WithinModel::AltScreen(PossiblePath {
        path: CleanPathResult {
            path: token.into(),
            line_and_column_num: None,
        },
        range: Point { row: 0, col: 0 }..=Point { row: 0, col: end_col },
    })
}

#[test]
fn strips_only_sentence_periods() {
    // A trailing period after a real file name is sentence punctuation.
    assert_eq!(
        path_without_trailing_sentence_period("notes/README.md."),
        Some("notes/README.md")
    );
    assert_eq!(
        path_without_trailing_sentence_period(".gitignore."),
        Some(".gitignore")
    );
    assert_eq!(
        path_without_trailing_sentence_period("C:/Users/c/warp-md-test.md."),
        Some("C:/Users/c/warp-md-test.md")
    );

    // No trailing period -> nothing to trim.
    assert_eq!(
        path_without_trailing_sentence_period("notes/README.md"),
        None
    );

    // `.`/`..` path components must be preserved, not treated as punctuation.
    assert_eq!(path_without_trailing_sentence_period("."), None);
    assert_eq!(path_without_trailing_sentence_period(".."), None);
    assert_eq!(path_without_trailing_sentence_period("foo/."), None);
    assert_eq!(path_without_trailing_sentence_period("foo/.."), None);
    assert_eq!(path_without_trailing_sentence_period("foo.."), None);
}

// Regression test for https://github.com/warpdotdev/warp/issues/11477:
// a `.md` path at the end of a sentence captured the trailing period, so the
// resolved file and the highlight range ended in `.md.` and the file failed
// markdown classification. The trailing period must be excluded from both.
#[test]
fn compute_valid_paths_excludes_trailing_sentence_period() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("warp-md-test.md");
    std::fs::write(&file, "# Hello\n").unwrap();

    // The captured token as it would appear in `Drafted at <abs path>.`
    let token = format!("{}.", file.to_string_lossy());
    let candidate = candidate_for(&token);

    let link = TerminalView::compute_valid_paths(
        dir.path().to_str().unwrap(),
        iter::once(candidate),
        1000,
        None,
        LinkValidationContext::Local,
    )
    .expect("the markdown file should be detected as a link");

    let GridHighlightedLink::File(file_link) = link else {
        panic!("expected a file link");
    };
    let file_link = file_link.get_inner();

    // The resolved file excludes the trailing period (so it classifies as `.md`)...
    assert_eq!(
        file_link.absolute_path.file_name().unwrap(),
        "warp-md-test.md"
    );
    // ...and the highlighted range stops before the trailing period.
    assert_eq!(
        *file_link.link.range.end(),
        Point {
            row: 0,
            col: token.chars().count() - 2,
        }
    );
}

/// Plain relative paths must keep resolving against the working directory —
/// the core behavior behind clickable file links in command output.
#[test]
fn compute_valid_paths_resolves_plain_relative_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("hello.txt"), "hi\n").unwrap();

    let link = TerminalView::compute_valid_paths(
        dir.path().to_str().unwrap(),
        iter::once(candidate_for("hello.txt")),
        1000,
        None,
        LinkValidationContext::Local,
    )
    .expect("a plain relative file in cwd must be detected as a link");

    let GridHighlightedLink::File(file_link) = link else {
        panic!("expected a file link");
    };
    assert_eq!(
        file_link.get_inner().absolute_path.file_name().unwrap(),
        "hello.txt"
    );
}

/// Directories resolve too, as long as no line/column suffix is attached.
#[test]
fn compute_valid_paths_resolves_directory_in_cwd() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();

    let link = TerminalView::compute_valid_paths(
        dir.path().to_str().unwrap(),
        iter::once(candidate_for("sub")),
        1000,
        None,
        LinkValidationContext::Local,
    );

    assert!(link.is_some(), "`sub/` exists in cwd but was not linked");
}

/// Nonexistent paths must NOT produce a link (validation still filters).
#[test]
fn compute_valid_paths_rejects_nonexistent_file() {
    let dir = tempfile::tempdir().unwrap();

    let link = TerminalView::compute_valid_paths(
        dir.path().to_str().unwrap(),
        iter::once(candidate_for("does-not-exist.txt")),
        1000,
        None,
        LinkValidationContext::Local,
    );

    assert!(link.is_none(), "nonexistent file must not be linked");
}
