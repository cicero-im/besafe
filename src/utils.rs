#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Untracked,      // ??
    Staged,         // A
    Modified,       // M
    StagedModified, // AM
    Unknown,
}

pub fn code_to_enum(code: &str) -> Status {
    match code {
        "??" => Status::Untracked,
        "A" => Status::Staged,
        "M" => Status::Modified,
        "AM" => Status::StagedModified,
        _ => Status::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{code_to_enum, Status};

    #[test]
    fn maps_known_statuses() {
        assert_eq!(code_to_enum("??"), Status::Untracked);
        assert_eq!(code_to_enum("A"), Status::Staged);
        assert_eq!(code_to_enum("M"), Status::Modified);
        assert_eq!(code_to_enum("AM"), Status::StagedModified);
    }

    #[test]
    fn maps_unknown_statuses() {
        assert_eq!(code_to_enum("MM"), Status::Unknown);
        assert_eq!(code_to_enum(""), Status::Unknown);
    }
}
