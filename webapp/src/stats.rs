use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Statistics {
    pub clocks: usize,
    pub memory_accesses: usize,
    pub cache_hits: usize,
    pub cache_conflict_misses: usize,
    pub cache_cold_misses: usize
}
