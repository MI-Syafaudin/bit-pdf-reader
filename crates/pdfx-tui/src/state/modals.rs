#[derive(Debug, Clone, Default)]
pub struct JumpToPageState {
    pub input: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RotateModalState {
    pub angle: u16,
    pub apply_all: bool,
}

impl Default for RotateModalState {
    fn default() -> Self {
        Self {
            angle: 90,
            apply_all: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ErrorModalState {
    pub title: String,
    pub message: String,
}
