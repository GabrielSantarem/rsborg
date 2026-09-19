use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatus {
    /// Explicitly selected by user to be included
    ExplicitInclude,
    /// Explicitly selected by user to be excluded (--exclude)
    ExplicitExclude,
    /// Included because a parent directory is included
    InheritedInclude,
    /// Excluded because a parent directory is excluded
    InheritedExclude,
    /// Not included
    Neutral,
}

#[derive(Clone)]
pub struct FsEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_parent_link: bool,
}

pub struct FileBrowser {
    pub current_dir: PathBuf,
    pub entries: Vec<FsEntry>,
    pub selected_index: usize,
    pub explicit_includes: HashSet<PathBuf>,
    pub explicit_excludes: HashSet<PathBuf>,
}

impl FileBrowser {
    pub fn new() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));

        let mut fb = Self {
            current_dir: home,
            entries: Vec::new(),
            selected_index: 0,
            explicit_includes: HashSet::new(),
            explicit_excludes: HashSet::new(),
        };
        fb.load_entries();
        fb
    }

    pub fn load_entries(&mut self) {
        self.entries.clear();
        self.selected_index = 0;

        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(FsEntry {
                path: parent.to_path_buf(),
                name: ".. (Diretório anterior)".to_string(),
                is_dir: true,
                is_parent_link: true,
            });
        }

        let mut dirs = vec![];
        let mut files = vec![];

        if let Ok(read_dir) = std::fs::read_dir(&self.current_dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();

                if name.starts_with('.') && name != ".rsborg" {
                    continue;
                }

                let is_dir = path.is_dir();
                let fs_entry = FsEntry {
                    path,
                    name,
                    is_dir,
                    is_parent_link: false,
                };

                if is_dir {
                    dirs.push(fs_entry);
                } else {
                    files.push(fs_entry);
                }
            }
        }

        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        files.sort_by(|a, b| a.name.cmp(&b.name));

        self.entries.extend(dirs);
        self.entries.extend(files);
    }

    pub fn next(&mut self) {
        if !self.entries.is_empty() {
            if self.selected_index >= self.entries.len().saturating_sub(1) {
                self.selected_index = 0;
            } else {
                self.selected_index += 1;
            }
        }
    }

    pub fn previous(&mut self) {
        if !self.entries.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.entries.len().saturating_sub(1);
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn enter_dir(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index)
            && entry.is_dir {
                self.current_dir = entry.path.clone();
                self.load_entries();
            }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.load_entries();
        }
    }

    pub fn get_status(&self, path: &Path) -> ItemStatus {
        if self.explicit_excludes.contains(path) {
            return ItemStatus::ExplicitExclude;
        }
        if self.explicit_includes.contains(path) {
            return ItemStatus::ExplicitInclude;
        }

        let mut curr = path.parent();
        while let Some(parent) = curr {
            if self.explicit_excludes.contains(parent) {
                return ItemStatus::InheritedExclude;
            }
            if self.explicit_includes.contains(parent) {
                return ItemStatus::InheritedInclude;
            }
            curr = parent.parent();
        }

        ItemStatus::Neutral
    }

    pub fn toggle_selection(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_parent_link {
                return;
            }

            let path = entry.path.clone();
            let status = self.get_status(&path);

            match status {
                ItemStatus::InheritedInclude => {
                    self.explicit_excludes.insert(path);
                }
                ItemStatus::ExplicitExclude => {
                    self.explicit_excludes.remove(&path);
                }
                ItemStatus::Neutral | ItemStatus::InheritedExclude => {
                    self.explicit_excludes.remove(&path);
                    self.explicit_includes.insert(path);
                }
                ItemStatus::ExplicitInclude => {
                    self.explicit_includes.remove(&path);
                }
            }
        }
    }

    pub fn get_final_paths(&self) -> (Vec<String>, Vec<String>) {
        let includes: Vec<String> = self
            .explicit_includes
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();

        let excludes: Vec<String> = self
            .explicit_excludes
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();

        (includes, excludes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neutral_status_by_default() {
        let fb = FileBrowser::new();
        let path = PathBuf::from("/home/user/docs/file.txt");
        assert_eq!(fb.get_status(&path), ItemStatus::Neutral);
    }

    #[test]
    fn test_explicit_include_and_inheritance() {
        let mut fb = FileBrowser::new();
        let parent = PathBuf::from("/home/user/docs");
        let child = PathBuf::from("/home/user/docs/file.txt");
        let deep_child = PathBuf::from("/home/user/docs/sub/folder/file.pdf");
        let unrelated = PathBuf::from("/home/user/downloads/video.mp4");

        fb.explicit_includes.insert(parent.clone());

        assert_eq!(fb.get_status(&parent), ItemStatus::ExplicitInclude);
        assert_eq!(fb.get_status(&child), ItemStatus::InheritedInclude);
        assert_eq!(fb.get_status(&deep_child), ItemStatus::InheritedInclude);
        assert_eq!(fb.get_status(&unrelated), ItemStatus::Neutral);
    }

    #[test]
    fn test_exclude_inside_included_parent_and_override() {
        let mut fb = FileBrowser::new();
        let parent = PathBuf::from("/home/user/docs");
        let excluded_child = PathBuf::from("/home/user/docs/temp");
        let inside_excluded = PathBuf::from("/home/user/docs/temp/cache.dat");

        fb.explicit_includes.insert(parent.clone());
        fb.explicit_excludes.insert(excluded_child.clone());

        assert_eq!(fb.get_status(&parent), ItemStatus::ExplicitInclude);
        assert_eq!(fb.get_status(&excluded_child), ItemStatus::ExplicitExclude);
        assert_eq!(
            fb.get_status(&inside_excluded),
            ItemStatus::InheritedExclude
        );

        // Edge case: Agora re-inclui explicitamente um arquivo dentro da pasta excluída!
        let re_included = inside_excluded.clone();
        fb.explicit_includes.insert(re_included.clone());
        assert_eq!(fb.get_status(&re_included), ItemStatus::ExplicitInclude);
    }

    #[test]
    fn test_toggle_cycle_logic() {
        let mut fb = FileBrowser::new();
        fb.entries.clear(); // Limpa as entradas carregadas do HOME

        let entry_path = PathBuf::from("/tmp/test_dir");
        fb.entries.push(FsEntry {
            path: entry_path.clone(),
            name: "test_dir".to_string(),
            is_dir: true,
            is_parent_link: false,
        });
        fb.selected_index = 0;

        // 1. Estado inicial Neutro -> Aperta Espaço -> Deve virar ExplicitInclude
        assert_eq!(fb.get_status(&entry_path), ItemStatus::Neutral);
        fb.toggle_selection();
        assert_eq!(fb.get_status(&entry_path), ItemStatus::ExplicitInclude);

        // 2. Aperta Espaço novamente -> Deve voltar para Neutro
        fb.toggle_selection();
        assert_eq!(fb.get_status(&entry_path), ItemStatus::Neutral);
    }

    #[test]
    fn test_toggle_inherited_item_to_excluded() {
        let mut fb = FileBrowser::new();
        fb.entries.clear(); // Limpa as entradas carregadas do HOME

        let parent = PathBuf::from("/tmp/parent");
        let child = PathBuf::from("/tmp/parent/child.txt");

        fb.explicit_includes.insert(parent);

        fb.entries.push(FsEntry {
            path: child.clone(),
            name: "child.txt".to_string(),
            is_dir: false,
            is_parent_link: false,
        });
        fb.selected_index = 0;

        // Está herdando inclusão
        assert_eq!(fb.get_status(&child), ItemStatus::InheritedInclude);

        // Usuário desmarca o filho -> Deve virar ExplicitExclude
        fb.toggle_selection();
        assert_eq!(fb.get_status(&child), ItemStatus::ExplicitExclude);

        // Usuário aperta espaço de novo -> Remove a exclusão e volta a herdar
        fb.toggle_selection();
        assert_eq!(fb.get_status(&child), ItemStatus::InheritedInclude);
    }

    #[test]
    fn test_parent_link_cannot_be_selected() {
        let mut fb = FileBrowser::new();
        fb.entries.clear();
        let parent = PathBuf::from("/home/user");
        fb.entries.push(FsEntry {
            path: parent,
            name: ".. (Diretório anterior)".to_string(),
            is_dir: true,
            is_parent_link: true,
        });
        fb.selected_index = 0;

        fb.toggle_selection();
        assert!(fb.explicit_includes.is_empty());
        assert!(fb.explicit_excludes.is_empty());
    }

    #[test]
    fn test_empty_browser_navigation_no_panic() {
        let mut fb = FileBrowser::new();
        fb.entries.clear();

        // Edge case: navegar em lista vazia não pode dar panic por out of bounds ou underflow
        fb.next();
        assert_eq!(fb.selected_index, 0);
        fb.previous();
        assert_eq!(fb.selected_index, 0);
    }

    #[test]
    fn test_root_directory_parent_edge_case() {
        let mut fb = FileBrowser::new();
        fb.current_dir = PathBuf::from("/");
        fb.load_entries();

        // No root "/", parent() é None, então não deve existir entrada ".."
        assert!(!fb.entries.iter().any(|e| e.is_parent_link));

        // go_up() no root não deve mudar o diretório nem entrar em loop
        fb.go_up();
        assert_eq!(fb.current_dir, PathBuf::from("/"));
    }
}
