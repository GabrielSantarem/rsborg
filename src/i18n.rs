#[derive(Clone, Copy, PartialEq, Debug)]
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

    // --- 1. Geral / Cabeçalho ---
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

    pub fn default_local(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Padrão Local",
            Language::En => "Local Default",
        }
    }

    pub fn unknown(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Desconhecido",
            Language::En => "Unknown",
        }
    }

    pub fn checking_borg(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Borg: Verificando...",
            Language::En => "Borg: Checking...",
        }
    }

    pub fn success_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Concluído / Sucesso ",
            Language::En => " Completed / Success ",
        }
    }

    pub fn warning_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Aviso / Warning ",
            Language::En => " Warning ",
        }
    }

    // --- 2. Rodapés de Atalho (Footers) ---
    pub fn footer_main(&self) -> &'static str {
        match self.lang {
            Language::Pt => {
                " [c] Criar | [Enter] Inspecionar | [p] Retenção | [x] Restaurar | [m/u] Montar | [d] Deletar | [r] Repos | [l] Idioma | [q] Sair"
            }
            Language::En => {
                " [c] Create | [Enter] Inspect | [p] Prune | [x] Restore | [m/u] Mount | [d] Delete | [r] Repos | [l] Language | [q] Quit"
            }
        }
    }

    pub fn footer_creating(&self) -> &'static str {
        match self.lang {
            Language::Pt => {
                " [Tab] Alternar Foco | [Espaço] Incluir/Excluir | [Enter/→] Entrar | [BS/←] Subir | [s / Ctrl+S] Criar | [Esc] Cancelar "
            }
            Language::En => {
                " [Tab] Switch Focus | [Space] Include/Exclude | [Enter/→] Enter | [BS/←] Up | [s / Ctrl+S] Create | [Esc] Cancel "
            }
        }
    }

    pub fn footer_confirm_delete(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [y / Enter] Confirmar Exclusão | [n / Esc] Cancelar ",
            Language::En => " [y / Enter] Confirm Deletion | [n / Esc] Cancel ",
        }
    }

    pub fn footer_confirm_restore(&self) -> &'static str {
        match self.lang {
            Language::Pt => {
                " [Enter] Iniciar Restauração | [Backspace] Editar Destino | [Esc] Cancelar "
            }
            Language::En => " [Enter] Start Restore | [Backspace] Edit Destination | [Esc] Cancel ",
        }
    }

    pub fn footer_inspect(&self) -> &'static str {
        match self.lang {
            Language::Pt => {
                " [x] Restaurar Item Selecionado | [j/k/Setas] Rolar | [Esc / Enter] Voltar "
            }
            Language::En => {
                " [x] Restore Selected Item | [j/k/Arrows] Scroll | [Esc / Enter] Back "
            }
        }
    }

    pub fn footer_pruning_policy(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [Tab/Setas] Campo | [Enter/s] Simular (Dry-Run) | [Esc] Cancelar ",
            Language::En => " [Tab/Arrows] Field | [Enter/s] Simulate (Dry-Run) | [Esc] Cancel ",
        }
    }

    pub fn footer_prune_plan(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [y] Confirmar Limpeza Definitiva | [j/k] Rolar | [n / Esc] Cancelar ",
            Language::En => " [y] Confirm Permanent Prune | [j/k] Scroll | [n / Esc] Cancel ",
        }
    }

    pub fn footer_managing_repos(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [Enter] Ativar | [a] Adicionar | [d] Remover | [Esc] Voltar ",
            Language::En => " [Enter] Activate | [a] Add | [d] Remove | [Esc] Back ",
        }
    }

    pub fn footer_adding_repo(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [Tab] Alternar Campo | [Enter] Salvar Repositório | [Esc] Cancelar ",
            Language::En => " [Tab] Switch Field | [Enter] Save Repository | [Esc] Cancel ",
        }
    }

    pub fn footer_popup(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [Esc / Enter] Fechar ",
            Language::En => " [Esc / Enter] Close ",
        }
    }

    pub fn footer_loading(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Executando tarefa do Borg em segundo plano... ",
            Language::En => " Running Borg task in background... ",
        }
    }

    // --- 3. Tabela Principal e Painel de Metadados ---
    pub fn table_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Lista de Backups ",
            Language::En => " Backup List ",
        }
    }

    pub fn metadata_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Metadados / Info ",
            Language::En => " Metadata / Info ",
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

    pub fn no_backup_selected(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Nenhum backup selecionado",
            Language::En => "No backup selected",
        }
    }

    pub fn fuse_mounted_fmt(&self, path: &str) -> String {
        match self.lang {
            Language::Pt => format!("Sim (em {})", path),
            Language::En => format!("Yes (at {})", path),
        }
    }

    pub fn fuse_not_mounted(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Não montado",
            Language::En => "Not mounted",
        }
    }

    pub fn metadata_content_fmt(
        &self,
        id: &str,
        name: &str,
        orig: &str,
        start: &str,
        time: &str,
        mount: &str,
    ) -> String {
        match self.lang {
            Language::Pt => format!(
                "ID: {}\nNome: {}\nArquivo Original: {}\n\nData de Criação: {}\nFinalizado em: {}\n\nMontagem FUSE: {}\n\n[p] Limpeza / Prune\n[x] Restaurar este backup\n[m] Montar pasta FUSE\n[u] Desmontar pasta",
                id, name, orig, start, time, mount
            ),
            Language::En => format!(
                "ID: {}\nName: {}\nOriginal Archive: {}\n\nCreation Date: {}\nFinished at: {}\n\nFUSE Mount: {}\n\n[p] Retention / Prune\n[x] Restore this backup\n[m] Mount FUSE folder\n[u] Unmount folder",
                id, name, orig, start, time, mount
            ),
        }
    }

    // --- 4. Assistente de Criação de Backup ---
    pub fn backup_wizard_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Assistente de Backup / Backup Wizard ",
            Language::En => " Backup Wizard ",
        }
    }

    pub fn backup_name_field_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "1. Nome do Backup (digite e aperte Tab para ir aos arquivos)",
            Language::En => "1. Backup Name (type and press Tab to go to files)",
        }
    }

    pub fn legend_label(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Legenda: ",
            Language::En => "Legend: ",
        }
    }

    pub fn legend_included(&self) -> &'static str {
        match self.lang {
            Language::Pt => "[+] Incluído ",
            Language::En => "[+] Included ",
        }
    }

    pub fn legend_inherited(&self) -> &'static str {
        match self.lang {
            Language::Pt => "[✓] Herdado ",
            Language::En => "[✓] Inherited ",
        }
    }

    pub fn legend_excluded(&self) -> &'static str {
        match self.lang {
            Language::Pt => "[-] Excluído ",
            Language::En => "[-] Excluded ",
        }
    }

    pub fn legend_unselected(&self) -> &'static str {
        match self.lang {
            Language::Pt => "[ ] Não selecionado ",
            Language::En => "[ ] Unselected ",
        }
    }

    pub fn file_browser_title_fmt(&self, path: &str) -> String {
        match self.lang {
            Language::Pt => format!(" 2. Selecionar Arquivos / Pastas  [{}] ", path),
            Language::En => format!(" 2. Select Files / Folders  [{}] ", path),
        }
    }

    pub fn item_status_included(&self) -> &'static str {
        match self.lang {
            Language::Pt => " (Incluído)",
            Language::En => " (Included)",
        }
    }

    pub fn item_status_inherited(&self) -> &'static str {
        match self.lang {
            Language::Pt => " (Herdado do pai)",
            Language::En => " (Inherited from parent)",
        }
    }

    pub fn item_status_excluded(&self) -> &'static str {
        match self.lang {
            Language::Pt => " (Excluído)",
            Language::En => " (Excluded)",
        }
    }

    pub fn item_status_parent_excluded(&self) -> &'static str {
        match self.lang {
            Language::Pt => " (Pai excluído)",
            Language::En => " (Parent excluded)",
        }
    }

    // --- 5. Assistente de Restauração ---
    pub fn restore_wizard_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Assistente de Restauração / Extract ",
            Language::En => " Restore Wizard / Extract ",
        }
    }

    pub fn restore_selected_backup_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Backup Selecionado",
            Language::En => "Selected Backup",
        }
    }

    pub fn restore_destination_dir_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Diretório de Destino (digite para alterar)",
            Language::En => "Destination Directory (type to change)",
        }
    }

    pub fn restore_all_files(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Restauração Total (Todos os arquivos do backup)",
            Language::En => "Full Restore (All backup files)",
        }
    }

    pub fn restore_specific_item_fmt(&self, item: &str) -> String {
        match self.lang {
            Language::Pt => format!("Item específico: {}", item),
            Language::En => format!("Specific item: {}", item),
        }
    }

    pub fn restore_origin_content_fmt(&self, archive: &str, content: &str) -> String {
        match self.lang {
            Language::Pt => format!("Origem: {}\nConteúdo: {}", archive, content),
            Language::En => format!("Origin: {}\nContent: {}", archive, content),
        }
    }

    pub fn btn_start_extraction(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [Enter] Iniciar Extração ",
            Language::En => " [Enter] Start Extraction ",
        }
    }

    pub fn btn_cancel(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [Esc] Cancelar ",
            Language::En => " [Esc] Cancel ",
        }
    }

    // --- 6. Inspetor de Arquivos ---
    pub fn inspect_title_fmt(&self, name: &str, count: usize) -> String {
        match self.lang {
            Language::Pt => format!(
                " Conteúdo do Backup: {} ({} itens) - [x] Restaurar Item Selecionado ",
                name, count
            ),
            Language::En => format!(
                " Backup Content: {} ({} items) - [x] Restore Selected Item ",
                name, count
            ),
        }
    }

    pub fn col_permissions(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Permissões",
            Language::En => "Permissions",
        }
    }

    pub fn col_size(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Tamanho",
            Language::En => "Size",
        }
    }

    pub fn col_path(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Caminho do Arquivo",
            Language::En => "File Path",
        }
    }

    // --- 7. Políticas de Retenção e Prune Plan ---
    pub fn prune_policy_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Políticas de Retenção e Limpeza Automática (Borg Prune) ",
            Language::En => " Automatic Retention & Cleanup Policies (Borg Prune) ",
        }
    }

    pub fn prune_keep_last(&self) -> &'static str {
        match self.lang {
            Language::Pt => "1. Manter últimos N backups (--keep-last):",
            Language::En => "1. Keep last N backups (--keep-last):",
        }
    }

    pub fn prune_keep_daily(&self) -> &'static str {
        match self.lang {
            Language::Pt => "2. Manter backups diários (--keep-daily):",
            Language::En => "2. Keep daily backups (--keep-daily):",
        }
    }

    pub fn prune_keep_weekly(&self) -> &'static str {
        match self.lang {
            Language::Pt => "3. Manter backups semanais (--keep-weekly):",
            Language::En => "3. Keep weekly backups (--keep-weekly):",
        }
    }

    pub fn prune_keep_monthly(&self) -> &'static str {
        match self.lang {
            Language::Pt => "4. Manter backups mensais (--keep-monthly):",
            Language::En => "4. Keep monthly backups (--keep-monthly):",
        }
    }

    pub fn prune_keep_yearly(&self) -> &'static str {
        match self.lang {
            Language::Pt => "5. Manter backups anuais (--keep-yearly):",
            Language::En => "5. Keep yearly backups (--keep-yearly):",
        }
    }

    pub fn prune_prefix(&self) -> &'static str {
        match self.lang {
            Language::Pt => "6. Filtrar por Prefixo (--prefix, opcional):",
            Language::En => "6. Filter by Prefix (--prefix, optional):",
        }
    }

    pub fn prune_hint(&self) -> &'static str {
        match self.lang {
            Language::Pt => {
                "💡 Dica: Deixe vazio para desativar a regra. NENHUM dado será apagado na simulação!"
            }
            Language::En => {
                "💡 Tip: Leave empty to disable rule. NO data will be deleted during simulation!"
            }
        }
    }

    pub fn prune_prompt_button(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [s / Enter] Simular Retenção (Dry-Run)      [Esc] Cancelar ",
            Language::En => " [s / Enter] Simulate Retention (Dry-Run)      [Esc] Cancel ",
        }
    }

    pub fn plan_title_fmt(&self, keep: usize, prune: usize) -> String {
        match self.lang {
            Language::Pt => format!(
                " Simulação de Limpeza: {} Manter | {} Excluir ",
                keep, prune
            ),
            Language::En => format!(" Cleanup Simulation: {} Keep | {} Delete ", keep, prune),
        }
    }

    pub fn plan_total_evaluated_label(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Total de backups avaliados: ",
            Language::En => "Total backups evaluated: ",
        }
    }

    pub fn plan_kept_fmt(&self, count: usize) -> String {
        match self.lang {
            Language::Pt => format!("🟢 Mantidos: {}  |  ", count),
            Language::En => format!("🟢 Kept: {}  |  ", count),
        }
    }

    pub fn plan_to_prune_fmt(&self, count: usize) -> String {
        match self.lang {
            Language::Pt => format!("🔴 A Eliminar: {}", count),
            Language::En => format!("🔴 To Delete: {}", count),
        }
    }

    pub fn col_action(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Ação",
            Language::En => "Action",
        }
    }

    pub fn col_date(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Data",
            Language::En => "Date",
        }
    }

    pub fn col_applied_rule(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Regra Aplicada",
            Language::En => "Applied Rule",
        }
    }

    pub fn badge_keep(&self) -> &'static str {
        match self.lang {
            Language::Pt => "[✓ MANTER]",
            Language::En => "[✓ KEEP]",
        }
    }

    pub fn badge_prune(&self) -> &'static str {
        match self.lang {
            Language::Pt => "[✗ ELIMINAR]",
            Language::En => "[✗ PRUNE]",
        }
    }

    pub fn plan_confirm_button(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [y] Confirmar Limpeza e Liberar Espaço com 'compact' ",
            Language::En => " [y] Confirm Cleanup and Reclaim Space with 'compact' ",
        }
    }

    pub fn plan_none_to_prune(&self) -> &'static str {
        match self.lang {
            Language::Pt => {
                "Todos os backups atendem à sua política de retenção! Nenhum será apagado. [Esc] Voltar"
            }
            Language::En => {
                "All backups meet your retention policy! None will be deleted. [Esc] Back"
            }
        }
    }

    // --- 8. Gerenciador de Repositórios ---
    pub fn repos_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Gerenciador de Repositórios ",
            Language::En => " Repository Manager ",
        }
    }

    pub fn repos_list_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Repositórios Cadastrados ",
            Language::En => " Registered Repositories ",
        }
    }

    pub fn badge_active(&self) -> &'static str {
        match self.lang {
            Language::Pt => " [ATIVO] ",
            Language::En => " [ACTIVE] ",
        }
    }

    pub fn encrypted_label(&self) -> &'static str {
        match self.lang {
            Language::Pt => "🔒 Criptografado",
            Language::En => "🔒 Encrypted",
        }
    }

    pub fn unencrypted_label(&self) -> &'static str {
        match self.lang {
            Language::Pt => "🔓 Sem senha",
            Language::En => "🔓 No passphrase",
        }
    }

    pub fn add_repo_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Adicionar Novo Repositório ",
            Language::En => " Add New Repository ",
        }
    }

    pub fn field_repo_name_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "1. Nome Identificador (ex: HD Externo, Servidor Remoto)",
            Language::En => "1. Identifier Name (e.g., External HD, Remote Server)",
        }
    }

    pub fn field_repo_loc_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "2. Localização (ex: /run/media/... ou ssh://user@host/repo)",
            Language::En => "2. Location (e.g., /run/media/... or ssh://user@host/repo)",
        }
    }

    pub fn field_repo_pass_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => "3. Senha / Passphrase (Opcional - deixe vazio se não tiver)",
            Language::En => "3. Passphrase (Optional - leave empty if unencrypted)",
        }
    }

    // --- 9. Popups, Exclusão e Telemetria de Carregamento ---
    pub fn confirm_delete_title(&self) -> &'static str {
        match self.lang {
            Language::Pt => " Confirmar Exclusão ",
            Language::En => " Confirm Deletion ",
        }
    }

    pub fn confirm_delete_question(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Tem certeza que deseja apagar o backup ",
            Language::En => "Are you sure you want to delete backup ",
        }
    }

    pub fn confirm_delete_warning(&self) -> &'static str {
        match self.lang {
            Language::Pt => {
                "Esta ação é IRREVERSÍVEL. O borg apagará os dados e executará 'borg compact'."
            }
            Language::En => {
                "This action is IRREVERSIBLE. Borg will delete the data and run 'borg compact'."
            }
        }
    }

    pub fn confirm_delete_btn_yes(&self) -> &'static str {
        match self.lang {
            Language::Pt => "[y] Sim, Apagar Definitivamente",
            Language::En => "[y] Yes, Delete Permanently",
        }
    }

    pub fn confirm_delete_btn_cancel(&self) -> &'static str {
        match self.lang {
            Language::Pt => "[n / Esc] Cancelar",
            Language::En => "[n / Esc] Cancel",
        }
    }

    pub fn gauge_activity_fmt(&self, secs: u64) -> String {
        match self.lang {
            Language::Pt => format!("Atividade Borg: {}s", secs),
            Language::En => format!("Borg Activity: {}s", secs),
        }
    }

    pub fn telemetry_elapsed(&self) -> &'static str {
        match self.lang {
            Language::Pt => "⏱️  Tempo decorrido: ",
            Language::En => "⏱️  Elapsed time: ",
        }
    }

    pub fn telemetry_files_count(&self) -> &'static str {
        match self.lang {
            Language::Pt => "📦  Arquivos processados: ",
            Language::En => "📦  Files processed: ",
        }
    }

    pub fn telemetry_original_size(&self) -> &'static str {
        match self.lang {
            Language::Pt => "📊  Tamanho Original: ",
            Language::En => "📊  Original Size: ",
        }
    }

    pub fn telemetry_compressed_size(&self) -> &'static str {
        match self.lang {
            Language::Pt => "  |  Comprimido: ",
            Language::En => "  |  Compressed: ",
        }
    }

    pub fn telemetry_deduplicated_size(&self) -> &'static str {
        match self.lang {
            Language::Pt => "  |  Deduplicado: ",
            Language::En => "  |  Deduplicated: ",
        }
    }

    pub fn telemetry_current_file(&self) -> &'static str {
        match self.lang {
            Language::Pt => "📄  Arquivo atual: ",
            Language::En => "📄  Current file: ",
        }
    }

    pub fn telemetry_processing(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Processando dados...",
            Language::En => "Processing data...",
        }
    }

    // --- 10. Validações e Mensagens de Feedback ---
    pub fn err_name_empty(&self) -> &'static str {
        match self.lang {
            Language::Pt => "O nome do backup não pode ser vazio!",
            Language::En => "Backup name cannot be empty!",
        }
    }

    pub fn err_name_invalid(&self) -> &'static str {
        match self.lang {
            Language::Pt => "O nome do backup não pode conter caracteres reservados (/ ou :)!",
            Language::En => "Backup name cannot contain reserved characters (/ or :)!",
        }
    }

    pub fn err_no_files(&self) -> &'static str {
        match self.lang {
            Language::Pt => {
                "Você precisa selecionar pelo menos um arquivo ou pasta para incluir no backup!"
            }
            Language::En => "You must select at least one file or folder to include in the backup!",
        }
    }

    pub fn err_repo_name_empty(&self) -> &'static str {
        match self.lang {
            Language::Pt => "O nome do repositório não pode ser vazio!",
            Language::En => "Repository name cannot be empty!",
        }
    }

    pub fn err_repo_loc_empty(&self) -> &'static str {
        match self.lang {
            Language::Pt => "A localização do repositório não pode ser vazia!",
            Language::En => "Repository location cannot be empty!",
        }
    }

    pub fn err_fuse_unsupported(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Montagem FUSE não é suportada em repositórios remotos",
            Language::En => "FUSE mount is not supported on remote repositories",
        }
    }

    pub fn err_not_mounted_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("O backup '{}' não está montado.", name),
            Language::En => format!("Backup '{}' is not mounted.", name),
        }
    }

    pub fn msg_backup_success_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Backup '{}' criado com sucesso!", name),
            Language::En => format!("Backup '{}' created successfully!", name),
        }
    }

    pub fn msg_restore_success_fmt(&self, path: &str) -> String {
        match self.lang {
            Language::Pt => format!("Restauração concluída com sucesso em '{}'!", path),
            Language::En => format!("Restoration completed successfully at '{}'!", path),
        }
    }

    pub fn msg_mount_success_fmt(&self, name: &str, path: &str) -> String {
        match self.lang {
            Language::Pt => format!("Backup '{}' montado com sucesso em '{}'!", name, path),
            Language::En => format!("Backup '{}' successfully mounted at '{}'!", name, path),
        }
    }

    pub fn msg_umount_success(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Ponto de montagem desmontado com sucesso!",
            Language::En => "Mount point successfully unmounted!",
        }
    }

    pub fn msg_prune_success_fmt(&self, count: usize) -> String {
        match self.lang {
            Language::Pt => format!(
                "Limpeza concluída! {} backups eliminados e repositório compactado.",
                count
            ),
            Language::En => format!(
                "Cleanup complete! {} archives pruned and repository compacted.",
                count
            ),
        }
    }

    pub fn msg_repo_added_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!(
                "Repositório '{}' adicionado e selecionado como ativo!",
                name
            ),
            Language::En => format!("Repository '{}' added and selected as active!", name),
        }
    }

    pub fn loading_creating_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Criando backup '{}'...", name),
            Language::En => format!("Creating backup '{}'...", name),
        }
    }

    pub fn loading_deleting_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Apagando backup '{}'...", name),
            Language::En => format!("Deleting backup '{}'...", name),
        }
    }

    pub fn loading_restoring_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Restaurando backup '{}'...", name),
            Language::En => format!("Restoring backup '{}'...", name),
        }
    }

    pub fn loading_inspecting_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Carregando lista de arquivos de '{}'...", name),
            Language::En => format!("Loading file list from '{}'...", name),
        }
    }

    pub fn loading_pruning_sim(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Simulando regras de retenção (dry-run)...",
            Language::En => "Simulating retention rules (dry-run)...",
        }
    }

    pub fn loading_pruning_exec(&self) -> &'static str {
        match self.lang {
            Language::Pt => "Executando limpeza e compactação do repositório...",
            Language::En => "Executing repository prune and compaction...",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_translations_differ() {
        let t_pt = Translator::new(Language::Pt);
        let t_en = Translator::new(Language::En);

        assert_ne!(t_pt.title(), t_en.title());
        assert_ne!(t_pt.header_repo(), t_en.header_repo());
        assert_ne!(t_pt.table_title(), t_en.table_title());
        assert_ne!(t_pt.metadata_title(), t_en.metadata_title());
        assert_ne!(t_pt.col_name(), t_en.col_name());
        assert_ne!(t_pt.col_start(), t_en.col_start());
        assert_ne!(t_pt.footer_main(), t_en.footer_main());
        assert_ne!(t_pt.footer_creating(), t_en.footer_creating());
        assert_ne!(t_pt.footer_confirm_delete(), t_en.footer_confirm_delete());
        assert_ne!(t_pt.backup_wizard_title(), t_en.backup_wizard_title());
        assert_ne!(t_pt.restore_wizard_title(), t_en.restore_wizard_title());
        assert_ne!(t_pt.prune_policy_title(), t_en.prune_policy_title());
        assert_ne!(t_pt.repos_title(), t_en.repos_title());
    }

    #[test]
    fn test_format_methods() {
        let t_pt = Translator::new(Language::Pt);
        let t_en = Translator::new(Language::En);

        let pt_fuse = t_pt.fuse_mounted_fmt("/mnt/test");
        let en_fuse = t_en.fuse_mounted_fmt("/mnt/test");
        assert!(pt_fuse.contains("Sim (em /mnt/test)"));
        assert!(en_fuse.contains("Yes (at /mnt/test)"));

        let pt_del = t_pt.loading_deleting_fmt("archive1");
        let en_del = t_en.loading_deleting_fmt("archive1");
        assert!(pt_del.contains("Apagando backup 'archive1'"));
        assert!(en_del.contains("Deleting backup 'archive1'"));

        let pt_prune = t_pt.msg_prune_success_fmt(5);
        let en_prune = t_en.msg_prune_success_fmt(5);
        assert!(pt_prune.contains("5 backups eliminados"));
        assert!(en_prune.contains("5 archives pruned"));
    }
}
