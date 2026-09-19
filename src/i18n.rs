#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    Pt,
    En,
}

pub struct Translator {
    pub lang: Language,
}

impl Translator {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    pub fn title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "RsBorg - Gerenciador de Backups",
            Language::En => "RsBorg - Backup Manager",
        }
    }

    pub fn header_repo(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Repositório",
            Language::En => "Repository",
        }
    }

    pub fn table_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Lista de Backups",
            Language::En => "Backup List",
        }
    }

    pub fn metadata_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Metadados / Info",
            Language::En => "Metadata / Info",
        }
    }

    pub fn col_name(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Nome",
            Language::En => "Name",
        }
    }
    pub fn col_start(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Criado em",
            Language::En => "Created at",
        }
    }

    pub fn footer_main(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [c] Criar | [l] Idioma | [k/↑] Cima | [j/↓] Baixo | [q/Esc] Sair",
            Language::En => " [c] Create | [l] Language | [k/↑] Up | [j/↓] Down | [q/Esc] Quit",
        }
    }
}
