use rocket::FromForm;

#[derive(FromForm, Debug)]
pub struct InitFormData {
    pub volatile_penalty: usize,
    pub writethrough: bool,
    pub miss_penalty: usize,
    pub cache_data_set_bits: usize,
    pub cache_data_offset_bits: usize,
    pub cache_instruction_offset_bits: usize,
    pub cache_data_ways: usize,
    pub cache_instruction_set_bits: usize,
    pub cache_instruction_ways: usize,
    pub pipelining: bool,
}
