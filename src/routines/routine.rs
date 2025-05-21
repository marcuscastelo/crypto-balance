#[derive(Debug, Clone)]
pub struct RoutineFailureInfo {
    pub message: String,
}

impl RoutineFailureInfo {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

pub type RoutineResult = Result<(), RoutineFailureInfo>;

#[async_trait::async_trait]
pub trait Routine {
    fn name(&self) -> &str;

    async fn run(&self) -> RoutineResult;
}
