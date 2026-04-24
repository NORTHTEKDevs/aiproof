#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Info => 0,
            Self::Warning => 1,
            Self::Error => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_and_ordering() {
        assert_eq!(Severity::Info.exit_code(), 0);
        assert_eq!(Severity::Warning.exit_code(), 1);
        assert_eq!(Severity::Error.exit_code(), 2);
        assert!(Severity::Error > Severity::Warning);
    }
}
