#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Language {
    Pt,
    En,
}

pub struct Translator {
    pub lang: Language,
}

macro_rules! tr {
    ($( $(#[$meta:meta])* $name:ident => (pt: $pt:expr, en: $en:expr) ),* $(,)?) => {
        $(
            $(#[$meta])*
            pub fn $name(&self) -> &'static str {
                match self.lang {
                    Language::Pt => $pt,
                    Language::En => $en,
                }
            }
        )*
    };
}

impl Translator {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    tr! {
        title => (pt: "RsBorg - Gerenciador de Backups", en: "RsBorg - Backup Manager"),
        header_repo => (pt: "Repositório Ativo", en: "Active Repository"),
        default_local => (pt: "Padrão Local", en: "Local Default"),
        unknown => (pt: "Desconhecido", en: "Unknown"),
        checking_borg => (pt: "Borg: Verificando...", en: "Borg: Checking..."),
        success_title => (pt: " Concluído / Sucesso ", en: " Completed / Success "),
        warning_title => (pt: " Aviso / Warning ", en: " Warning "),
        #[allow(dead_code)]
        footer_main => (pt: {
                " [c] Criar | [b] Perfis | [Enter] Inspecionar | [f] Diff | [v] Verificar | [p] Retenção | [x] Restaurar | [m/u] Montar | [d] Deletar | [r] Repos | [l] Idioma | [q] Sair"
            }, en: {
                " [c] Create | [b] Profiles | [Enter] Inspect | [f] Diff | [v] Verify | [p] Prune | [x] Restore | [m/u] Mount | [d] Delete | [r] Repos | [l] Language | [q] Quit"
            }),
        footer_creating => (pt: {
                " [Tab] Alternar Foco | [Espaço] Incluir/Excluir | [Enter/→] Entrar | [BS/←] Subir | [s / Ctrl+S] Criar | [Esc] Cancelar "
            }, en: {
                " [Tab] Switch Focus | [Space] Include/Exclude | [Enter/→] Enter | [BS/←] Up | [s / Ctrl+S] Create | [Esc] Cancel "
            }),
        footer_confirm_delete => (pt: " [y / Enter] Confirmar Exclusão | [n / Esc] Cancelar ", en: " [y / Enter] Confirm Deletion | [n / Esc] Cancel "),
        footer_confirm_restore => (pt: {
                " [Enter] Iniciar Restauração | [Backspace] Editar Destino | [Esc] Cancelar "
            }, en: " [Enter] Start Restore | [Backspace] Edit Destination | [Esc] Cancel "),
        footer_inspect => (pt: {
                " [x] Restaurar Item Selecionado | [j/k/Setas] Rolar | [Esc / Enter] Voltar "
            }, en: {
                " [x] Restore Selected Item | [j/k/Arrows] Scroll | [Esc / Enter] Back "
            }),
        footer_pruning_policy => (pt: " [Tab/Setas] Campo | [Enter/s] Simular (Dry-Run) | [Esc] Cancelar ", en: " [Tab/Arrows] Field | [Enter/s] Simulate (Dry-Run) | [Esc] Cancel "),
        footer_prune_plan => (pt: " [y] Confirmar Limpeza Definitiva | [j/k] Rolar | [n / Esc] Cancelar ", en: " [y] Confirm Permanent Prune | [j/k] Scroll | [n / Esc] Cancel "),
        footer_managing_repos => (pt: " [Enter] Ativar | [a] Adicionar | [d] Remover | [Esc] Voltar ", en: " [Enter] Activate | [a] Add | [d] Remove | [Esc] Back "),
        footer_adding_repo => (pt: " [Tab] Alternar Campo | [Enter] Salvar Repositório | [Esc] Cancelar ", en: " [Tab] Switch Field | [Enter] Save Repository | [Esc] Cancel "),
        footer_popup => (pt: " [Esc / Enter] Fechar ", en: " [Esc / Enter] Close "),
        footer_profiles => (pt: " [Enter] Executar Backup | [a] Novo Perfil | [s] Automação Systemd/Cron | [d] Excluir | [Esc] Voltar ", en: " [Enter] Run Backup Now | [a] New Profile | [s] Systemd/Cron Automation | [d] Delete | [Esc] Back "),
        footer_profile_wizard => (pt: " [Tab] Campo | [←/→] Opção | [Espaço] Incluir | [e] Excluir | [Ctrl+S] Salvar | [Esc] Cancelar ", en: " [Tab] Field | [←/→] Option | [Space] Include | [e] Exclude | [Ctrl+S] Save | [Esc] Cancel "),
        footer_automation_view => (pt: " [Tab] Alternar Aba | [i] Instalar no Systemd User | [Esc] Voltar ", en: " [Tab] Switch Tab | [i] Install to Systemd User | [Esc] Back "),
        footer_loading => (pt: " Executando tarefa do Borg em segundo plano... ", en: " Running Borg task in background... "),
        table_title => (pt: " Lista de Backups ", en: " Backup List "),
        metadata_title => (pt: " Metadados / Info ", en: " Metadata / Info "),
        col_name => (pt: "Nome", en: "Name"),
        col_start => (pt: "Criado em", en: "Created at"),
        no_backup_selected => (pt: "Nenhum backup selecionado", en: "No backup selected"),
        #[allow(dead_code)]
        fuse_not_mounted => (pt: "Não montado", en: "Not mounted"),
        backup_wizard_title => (pt: " Assistente de Backup / Backup Wizard ", en: " Backup Wizard "),
        backup_name_field_title => (pt: "1. Nome do Backup (digite e aperte Tab para ir aos arquivos)", en: "1. Backup Name (type and press Tab to go to files)"),
        legend_label => (pt: "Legenda: ", en: "Legend: "),
        legend_included => (pt: "[+] Incluído ", en: "[+] Included "),
        legend_inherited => (pt: "[✓] Herdado ", en: "[✓] Inherited "),
        legend_excluded => (pt: "[-] Excluído ", en: "[-] Excluded "),
        legend_unselected => (pt: "[ ] Não selecionado ", en: "[ ] Unselected "),
        item_status_included => (pt: " (Incluído)", en: " (Included)"),
        item_status_inherited => (pt: " (Herdado do pai)", en: " (Inherited from parent)"),
        item_status_excluded => (pt: " (Excluído)", en: " (Excluded)"),
        item_status_parent_excluded => (pt: " (Pai excluído)", en: " (Parent excluded)"),
        restore_wizard_title => (pt: " Assistente de Restauração / Extract ", en: " Restore Wizard / Extract "),
        restore_selected_backup_title => (pt: "Backup Selecionado", en: "Selected Backup"),
        restore_destination_dir_title => (pt: "Diretório de Destino (digite para alterar)", en: "Destination Directory (type to change)"),
        restore_all_files => (pt: "Restauração Total (Todos os arquivos do backup)", en: "Full Restore (All backup files)"),
        btn_start_extraction => (pt: " [Enter] Iniciar Extração ", en: " [Enter] Start Extraction "),
        btn_cancel => (pt: " [Esc] Cancelar ", en: " [Esc] Cancel "),
        col_permissions => (pt: "Permissões", en: "Permissions"),
        col_size => (pt: "Tamanho", en: "Size"),
        col_path => (pt: "Caminho do Arquivo", en: "File Path"),
        prune_policy_title => (pt: " Políticas de Retenção e Limpeza Automática (Borg Prune) ", en: " Automatic Retention & Cleanup Policies (Borg Prune) "),
        prune_keep_last => (pt: "1. Manter últimos N backups (--keep-last):", en: "1. Keep last N backups (--keep-last):"),
        prune_keep_daily => (pt: "2. Manter backups diários (--keep-daily):", en: "2. Keep daily backups (--keep-daily):"),
        prune_keep_weekly => (pt: "3. Manter backups semanais (--keep-weekly):", en: "3. Keep weekly backups (--keep-weekly):"),
        prune_keep_monthly => (pt: "4. Manter backups mensais (--keep-monthly):", en: "4. Keep monthly backups (--keep-monthly):"),
        prune_keep_yearly => (pt: "5. Manter backups anuais (--keep-yearly):", en: "5. Keep yearly backups (--keep-yearly):"),
        prune_prefix => (pt: "6. Filtrar por Prefixo (--prefix, opcional):", en: "6. Filter by Prefix (--prefix, optional):"),
        prune_hint => (pt: {
                "💡 Dica: Deixe vazio para desativar a regra. NENHUM dado será apagado na simulação!"
            }, en: {
                "💡 Tip: Leave empty to disable rule. NO data will be deleted during simulation!"
            }),
        prune_prompt_button => (pt: " [s / Enter] Simular Retenção (Dry-Run)      [Esc] Cancelar ", en: " [s / Enter] Simulate Retention (Dry-Run)      [Esc] Cancel "),
        plan_total_evaluated_label => (pt: "Total de backups avaliados: ", en: "Total backups evaluated: "),
        col_action => (pt: "Ação", en: "Action"),
        col_date => (pt: "Data", en: "Date"),
        col_applied_rule => (pt: "Regra Aplicada", en: "Applied Rule"),
        badge_keep => (pt: "[✓ MANTER]", en: "[✓ KEEP]"),
        badge_prune => (pt: "[✗ ELIMINAR]", en: "[✗ PRUNE]"),
        plan_confirm_button => (pt: " [y] Confirmar Limpeza e Liberar Espaço com 'compact' ", en: " [y] Confirm Cleanup and Reclaim Space with 'compact' "),
        plan_none_to_prune => (pt: {
                "Todos os backups atendem à sua política de retenção! Nenhum será apagado. [Esc] Voltar"
            }, en: {
                "All backups meet your retention policy! None will be deleted. [Esc] Back"
            }),
        repos_title => (pt: " Gerenciador de Repositórios ", en: " Repository Manager "),
        repos_list_title => (pt: " Repositórios Cadastrados ", en: " Registered Repositories "),
        badge_active => (pt: " [ATIVO] ", en: " [ACTIVE] "),
        encrypted_label => (pt: "🔒 Criptografado", en: "🔒 Encrypted"),
        unencrypted_label => (pt: "🔓 Sem senha", en: "🔓 No passphrase"),
        add_repo_title => (pt: " Adicionar Novo Repositório ", en: " Add New Repository "),
        field_repo_name_title => (pt: "1. Nome Identificador (ex: HD Externo, Servidor Remoto)", en: "1. Identifier Name (e.g., External HD, Remote Server)"),
        field_repo_loc_title => (pt: "2. Localização (ex: /run/media/... ou ssh://user@host/repo)", en: "2. Location (e.g., /run/media/... or ssh://user@host/repo)"),
        field_repo_pass_title => (pt: "3. Senha / Passphrase (Opcional - deixe vazio se não tiver)", en: "3. Passphrase (Optional - leave empty if unencrypted)"),
        confirm_delete_title => (pt: " Confirmar Exclusão ", en: " Confirm Deletion "),
        confirm_delete_question => (pt: "Tem certeza que deseja apagar o backup ", en: "Are you sure you want to delete backup "),
        confirm_delete_warning => (pt: {
                "Esta ação é IRREVERSÍVEL. O borg apagará os dados e executará 'borg compact'."
            }, en: {
                "This action is IRREVERSIBLE. Borg will delete the data and run 'borg compact'."
            }),
        confirm_delete_btn_yes => (pt: "[y] Sim, Apagar Definitivamente", en: "[y] Yes, Delete Permanently"),
        confirm_delete_btn_cancel => (pt: "[n / Esc] Cancelar", en: "[n / Esc] Cancel"),
        telemetry_elapsed => (pt: "⏱️  Tempo decorrido: ", en: "⏱️  Elapsed time: "),
        telemetry_files_count => (pt: "📦  Arquivos processados: ", en: "📦  Files processed: "),
        telemetry_original_size => (pt: "📊  Tamanho Original: ", en: "📊  Original Size: "),
        telemetry_compressed_size => (pt: "  |  Comprimido: ", en: "  |  Compressed: "),
        telemetry_deduplicated_size => (pt: "  |  Deduplicado: ", en: "  |  Deduplicated: "),
        telemetry_current_file => (pt: "📄  Arquivo atual: ", en: "📄  Current file: "),
        telemetry_processing => (pt: "Processando dados...", en: "Processing data..."),
        err_name_empty => (pt: "O nome do backup não pode ser vazio!", en: "Backup name cannot be empty!"),
        err_name_invalid => (pt: "O nome do backup não pode conter caracteres reservados (/ ou :)!", en: "Backup name cannot contain reserved characters (/ or :)!"),
        err_no_files => (pt: {
                "Você precisa selecionar pelo menos um arquivo ou pasta para incluir no backup!"
            }, en: "You must select at least one file or folder to include in the backup!"),
        err_repo_name_empty => (pt: "O nome do repositório não pode ser vazio!", en: "Repository name cannot be empty!"),
        err_repo_loc_empty => (pt: "A localização do repositório não pode ser vazia!", en: "Repository location cannot be empty!"),
        err_fuse_unsupported => (pt: "Montagem FUSE não é suportada em repositórios remotos", en: "FUSE mount is not supported on remote repositories"),
        msg_umount_success => (pt: "Ponto de montagem desmontado com sucesso!", en: "Mount point successfully unmounted!"),
        loading_pruning_sim => (pt: "Simulando regras de retenção (dry-run)...", en: "Simulating retention rules (dry-run)..."),
        loading_pruning_exec => (pt: "Executando limpeza e compactação do repositório...", en: "Executing repository prune and compaction..."),
        check_wizard_title => (pt: " Diagnóstico e Verificação de Integridade (Borg Check) ", en: " Integrity Diagnosis & Verification (Borg Check) "),
        check_target_title => (pt: "1. Alvo da Verificação ([Tab] para alternar)", en: "1. Verification Target ([Tab] to toggle)"),
        check_target_entire_repo => (pt: "Repositório Completo (Todos os arquivos e índices)", en: "Entire Repository (All archives and indexes)"),
        check_mode_title => (pt: "2. Modo de Diagnóstico ([↑/↓] para navegar)", en: "2. Diagnostic Mode ([↑/↓] to navigate)"),
        check_mode_quick => (pt: "Rápido (--repository-only): Valida estruturas do repo e índices de chunks", en: "Quick (--repository-only): Validates repo structure and chunk indexes"),
        check_mode_standard => (pt: "Padrão (Repo + Arquivos): Valida integridade do repo e manifestos dos backups", en: "Standard (Repo + Archives): Validates repo integrity and backup manifests"),
        check_mode_verify_data => (pt: "Profundo (--verify-data): Descriptografa e valida integridade de TODOS os dados (Mais lento)", en: "Deep (--verify-data): Decrypts and verifies integrity of ALL data chunks (Slower)"),
        check_mode_repair => (pt: "Reparo (--repair): Tenta reconstruir índices e recuperar dados corrompidos (Avançado)", en: "Repair (--repair): Attempts to rebuild indexes and salvage corrupted data (Advanced)"),
        check_start_prompt => (pt: " [Enter] Iniciar Diagnóstico      [Esc] Cancelar ", en: " [Enter] Start Diagnosis      [Esc] Cancel "),
        check_result_title => (pt: " Relatório de Diagnóstico e Integridade ", en: " Diagnostic & Integrity Report "),
        check_status_healthy => (pt: "🟢 CONSISTENTE: Nenhum problema ou corrupção detectada!", en: "🟢 HEALTHY: No inconsistencies or corruption detected!"),
        check_status_warning => (pt: "🟡 AVISOS: Inconsistências leves ou avisos detectados.", en: "🟡 WARNINGS: Minor inconsistencies or warnings detected."),
        check_status_corrupted => (pt: "🔴 ERRO: Corrupção ou inconsistência grave detectada!", en: "🔴 ERROR: Serious corruption or inconsistency detected!"),
        check_logs_header => (pt: " Logs e Mensagens do Borg (stderr) ", en: " Borg Diagnostic Output (stderr) "),
        footer_check_wizard => (pt: " [Tab] Alvo | [↑/↓] Modo | [Enter] Executar Diagnóstico | [Esc] Cancelar ", en: " [Tab] Target | [↑/↓] Mode | [Enter] Run Diagnosis | [Esc] Cancel "),
        footer_check_result => (pt: " [j/k/Setas] Rolar Logs | [Esc / Enter] Voltar ", en: " [j/k/Arrows] Scroll Logs | [Esc / Enter] Back "),
        diff_wizard_title => (pt: " Comparação entre Backups (Borg Diff) ", en: " Backup Comparison (Borg Diff) "),
        diff_base_title => (pt: "1. Backup Base (Origem / Mais Antigo)", en: "1. Base Backup (Source / Older)"),
        diff_target_title => (pt: "2. Backup Alvo para Comparação ([↑/↓] para escolher)", en: "2. Target Backup to Compare ([↑/↓] to select)"),
        diff_start_prompt => (pt: " [Enter] Comparar Alterações      [Esc] Cancelar ", en: " [Enter] Compare Changes      [Esc] Cancel "),
        diff_view_title => (pt: " Relatório de Alterações (Diff) ", en: " Changes Report (Diff) "),
        diff_col_type => (pt: "Tipo", en: "Type"),
        diff_col_change => (pt: "Variação / Detalhes", en: "Change / Details"),
        diff_col_path => (pt: "Caminho do Arquivo", en: "File Path"),
        diff_badge_added => (pt: "[+ NOVO]", en: "[+ ADDED]"),
        diff_badge_removed => (pt: "[- REMOVIDO]", en: "[- REMOVED]"),
        diff_badge_modified => (pt: "[~ MODIFICADO]", en: "[~ MODIFIED]"),
        diff_badge_metadata => (pt: "[⚙ METADADOS]", en: "[⚙ METADATA]"),
        diff_no_changes => (pt: "Nenhuma alteração detectada entre estes backups (conteúdo e metadados idênticos).", en: "No changes detected between these backups (content and metadata are identical)."),
        diff_err_need_two => (pt: "Você precisa de pelo menos 2 backups para realizar uma comparação!", en: "You need at least 2 backups to perform a comparison!"),
        footer_diff_wizard => (pt: " [↑/↓] Selecionar Alvo | [Tab] Conteúdo apenas | [Enter] Comparar | [Esc] Cancelar ", en: " [↑/↓] Select Target | [Tab] Content only | [Enter] Compare | [Esc] Cancel "),
        footer_diff_view => (pt: " [j/k/Setas] Navegar Alterações | [Esc / Enter] Voltar ", en: " [j/k/Arrows] Navigate Changes | [Esc / Enter] Back "),
        profiles_title => (pt: " Conjuntos e Perfis de Backup Automático ", en: " Automated Backup Sets & Profiles "),
        profiles_empty => (pt: "Nenhum conjunto cadastrado ainda. Pressione [a] para criar seu primeiro perfil de backup!", en: "No backup profiles registered yet. Press [a] to create your first backup profile!"),
        profile_details_title => (pt: " Detalhes do Perfil Selecionado ", en: " Selected Profile Details "),
        profile_wizard_title => (pt: " Criar Novo Conjunto de Backup ", en: " Create New Backup Profile "),
        profile_name_label => (pt: "1. Nome do Conjunto (ex: BACKUP_DIARIO, FOTOS):", en: "1. Profile Name (e.g. BACKUP_DIARIO, PHOTOS):"),
        profile_compression_label => (pt: "2. Algoritmo de Compressão ([←/→] para alternar):", en: "2. Compression Algorithm ([←/→] to switch):"),
        profile_schedule_label => (pt: "3. Agendamento Frequência ([←/→] para alternar):", en: "3. Schedule Frequency ([←/→] to switch):"),
        profile_paths_label => (pt: "4. Selecionar Pastas e Arquivos ([Espaço] inclui / [e] exclui):", en: "4. Select Folders & Files ([Space] include / [e] exclude):"),
        automation_installed_msg => (pt: "Arquivos Systemd criados com sucesso em ~/.config/systemd/user/!", en: "Systemd files successfully created in ~/.config/systemd/user/!"),
        profile_save_prompt => (pt: " [Ctrl+S] Salvar Perfil      [Esc] Cancelar ", en: " [Ctrl+S] Save Profile      [Esc] Cancel "),

        // --- Perfis UI ---
        profiles_configured_title => (pt: " Perfis Configurados ", en: " Configured Profiles "),
        profile_label_name => (pt: "Nome do Perfil: ", en: "Profile Name: "),
        profile_label_compression => (pt: "Compressão: ", en: "Compression: "),
        profile_label_schedule => (pt: "Frequência: ", en: "Schedule: "),
        profile_label_runs => (pt: "Execuções Realizadas: ", en: "Completed Runs: "),

        // --- Automação Modal ---
        automation_modal_title => (pt: " Automação e Agendamento no Linux ", en: " Linux Automation & Scheduling "),
        automation_tab_service => (pt: " 1. Systemd Service (.service) ", en: " 1. Systemd Service (.service) "),
        automation_tab_timer => (pt: " 2. Systemd Timer (.timer) ", en: " 2. Systemd Timer (.timer) "),
        automation_tab_cron => (pt: " 3. Linha Crontab ", en: " 3. Crontab Line "),
        automation_generated_content_title => (pt: " Conteúdo Gerado ", en: " Generated Content "),
        automation_footer_prompt => (pt: " [Tab] Alternar Aba    [i] Gravar em ~/.config/systemd/user/    [Esc] Voltar ", en: " [Tab] Switch Tab    [i] Save to ~/.config/systemd/user/    [Esc] Back "),

        // --- Navegação & Empty State ---
        browse_empty_title => (pt: " [ LISTA DE BACKUPS ] ", en: " [ BACKUP LIST ] "),
        browse_empty_header => (pt: " [ NENHUM BACKUP ENCONTRADO ] ", en: " [ NO ARCHIVES FOUND ] "),
        browse_empty_desc => (pt: "O repositório ativo está pronto, mas ainda não possui nenhum snapshot arquivado.", en: "The active repository is ready, but contains no archived snapshots yet."),
        browse_empty_actions => (pt: "Ações recomendadas:", en: "Recommended actions:"),
        browse_empty_act1 => (pt: " para criar seu primeiro backup interativo", en: " to create your first interactive backup"),
        browse_empty_act2 => (pt: " para configurar um perfil automatizado com systemd/cron", en: " to configure an automated profile with systemd/cron"),
        browse_empty_act3 => (pt: " para alternar para outro repositório com backups existentes", en: " to switch to another repository with existing backups"),
        browse_press => (pt: "Pressione ", en: "Press "),
        browse_meta_snapshot => (pt: "  Snapshot:  ", en: "  Snapshot:  "),
        browse_meta_id => (pt: "  ID:        ", en: "  ID:        "),
        browse_meta_start => (pt: "  Início:    ", en: "  Start:     "),
        browse_meta_duration => (pt: "  Duração:   ", en: "  Duration:  "),
        browse_meta_fuse => (pt: "  FUSE:      ", en: "  FUSE:      "),
        browse_fuse_unmounted => (pt: " [NÃO MONTADO]", en: " [NOT MOUNTED]"),
        browse_quick_actions => (pt: "  Ações Rápidas:", en: "  Quick Actions:"),
        browse_act_explore => (pt: "Explorar conteúdo interno", en: "Explore internal files"),
        browse_act_mount => (pt: "Montar / Desmontar FUSE", en: "Mount / Unmount FUSE"),
        browse_act_restore => (pt: "Restaurar este backup", en: "Restore this backup"),
        browse_act_delete => (pt: "Excluir permanentemente", en: "Delete permanently"),

        // --- Help Modal ---
        help_title => (pt: " [ AJUDA / ATALHOS DE TECLADO ] ", en: " [ HELP / KEYBOARD SHORTCUTS ] "),
        help_sec_nav => (pt: "  NAVEGAÇÃO GERAL", en: "  GENERAL NAVIGATION"),
        help_nav_jk => (pt: "Navegar pelas listas de backups ou arquivos", en: "Navigate backup or file lists"),
        help_nav_enter => (pt: "Acessar diretório / Confirmar seleção", en: "Enter directory / Confirm selection"),
        help_nav_esc => (pt: "Voltar à tela anterior / Fechar modal / Sair", en: "Go back / Close modal / Exit"),
        help_nav_theme => (pt: "Alternar Tema Visual (Rust Oxide <-> Catppuccin Mocha)", en: "Toggle Theme (Rust Oxide <-> Catppuccin Mocha)"),
        help_nav_lang => (pt: "Alternar Idioma (Português <-> English)", en: "Toggle Language (Português <-> English)"),
        help_sec_actions => (pt: "  AÇÕES EM SNAPSHOTS", en: "  SNAPSHOT ACTIONS"),
        help_act_create => (pt: "Criar Novo Backup (Assistente com seletor de arquivos)", en: "Create New Backup (File picker wizard)"),
        help_act_inspect => (pt: "Inspecionar arquivos dentro do snapshot selecionado", en: "Inspect files inside selected snapshot"),
        help_act_restore => (pt: "Restaurar snapshot selecionado para disco", en: "Restore selected snapshot to disk"),
        help_act_mount => (pt: "Montar snapshot via FUSE em ~/.rsborg/mnt / Desmontar", en: "Mount snapshot via FUSE in ~/.rsborg/mnt / Unmount"),
        help_act_diff => (pt: "Comparar Versões (Diff visual entre dois snapshots)", en: "Compare Versions (Visual diff between two snapshots)"),
        help_act_check => (pt: "Verificar integridade do repositório (borg check)", en: "Check repository integrity (borg check)"),
        help_act_prune => (pt: "Política de Retenção & Poda com Simulação (borg prune dry-run)", en: "Retention Policy & Pruning Simulation (borg prune dry-run)"),
        help_act_delete => (pt: "Excluir snapshot permanentemente do repositório", en: "Permanently delete snapshot from repository"),
        help_sec_profiles => (pt: "  PERFIS & AUTOMAÇÃO", en: "  PROFILES & AUTOMATION"),
        help_prof_manage => (pt: "Gerenciar Perfis de Backup e Gerador Systemd/Cron", en: "Manage Backup Profiles and Systemd/Cron Generator"),
        help_prof_repos => (pt: "Alternar entre Repositórios configurados ou adicionar novo", en: "Switch configured Repositories or add new one"),
        help_close_hint => (pt: "  [ Pressione Esc, q ou ? para fechar este menu de ajuda ]", en: "  [ Press Esc, q or ? to close this help menu ]"),
        btn_break_lock => (pt: "Destravar Repositório (break-lock)", en: "Break Repository Lock (break-lock)"),
        msg_break_lock_success => (pt: "Trava do repositório liberada com sucesso (break-lock)!", en: "Repository lock released successfully (break-lock)!"),
        logs_title => (pt: " REGISTRO DE LOGS E OPERAÇÕES (RSBORG.LOG) ", en: " OPERATION LOGS & DEBUG (RSBORG.LOG) "),
        logs_filter_all => (pt: "Todos", en: "All"),
        logs_filter_info => (pt: "Info", en: "Info"),
        logs_filter_warn => (pt: "Avisos", en: "Warnings"),
        logs_filter_error => (pt: "Erros", en: "Errors"),
        logs_empty => (pt: "Nenhum registro de log encontrado para este filtro.", en: "No log records found for this filter."),
        logs_footer => (pt: " [1-4] Filtros | [j/k/Setas] Rolar | [g/G] Topo/Fim | [c] Limpar | [r] Recarregar | [Esc] Fechar ", en: " [1-4] Filters | [j/k/Arrows] Scroll | [g/G] Top/End | [c] Clear | [r] Reload | [Esc] Close "),
        help_act_logs => (pt: "Visualizar Logs do Sistema e Depuração (rsborg.log)", en: "View System and Debugging Logs (rsborg.log)"),
        settings_title => (pt: " CONFIGURAÇÕES & PREFERÊNCIAS ", en: " SETTINGS & PREFERENCES "),
        settings_item_lang => (pt: "Idioma da Interface", en: "UI Language"),
        settings_item_theme => (pt: "Tema Visual", en: "Visual Theme"),
        settings_item_repos => (pt: "Gerenciar Repositórios", en: "Manage Repositories"),
        settings_item_stats => (pt: "Estatísticas & Deduplicação", en: "Statistics & Deduplication"),
        settings_item_logs => (pt: "Visualizador de Logs", en: "System Logs Viewer"),
        settings_item_restore => (pt: "Pasta Padrão de Restauração", en: "Default Restore Folder"),
        settings_item_core => (pt: "Motor BorgBackup", en: "BorgBackup Engine"),
        settings_footer => (pt: " [j/k/Setas] Navegar | [Enter/Espaço] Alternar/Abrir | [Esc] Fechar ", en: " [j/k/Arrows] Navigate | [Enter/Space] Toggle/Open | [Esc] Close "),
        footer_settings => (pt: "Configurações", en: "Settings"),
        help_nav_settings => (pt: "Abrir Central de Configurações (Idioma, Tema, Repositórios, Logs)", en: "Open Settings Center (Language, Theme, Repos, Logs)"),
        info_title => (pt: " DETALHES & ESTATÍSTICAS (BORG INFO) ", en: " DETAILS & STATISTICS (BORG INFO) "),
        info_tab_snapshot => (pt: " [1] Snapshot Selecionado ", en: " [1] Selected Snapshot "),
        info_tab_repo => (pt: " [2] Repositório & Deduplicação Global ", en: " [2] Repository & Global Deduplication "),
        info_sec_metadata => (pt: "Identificação & Metadados", en: "Identification & Metadata"),
        info_sec_stats => (pt: "Tamanhos & Taxa de Compressão", en: "Sizes & Compression Ratio"),
        info_sec_dedup => (pt: "Deduplicação & Economia de Disco", en: "Deduplication & Disk Savings"),
        info_sec_repo => (pt: "Estrutura do Repositório & Cache Local", en: "Repository Structure & Local Cache"),
        info_lbl_name => (pt: "Nome:", en: "Name:"),
        info_lbl_id => (pt: "ID Hash:", en: "Hash ID:"),
        info_lbl_timestamp => (pt: "Timestamp:", en: "Timestamp:"),
        info_lbl_duration => (pt: "Duração da Execução:", en: "Execution Duration:"),
        info_lbl_host_user => (pt: "Host / Usuário:", en: "Host / User:"),
        info_lbl_command => (pt: "Comando Borg:", en: "Borg Command:"),
        info_lbl_files => (pt: "Arquivos Contidos:", en: "Files Count:"),
        info_lbl_original_size => (pt: "Tamanho Original (Não comprimido):", en: "Original Size (Uncompressed):"),
        info_lbl_compressed_size => (pt: "Tamanho Comprimido:", en: "Compressed Size:"),
        info_lbl_dedup_size => (pt: "Deduplicado (Novo no disco):", en: "Deduplicated (New on disk):"),
        info_lbl_compression_ratio => (pt: "Economia por Compressão:", en: "Compression Savings:"),
        info_lbl_repo_location => (pt: "Localização:", en: "Location:"),
        info_lbl_repo_id => (pt: "ID do Repositório:", en: "Repository ID:"),
        info_lbl_encryption => (pt: "Criptografia:", en: "Encryption:"),
        info_lbl_repo_original => (pt: "Volume Total Não Comprimido:", en: "Total Uncompressed Volume:"),
        info_lbl_repo_compressed => (pt: "Volume Total Comprimido:", en: "Total Compressed Volume:"),
        info_lbl_repo_dedup => (pt: "Tamanho Real Ocupado em Disco:", en: "Actual Disk Space Used:"),
        info_lbl_repo_savings => (pt: "Economia Total por Deduplicação:", en: "Total Deduplication Savings:"),
        info_lbl_chunks => (pt: "Total de Chunks:", en: "Total Chunks:"),
        info_lbl_unique_chunks => (pt: "Chunks Únicos (Deduplicados):", en: "Unique Chunks (Deduplicated):"),
        info_footer => (pt: " [1/2/Tab] Alternar Abas | [Esc/q] Fechar ", en: " [1/2/Tab] Switch Tabs | [Esc/q] Close "),
        footer_info => (pt: "Info", en: "Info"),
        help_act_info => (pt: "Ver Estatísticas e Deduplicação (borg info)", en: "View Statistics & Deduplication (borg info)"),
        loading_repo_info => (pt: "Consultando estatísticas do repositório...", en: "Querying repository statistics..."),
    }

    // --- Métodos Dinâmicos com Formatação (Parâmetros) ---
    #[allow(dead_code)]
    pub fn loading_info_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Consultando estatísticas de '{}'...", name),
            Language::En => format!("Querying statistics for '{}'...", name),
        }
    }

    #[allow(dead_code)]
    pub fn fuse_mounted_fmt(&self, path: &str) -> String {
        match self.lang {
            Language::Pt => format!("Sim (em {})", path),
            Language::En => format!("Yes (at {})", path),
        }
    }

    #[allow(dead_code)]
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

    pub fn file_browser_title_fmt(&self, path: &str) -> String {
        match self.lang {
            Language::Pt => format!(" 2. Selecionar Arquivos / Pastas  [{}] ", path),
            Language::En => format!(" 2. Select Files / Folders  [{}] ", path),
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

    pub fn plan_title_fmt(&self, keep: usize, prune: usize) -> String {
        match self.lang {
            Language::Pt => format!(
                " Simulação de Limpeza: {} Manter | {} Excluir ",
                keep, prune
            ),
            Language::En => format!(" Cleanup Simulation: {} Keep | {} Delete ", keep, prune),
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

    pub fn gauge_activity_fmt(&self, secs: u64) -> String {
        match self.lang {
            Language::Pt => format!("Atividade Borg: {}s", secs),
            Language::En => format!("Borg Activity: {}s", secs),
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

    pub fn check_target_archive_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Apenas o backup selecionado: '{}'", name),
            Language::En => format!("Selected backup only: '{}'", name),
        }
    }

    pub fn loading_checking_fmt(&self, target: &str) -> String {
        match self.lang {
            Language::Pt => format!("Diagnosticando integridade em '{}'...", target),
            Language::En => format!("Checking integrity on '{}'...", target),
        }
    }

    pub fn diff_option_content_only(&self, active: bool) -> String {
        let check = if active { "[X]" } else { "[ ]" };
        match self.lang {
            Language::Pt => format!("{} [Tab] Comparar apenas conteúdo de arquivos (ignorar metadados/permissões)", check),
            Language::En => format!("{} [Tab] Compare file content only (ignore metadata/permissions)", check),
        }
    }

    pub fn diff_summary_fmt(&self, added: usize, removed: usize, modified: usize, meta: usize) -> String {
        match self.lang {
            Language::Pt => format!(
                "+{} Adicionados | -{} Removidos | ~{} Modificados | {} Metadados | Total: {} alterações",
                added, removed, modified, meta, added + removed + modified + meta
            ),
            Language::En => format!(
                "+{} Added | -{} Removed | ~{} Modified | {} Metadata | Total: {} changes",
                added, removed, modified, meta, added + removed + modified + meta
            ),
        }
    }

    pub fn loading_diffing_fmt(&self, a1: &str, a2: &str) -> String {
        match self.lang {
            Language::Pt => format!("Comparando '{}' com '{}'...", a1, a2),
            Language::En => format!("Comparing '{}' with '{}'...", a1, a2),
        }
    }

    pub fn profile_next_archive_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Próximo arquivo que será criado: {}", name),
            Language::En => format!("Next archive that will be created: {}", name),
        }
    }



    pub fn profile_included_paths_fmt(&self, count: usize) -> String {
        match self.lang {
            Language::Pt => format!("Pastas e Arquivos Incluídos ({}):", count),
            Language::En => format!("Included Folders and Files ({}):", count),
        }
    }

    pub fn profile_excluded_paths_fmt(&self, count: usize) -> String {
        match self.lang {
            Language::Pt => format!("Pastas e Arquivos Excluídos ({}):", count),
            Language::En => format!("Excluded Folders and Files ({}):", count),
        }
    }

    pub fn browse_fuse_mounted_fmt(&self, path: &str) -> String {
        match self.lang {
            Language::Pt => format!(" [MONTADO em {}]", path),
            Language::En => format!(" [MOUNTED at {}]", path),
        }
    }

    pub fn err_mount_fuse_fmt(&self, err: &str) -> String {
        match self.lang {
            Language::Pt => format!("Erro ao montar backup FUSE:
{}", err),
            Language::En => format!("Error mounting FUSE backup:
{}", err),
        }
    }

    pub fn err_umount_fmt(&self, err: &str) -> String {
        match self.lang {
            Language::Pt => format!("Erro ao desmontar:
{}", err),
            Language::En => format!("Error unmounting:
{}", err),
        }
    }

    pub fn err_systemd_dir_fmt(&self, err: &str) -> String {
        match self.lang {
            Language::Pt => format!("Erro ao criar diretório systemd: {}", err),
            Language::En => format!("Error creating systemd directory: {}", err),
        }
    }

    pub fn err_service_file_fmt(&self, err: &str) -> String {
        match self.lang {
            Language::Pt => format!("Erro ao salvar arquivo .service: {}", err),
            Language::En => format!("Error saving .service file: {}", err),
        }
    }

    pub fn err_timer_file_fmt(&self, err: &str) -> String {
        match self.lang {
            Language::Pt => format!("Erro ao salvar arquivo .timer: {}", err),
            Language::En => format!("Error saving .timer file: {}", err),
        }
    }

    pub fn msg_delete_success_fmt(&self, name: &str) -> String {
        match self.lang {
            Language::Pt => format!("Snapshot '{}' excluído com sucesso!", name),
            Language::En => format!("Snapshot '{}' deleted successfully!", name),
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
        assert_ne!(t_pt.diff_wizard_title(), t_en.diff_wizard_title());
        assert_ne!(t_pt.diff_view_title(), t_en.diff_view_title());
        assert_ne!(t_pt.diff_badge_added(), t_en.diff_badge_added());
        assert_ne!(t_pt.diff_badge_removed(), t_en.diff_badge_removed());
        assert_ne!(t_pt.profiles_title(), t_en.profiles_title());
        assert_ne!(t_pt.profiles_empty(), t_en.profiles_empty());
        assert_ne!(t_pt.profile_wizard_title(), t_en.profile_wizard_title());
        assert_ne!(t_pt.footer_profiles(), t_en.footer_profiles());
    }

    #[test]
    fn test_format_methods() {
        let t_pt = Translator::new(Language::Pt);
        let t_en = Translator::new(Language::En);

        let pt_fuse = t_pt.fuse_mounted_fmt("/mnt/test");
        let en_fuse = t_en.fuse_mounted_fmt("/mnt/test");
        assert!(pt_fuse.contains("Sim (em /mnt/test)"));
        assert!(en_fuse.contains("Yes (at /mnt/test)"));


        let pt_diff = t_pt.diff_summary_fmt(1, 2, 3, 4);
        let en_diff = t_en.diff_summary_fmt(1, 2, 3, 4);
        assert!(pt_diff.contains("Total: 10 alterações"));
        assert!(en_diff.contains("Total: 10 changes"));
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
