//! Path context tracking and candidate path generation for inlining.
//! source: https://github.com/TedDriggs/syn-inline-mod/blob/master/src/mod_path.rs

use {
    std::path::{Path, PathBuf},
    syn::{Expr, ExprLit, Ident, ItemMod, Lit, Meta},
};

/// Extensions to the built-in `Path` type for the purpose of mod expansion.
trait ModPath {
    /// Check if the current file is a 2015-style mod file. If so, named mods should be
    /// resolved in the current directory. If not, we should check, in order:
    ///
    /// 1. `{fileStem}/{name}.rs`
    /// 2. `{fileStem}/{name}/mod.rs`
    fn is_mod_file(&self) -> bool;
}

impl ModPath for Path {
    fn is_mod_file(&self) -> bool {
        self.file_name().map(|s| s == "mod.rs").unwrap_or_default()
    }
}

/// The current mod path, including idents and explicit paths.
#[derive(Debug, Clone, Default)]
pub struct ModContext(Vec<ModSegment>);

impl ModContext {
    pub fn push(&mut self, value: ModSegment) {
        self.0.push(value);
    }

    pub fn pop(&mut self) -> Option<ModSegment> {
        self.0.pop()
    }

    /// Get the list of places a module's source code may appear relative to the current file
    /// location.
    pub fn relative_to(&self, base: &Path, root: bool) -> Vec<PathBuf> {
        let mut parent = base.to_path_buf();
        parent.pop();
        if root || base.is_mod_file() {
            self.to_path_bufs()
                .into_iter()
                .map(|end| parent.clone().join(end))
                .collect()
        } else {
            parent = parent.join(base.file_stem().unwrap());

            self.to_path_bufs()
                .into_iter()
                .map(|end| parent.clone().join(end))
                .collect()
        }
    }

    fn to_path_bufs(&self) -> Vec<PathBuf> {
        let mut buf = PathBuf::new();
        for item in &self.0 {
            buf.push(PathBuf::from(item.clone()));
        }

        // If the last term was an explicit path, there is only one valid interpretation
        // of this context as a file path.
        if !self.is_last_ident() {
            return vec![buf];
        }

        // If it was an ident, we need to look in both `foo.rs` and `foo/mod.rs`

        let mut inline = buf.clone();
        inline.set_extension("rs");

        vec![inline, buf.join("mod.rs")]
    }

    /// Checks if the last term in the context was a module identifier, rather
    /// than an explicit `path` attribute.
    fn is_last_ident(&self) -> bool {
        self.0.last().map(|seg| seg.is_ident()).unwrap_or_default()
    }
}

impl From<Vec<ModSegment>> for ModContext {
    fn from(segments: Vec<ModSegment>) -> Self {
        Self(segments)
    }
}

#[derive(Debug, Clone)]
pub enum ModSegment {
    Ident(Ident),
    Path(PathBuf),
}

impl ModSegment {
    /// Checks if the `self` mod segment was taken from the module identifier.
    pub fn is_ident(&self) -> bool {
        match self {
            ModSegment::Ident(_) => true,
            ModSegment::Path(_) => false,
        }
    }
}

#[cfg(test)]
impl ModSegment {
    pub(self) fn new_ident(ident: &'static str) -> Self {
        ModSegment::Ident(syn::Ident::new(ident, proc_macro2::Span::call_site()))
    }

    pub(self) fn new_path(path: &'static str) -> Self {
        ModSegment::Path(PathBuf::from(path))
    }
}

impl From<&ItemMod> for ModSegment {
    fn from(v: &ItemMod) -> Self {
        for attr in &v.attrs {
            if let Meta::NameValue(ref name_value) = attr.meta {
                if name_value.path.is_ident("path") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(ref path_value),
                        ..
                    }) = name_value.value
                    {
                        return ModSegment::Path(path_value.value().into());
                    }
                }
            }
        }

        ModSegment::Ident(v.ident.clone())
    }
}

impl From<&mut ItemMod> for ModSegment {
    fn from(v: &mut ItemMod) -> Self {
        ModSegment::from(&*v)
    }
}

impl From<ModSegment> for PathBuf {
    fn from(seg: ModSegment) -> Self {
        match seg {
            ModSegment::Path(buf) => buf,
            ModSegment::Ident(ident) => ident.to_string().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, std::path::Path};

    #[test]
    fn relative_to_lib() {
        let ctx = ModContext::from(vec![
            ModSegment::new_ident("threads"),
            ModSegment::new_ident("local"),
        ]);

        assert_eq!(
            ctx.relative_to(Path::new("/src/lib.rs"), true),
            vec![
                Path::new("/src/threads/local.rs"),
                Path::new("/src/threads/local/mod.rs"),
            ]
        );
    }

    #[test]
    fn relative_to_mod() {
        let ctx = ModContext::from(vec![
            ModSegment::new_ident("threads"),
            ModSegment::new_ident("local"),
        ]);

        assert_eq!(
            ctx.relative_to(Path::new("/src/runner/mod.rs"), false),
            vec![
                Path::new("/src/runner/threads/local.rs"),
                Path::new("/src/runner/threads/local/mod.rs"),
            ]
        );
    }

    /// Check that non-root modules and files not named 'mod.rs' have their file stem preserved
    /// in the search when generating candidate paths.
    // e.g. `mod runner` called from `/src/lib.rs`
    #[test]
    fn relative_to_2018_mod() {
        let ctx = ModContext::from(vec![
            ModSegment::new_ident("threads"),
            ModSegment::new_ident("local"),
        ]);

        assert_eq!(
            ctx.relative_to(Path::new("/src/runner.rs"), false),
            vec![
                Path::new("/src/runner/threads/local.rs"),
                Path::new("/src/runner/threads/local/mod.rs"),
            ]
        );
    }

    /// Check that root modules are not part of the directory
    /// when generating candidate paths.
    // e.g. `rustc` is invoked with `src/runner.rs` as the input
    #[test]
    fn relative_to_non_standard_root() {
        let ctx = ModContext::from(vec![
            ModSegment::new_ident("threads"),
            ModSegment::new_ident("local"),
        ]);

        assert_eq!(
            ctx.relative_to(Path::new("/src/runner.rs"), true),
            vec![
                Path::new("/src/threads/local.rs"),
                Path::new("/src/threads/local/mod.rs"),
            ]
        );
    }

    /// Check that a full chain of explicit file names works produces exactly one candidate file with
    /// the correct absolute path.
    #[test]
    fn relative_to_paths() {
        let ctx = ModContext::from(vec![
            ModSegment::new_path("threads"),
            ModSegment::new_path("tls.rs"),
        ]);

        assert_eq!(
            ctx.relative_to(Path::new("/src/lib.rs"), true),
            vec![Path::new("/src/threads/tls.rs")]
        );
    }

    /// Check that a path is honored, but an inner ident still generates multiple possibilities.
    #[test]
    fn relative_to_path_around_ident() {
        let ctx = ModContext::from(vec![
            ModSegment::new_path("threads"),
            ModSegment::new_ident("tls"),
        ]);

        assert_eq!(
            ctx.relative_to(Path::new("/src/lib.rs"), true),
            vec![
                Path::new("/src/threads/tls.rs"),
                Path::new("/src/threads/tls/mod.rs"),
            ]
        );
    }
}
