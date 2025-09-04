// rectiq-cli/src/local_coordinator.rs
use crate::{
    SketchOrchestrator, controller::fix_flow::FixFlowController, fix_executor::FixExecutor,
    remote::fix_transmitter::FixTransmitter, types::scan::LocalScan,
};

pub struct LocalCoordinator<'a> {
    sketcher: SketchOrchestrator<'a>,
}

impl Default for LocalCoordinator<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalCoordinator<'_> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sketcher: SketchOrchestrator::default(),
        }
    }

    pub fn scan<'b>(&'b mut self, input: &'b str) -> LocalScan<'b> {
        self.sketcher.run(input)
    }

    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn run_with_api(&mut self, input: &str, user_id: &str) -> Result<String, String> {
        let sketches = self.scan(input).sketches;
        println!("==== SKETCHES ====");
        println!("{sketches:?}");
        let transmitter = FixTransmitter::initialize(user_id)?;
        let encrypted = FixFlowController::execute(&transmitter, &sketches)?;
        let key = transmitter.extract_key_base64();
        let output = FixExecutor::apply_from_blob(input, &encrypted, &key)?;

        Ok(output)
    }
}
