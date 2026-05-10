use std::{collections::HashSet, sync::Arc};

use crate::terminal::model::terminal_model::BlockIndex;
use warp_core::command::ExitCode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AskAIType {
    FromTextSelection {
        text: Arc<String>,
        populate_input_box: bool,
    },
    FromBlock {
        input: Arc<String>,
        output: Arc<String>,
        exit_code: ExitCode,
        block_index: BlockIndex,
    },
    FromBlocks {
        block_indices: HashSet<BlockIndex>,
    },
    FromAICommandSearch {
        query: Arc<String>,
    },
}
