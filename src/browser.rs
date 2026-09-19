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

        // Se o diretório atual tem pai, adiciona a entrada ".." no topo
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

                // Pular arquivos/pastas ocultas por padrão
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
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.load_entries();
            }
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.load_entries();
        }
    }

    /// Calcula o status de inclusão/exclusão de um caminho considerando a hierarquia
    pub fn get_status(&self, path: &Path) -> ItemStatus {
        if self.explicit_excludes.contains(path) {
            return ItemStatus::ExplicitExclude;
        }
        if self.explicit_includes.contains(path) {
            return ItemStatus::ExplicitInclude;
        }

        // Verifica os ancestrais (pais)
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

    /// Alterna a seleção do item sob o cursor com suporte a hierarquia:
    /// - Se herdado de pasta incluída: apertar Space marca como Exclusão Explícita `[-]`
    /// - Se excluído explicitamente: apertar Space desfaz a exclusão (volta a herdar `[✓]` ou `[ ]`)
    /// - Se neutro: apertar Space marca como Inclusão Explícita `[+]`
    /// - Se incluído explicitamente: apertar Space desfaz a inclusão `[ ]`
    pub fn toggle_selection(&mut self) {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_parent_link {
                return; // Não seleciona a entrada ".."
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

    /// Retorna os caminhos incluídos e excluídos finais para passar para o Borg
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
