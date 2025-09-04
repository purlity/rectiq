// rectiq-cli/src/controller/fix_flow.rs
use crate::{
    controller::{fix_encoder::FixEncoder, masking::to_masked_owned_envelope},
    remote::{fix_transmitter::FixTransmitter, symbolset_fetcher::fetch_symbolset_from_server},
};
use rectiq_types::SketchNode;

pub struct FixFlowController;

impl FixFlowController {
    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn execute(
        transmitter: &FixTransmitter,
        envelopes: &[SketchNode],
    ) -> Result<String, String> {
        // 1) Mask+own envelopes before any serialization or transport
        let masked: Vec<_> = envelopes.iter().map(to_masked_owned_envelope).collect();

        // 2) SymbolSet comes from server and can depend on envelope kinds/meta
        let (symbolset, _meta) = fetch_symbolset_from_server(&transmitter.user_id, &masked)?;

        // 3) Encode using the masked envelopes
        let encoded = FixEncoder::encode_all(&symbolset, &masked)?;

        let encrypted = transmitter
            .send_with(&symbolset, &encoded)
            .map_err(|e| e.to_string())?;
        Ok(encrypted)
    }
}
