use std::env;
use sysinfo::System;

pub fn check_instances() -> Result<(), String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut borg_count = 0;

    let current_pid = std::process::id();
    let current_exe_name = env::current_exe()
        .map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .unwrap_or_else(|_| "reborg".to_string());

    let mut our_app_count = 0;

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();

        if name == "borg" {
            borg_count += 1;
        }

        if (name == current_exe_name || name == "rsborg" || name == "reborg")
            && pid.as_u32() != current_pid {
                our_app_count += 1;
            }
    }

    if our_app_count > 0 {
        return Err("Já existe outra instância do RsBorg rodando no sistema!".to_string());
    }

    if borg_count > 0 {
        return Err("O sistema detectou que o aplicativo original 'borg' já está em execução em segundo plano ou em outro terminal. Aguarde finalizar.".to_string());
    }

    Ok(())
}
