use pgrx::{
    item_pointer_get_both,
    pg_sys::{BlockNumber, ItemPointerData, OffsetNumber},
};
use std::collections::HashMap;
use tantivy::DocAddress;

static mut MANAGER: Manager = Manager::new();

pub fn get_current_executor_manager() -> &'static mut Manager {
    unsafe { &mut MANAGER }
}

pub fn get_fresh_executor_manager() -> &'static mut Manager {
    // We should call this at the top of a scan to clear out the manager memory.
    // Otherwise, the static manager could grow unbound and leak memory.
    unsafe {
        MANAGER = Manager::new();
        &mut MANAGER
    }
}

type BlockInfo = (BlockNumber, OffsetNumber);

pub struct Manager {
    max_score: f32,
    min_score: f32,
    scores: Option<HashMap<BlockInfo, f32>>,
    doc_addresses: Option<HashMap<BlockInfo, DocAddress>>,
}

impl Manager {
    pub const fn new() -> Self {
        Self {
            scores: None,
            max_score: 0.0,
            min_score: 0.0,
            doc_addresses: None,
        }
    }

    pub fn add_score(&mut self, ctid: (BlockNumber, OffsetNumber), score: f32) {
        if self.scores.is_none() {
            self.scores.replace(HashMap::new());
        }

        self.scores.as_mut().unwrap().insert(ctid, score);
    }

    pub fn get_score(&mut self, ctid: ItemPointerData) -> Option<f32> {
        let (block, offset) = item_pointer_get_both(ctid);
        self.scores.as_mut().unwrap().get(&(block, offset)).copied()
    }

    pub fn set_max_score(&mut self, max_score: f32) {
        self.max_score = max_score;
    }

    pub fn get_max_score(&self) -> f32 {
        self.max_score
    }

    pub fn set_min_score(&mut self, min_score: f32) {
        self.min_score = min_score;
    }

    pub fn get_min_score(&self) -> f32 {
        self.min_score
    }

    pub fn add_doc_address(&mut self, ctid: (BlockNumber, OffsetNumber), doc_address: DocAddress) {
        if self.doc_addresses.is_none() {
            self.doc_addresses.replace(HashMap::new());
        }

        self.doc_addresses
            .as_mut()
            .unwrap()
            .insert(ctid, doc_address);
    }
}
