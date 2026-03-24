use std::path::{Path, PathBuf};

use crate::core::wallpaper;
use crate::error::{QueryError, SwpError};

#[derive(Debug)]
pub enum SetInputResolution {
    Resolved(PathBuf),
    MultipleMatches { query: String, matches: Vec<PathBuf> },
}

pub fn resolve_set_input(input: &str) -> Result<SetInputResolution, SwpError> {
    let dir = wallpaper::wallpaper_dir()?;
    resolve_set_input_in_dir(&dir, input)
}

pub fn resolve_set_input_in_dir(dir: &Path, input: &str) -> Result<SetInputResolution, SwpError> {
    let literal = PathBuf::from(input);
    if literal.exists() {
        return Ok(SetInputResolution::Resolved(literal));
    }

    let mut matches = wallpaper::find_by_words(dir, input);

    match matches.len() {
        0 => Err(QueryError::NoMatches {
            query: input.to_string(),
        }
        .into()),
        1 => Ok(SetInputResolution::Resolved(matches.remove(0))),
        _ => {
            matches.sort();
            Ok(SetInputResolution::MultipleMatches {
                query: input.to_string(),
                matches,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{resolve_set_input_in_dir, SetInputResolution};
    use crate::error::{QueryError, SwpError};

    fn test_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("swp-query-tests-{stamp}"));
        fs::create_dir_all(&dir).expect("temp test dir should be created");
        dir
    }

    #[test]
    fn resolve_uses_literal_path_when_it_exists() {
        let dir = test_dir();
        let file = dir.join("literal.png");
        fs::write(&file, b"x").expect("test file should be writable");

        let result = resolve_set_input_in_dir(&dir, file.to_str().expect("utf8 path"))
            .expect("literal path should resolve");

        match result {
            SetInputResolution::Resolved(path) => assert_eq!(path, file),
            SetInputResolution::MultipleMatches { .. } => {
                panic!("expected resolved path, got multiple matches")
            }
        }

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }

    #[test]
    fn resolve_returns_error_for_no_matches() {
        let dir = test_dir();

        let err = resolve_set_input_in_dir(&dir, "does-not-exist")
            .expect_err("non-matching query should fail");
        match err {
            SwpError::Query(QueryError::NoMatches { query }) => {
                assert_eq!(query, "does-not-exist")
            }
            other => panic!("unexpected error variant: {other}"),
        }

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }

    #[test]
    fn resolve_returns_single_match() {
        let dir = test_dir();
        let file = dir.join("hello-world.png");
        fs::write(&file, b"x").expect("test file should be writable");

        let result = resolve_set_input_in_dir(&dir, "hello world")
            .expect("matching query should resolve");

        match result {
            SetInputResolution::Resolved(path) => assert_eq!(path, file),
            SetInputResolution::MultipleMatches { .. } => {
                panic!("expected single resolved match")
            }
        }

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }

    #[test]
    fn resolve_returns_sorted_multiple_matches() {
        let dir = test_dir();
        let a = dir.join("angel-a.jpg");
        let b = dir.join("angel-b.jpg");
        fs::write(&b, b"x").expect("test file should be writable");
        fs::write(&a, b"x").expect("test file should be writable");

        let result = resolve_set_input_in_dir(&dir, "angel")
            .expect("query with many matches should resolve to disambiguation");

        match result {
            SetInputResolution::Resolved(_) => panic!("expected multiple matches"),
            SetInputResolution::MultipleMatches { query, matches } => {
                assert_eq!(query, "angel");
                assert_eq!(matches, vec![a, b]);
            }
        }

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }
}