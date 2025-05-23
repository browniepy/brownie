#[derive(Debug)]
pub enum WorkError {
    AlreadyEmployed,
    NotEmployed,
    JobNotFound,
    CannotApply,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for WorkError {
    fn from(err: sqlx::Error) -> Self {
        WorkError::Database(err)
    }
}

impl std::fmt::Display for WorkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkError::AlreadyEmployed => write!(f, "Already employed"),
            WorkError::NotEmployed => write!(f, "Not employed"),
            WorkError::JobNotFound => write!(f, "Job not found"),
            WorkError::CannotApply => write!(f, "Cannot apply"),
            WorkError::Database(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl std::error::Error for WorkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WorkError::Database(err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ClubError {
    NotRenameable,
    NotDeletable,
    RoleAlreadyExists,
    RoleNotFound,
    CannotDeleteRole,
    InvalidAuthority,
    DuplicateAuthority,
    ItemAlreadyExists,
    ItemNotFound,
    MemberAlreadyHasRole,
    MemberLimitReached,
    MemberNotFound,
    MemberAlreadyExists,
    InsufficientFunds,
    InsufficientPermissions,
    AlreadyLeader,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for ClubError {
    fn from(err: sqlx::Error) -> Self {
        ClubError::Database(err)
    }
}

impl std::fmt::Display for ClubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClubError::NotRenameable => write!(f, "Not renameable"),
            ClubError::NotDeletable => write!(f, "Not deletable"),
            ClubError::RoleAlreadyExists => write!(f, "Role already exists"),
            ClubError::RoleNotFound => write!(f, "Role not found"),
            ClubError::CannotDeleteRole => write!(f, "Cannot delete role"),
            ClubError::InvalidAuthority => write!(f, "Invalid authority"),
            ClubError::DuplicateAuthority => write!(f, "Duplicate authority"),
            ClubError::ItemAlreadyExists => write!(f, "Item already exists"),
            ClubError::ItemNotFound => write!(f, "Item not found"),
            ClubError::MemberAlreadyHasRole => write!(f, "Member already has role"),
            ClubError::MemberLimitReached => write!(f, "Member limit reached"),
            ClubError::MemberNotFound => write!(f, "Member not found"),
            ClubError::MemberAlreadyExists => write!(f, "Member already exists"),
            ClubError::InsufficientFunds => write!(f, "Insufficient funds"),
            ClubError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            ClubError::AlreadyLeader => write!(f, "Already leader"),
            ClubError::Database(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl std::error::Error for ClubError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClubError::Database(err) => Some(err),
            _ => None,
        }
    }
}
