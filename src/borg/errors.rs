use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorgError {
    BorgNotFound(String),
    RepositoryLocked(String),
    InvalidPassphrase(String),
    RepositoryNotFound(String),
    CommandFailed {
        exit_code: Option<i32>,
        message: String,
    },
    IoError(String),
    JsonParseError(String),
}

impl BorgError {
    pub fn from_stderr(exit_code: Option<i32>, stderr: &str) -> Self {
        let trimmed = stderr.trim();
        if trimmed.contains("is already locked")
            || trimmed.contains("LockFailed")
            || trimmed.contains("Failed to create/acquire the lock")
        {
            BorgError::RepositoryLocked(trimmed.to_string())
        } else if trimmed.contains("Passphrase")
            || trimmed.contains("passphrase")
            || trimmed.contains("key wrong")
        {
            BorgError::InvalidPassphrase(trimmed.to_string())
        } else if trimmed.contains("Repository does not exist")
            || trimmed.contains("does not look like a Borg backup repository")
        {
            BorgError::RepositoryNotFound(trimmed.to_string())
        } else if trimmed.contains("borg: command not found")
            || trimmed.contains("No such file or directory")
        {
            BorgError::BorgNotFound(trimmed.to_string())
        } else {
            BorgError::CommandFailed {
                exit_code,
                message: trimmed.to_string(),
            }
        }
    }
}

impl fmt::Display for BorgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorgError::BorgNotFound(msg) => {
                write!(f, "Borg não encontrado no sistema: {}", msg)
            }
            BorgError::RepositoryLocked(msg) => {
                write!(
                    f,
                    "Repositório bloqueado por outro processo:\n{}\n\nDica: Se nenhuma operação estiver em execução, use 'borg break-lock'.",
                    msg
                )
            }
            BorgError::InvalidPassphrase(msg) => {
                write!(f, "Frase secreta incorreta ou ausente: {}", msg)
            }
            BorgError::RepositoryNotFound(msg) => {
                write!(f, "Repositório não encontrado ou inacessível: {}", msg)
            }
            BorgError::CommandFailed { message, .. } => {
                write!(f, "{}", message)
            }
            BorgError::IoError(msg) => {
                write!(f, "Erro de E/S: {}", msg)
            }
            BorgError::JsonParseError(msg) => {
                write!(f, "Erro ao processar dados JSON do Borg: {}", msg)
            }
        }
    }
}

impl std::error::Error for BorgError {}
