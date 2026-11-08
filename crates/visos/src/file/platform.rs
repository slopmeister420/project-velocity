//! platform layer replacing the xbox file api (FILE_Open/FILE_GetInfo over `d:\`).
//! game paths are xbox style (`\Scenery\SCENERYM.VCD`, `d:\fe\fe.vcd`); they resolve case-insensitively against
//! the game dir (the xbox `d:\`), found in this order (deployment rule, see CLAUDE.md):
//! `--game <dir>` → `VELOCITY_GAME_DIR` → the exe's own directory if it holds `default.xbe` → error.
//! nothing here ever writes into the game dir.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// `FILE_MAX_PATHNAME`
pub const FILE_MAX_PATHNAME: usize = 256;

static GAME_DIR: OnceLock<Result<PathBuf, GameDirError>> = OnceLock::new();

#[derive(Clone, Debug, thiserror::Error)]
#[error(
    "game data not found: pass --game <dir>, set VELOCITY_GAME_DIR, or put the exe next to default.xbe \
     (checked --game {arg:?}, VELOCITY_GAME_DIR {env:?}, exe dir {exe:?})"
)]
pub struct GameDirError {
    pub arg: Option<PathBuf>,
    pub env: Option<PathBuf>,
    pub exe: Option<PathBuf>,
}

/// value of `--game <dir>` / `--game=<dir>` in an argument list.
pub fn game_arg<I: IntoIterator<Item = String>>(args: I) -> Option<PathBuf> {
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        if a == "--game" {
            return it.next().map(PathBuf::from);
        }
        if let Some(v) = a.strip_prefix("--game=") {
            return Some(PathBuf::from(v));
        }
    }
    None
}

fn is_game_dir(p: &Path) -> bool {
    p.join("default.xbe").is_file()
}

fn resolve_game_dir(arg: Option<PathBuf>) -> Result<PathBuf, GameDirError> {
    let env = std::env::var_os("VELOCITY_GAME_DIR").map(PathBuf::from);
    let exe = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf));
    if let Some(a) = arg.as_ref().filter(|a| a.is_dir()) {
        return Ok(a.clone());
    }
    if let Some(e) = env.as_ref().filter(|e| e.is_dir()) {
        return Ok(e.clone());
    }
    if let Some(x) = &exe {
        if is_game_dir(x) {
            return Ok(x.clone());
        }
    }
    Err(GameDirError { arg, env, exe })
}

/// fix the game dir from the process arguments (`--game`). call once at startup before any read; later calls
/// return the first result.
pub fn init_game_dir_from_args() -> Result<&'static Path, GameDirError> {
    init_game_dir(game_arg(std::env::args().skip(1)))
}

/// fix the game dir with an explicit `--game` value (None = the other sources only).
pub fn init_game_dir(arg: Option<PathBuf>) -> Result<&'static Path, GameDirError> {
    GAME_DIR.get_or_init(|| resolve_game_dir(arg)).as_ref().map(PathBuf::as_path).map_err(Clone::clone)
}

/// the resolved game dir or the resolution error.
pub fn try_game_dir() -> Result<&'static Path, GameDirError> {
    init_game_dir(None)
}

/// root of the game data. unresolved: logs the error once and returns a path that does not exist, so reads fail
/// with not-found and tests checking `game_dir().exists()` skip.
pub fn game_dir() -> &'static Path {
    match try_game_dir() {
        Ok(p) => p,
        Err(e) => {
            static LOGGED: OnceLock<()> = OnceLock::new();
            LOGGED.get_or_init(|| log::error!("{e}"));
            Path::new("<no game dir>")
        }
    }
}

/// split an xbox path into components, dropping a drive prefix and empty parts.
pub fn components(path: &str) -> impl Iterator<Item = &str> {
    let p = match path.as_bytes() {
        [d, b':', ..] if d.is_ascii_alphabetic() => &path[2..],
        _ => path,
    };
    p.split(['\\', '/']).filter(|c| !c.is_empty() && *c != ".")
}

/// resolve a game path to a host path, matching each component case-insensitively.
/// returns the best-effort joined path even if it does not exist.
pub fn resolve(path: &str) -> PathBuf {
    resolve_in(game_dir(), path)
}

/// `resolve` against an explicit root.
pub fn resolve_in(root: &Path, path: &str) -> PathBuf {
    let mut cur = root.to_path_buf();
    for c in components(path) {
        if c == ".." {
            if cur != root {
                cur.pop();
            }
            continue;
        }
        let direct = cur.join(c);
        if direct.exists() {
            cur = direct;
            continue;
        }
        let found = std::fs::read_dir(&cur).ok().and_then(|rd| {
            rd.flatten().map(|e| e.file_name()).find(|n| n.to_str().is_some_and(|s| s.eq_ignore_ascii_case(c)))
        });
        cur = match found {
            Some(n) => cur.join(n),
            None => direct,
        };
    }
    cur
}

/// FILE_GetInfo (0x0029B560) reduced to "does it exist".
pub fn exists(path: &str) -> bool {
    resolve(path).exists()
}

/// read a whole game file (FILE_LoadWholeFile, 0x0029D1E0).
pub fn read(path: &str) -> std::io::Result<Vec<u8>> {
    std::fs::read(resolve(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_arg_forms() {
        let a = |v: &[&str]| game_arg(v.iter().map(|s| s.to_string()));
        assert_eq!(a(&["x", "--game", "D:\\g"]), Some(PathBuf::from("D:\\g")));
        assert_eq!(a(&["--game=E:\\h"]), Some(PathBuf::from("E:\\h")));
        assert_eq!(a(&["--track", "x"]), None);
    }
}
