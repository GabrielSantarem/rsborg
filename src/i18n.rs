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
            Language::Pt => "Repositório Ativo",
            Language::En => "Active Repository",
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
            Language::Pt => {
                " [c] Criar | [Enter] Inspecionar | [p] Retenção | [x] Restaurar | [m/u] Montar | [d] Deletar | [r] Repos | [q] Sair"
            }
            Language::En => {
                " [c] Create | [Enter] Inspect | [p] Prune | [x] Restore | [m/u] Mount | [d] Delete | [r] Repos | [q] Quit"
            }
        }
    }
}
