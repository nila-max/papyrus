/// The central marker is the first block number that doesn't exist yet.
pub const PAPYRUS_CENTRAL_BLOCK_MARKER: &str = "papyrus_central_block_marker";

/// The header marker is the first block number for which the node does not have a header.
pub const PAPYRUS_HEADER_MARKER: &str = "papyrus_header_marker";

/// The body marker is the first block number for which the node does not have a body.
pub const PAPYRUS_BODY_MARKER: &str = "papyrus_body_marker";

/// The state marker is the first block number for which the node does not have a state body.
pub const PAPYRUS_STATE_MARKER: &str = "papyrus_state_marker";

/// The compiled class marker is the first block number for which the node does not have all of the
/// corresponding compiled classes.
pub const PAPYRUS_COMPILED_CLASS_MARKER: &str = "papyrus_compiled_class_marker";

/// The base layer marker is the first block number for which the node does not guerantee L1
/// finality.
pub const PAPYRUS_BASE_LAYER_MARKER: &str = "papyrus_base_layer_marker";