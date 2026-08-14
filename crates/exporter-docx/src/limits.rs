const MIB: u64 = 1024 * 1024;

/// Fail-closed limits shared by DOCX generation and subset validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocxLimits {
    pub max_output_bytes: u64,
    pub max_entries: usize,
    pub max_xml_bytes: u64,
    pub max_blocks: usize,
    pub max_paragraphs: usize,
    pub max_runs: usize,
    pub max_images: usize,
    pub max_image_bytes: u64,
    pub max_total_asset_bytes: u64,
    pub max_image_pixels: u64,
    pub max_image_dimension: u32,
    pub max_equation_depth: usize,
    pub max_equation_nodes: usize,
    pub max_equation_output_bytes: u64,
    pub max_entry_uncompressed_bytes: u64,
    pub max_total_uncompressed_bytes: u64,
    pub max_compression_ratio: u64,
    pub max_xml_depth: usize,
    pub max_xml_nodes: usize,
    pub max_part_name_bytes: usize,
}

impl Default for DocxLimits {
    fn default() -> Self {
        Self {
            max_output_bytes: 64 * MIB,
            max_entries: 4_096,
            max_xml_bytes: 16 * MIB,
            max_blocks: 100_000,
            max_paragraphs: 100_000,
            max_runs: 500_000,
            max_images: 10_000,
            max_image_bytes: 16 * MIB,
            max_total_asset_bytes: 64 * MIB,
            max_image_pixels: 100_000_000,
            max_image_dimension: 65_535,
            max_equation_depth: 256,
            max_equation_nodes: 100_000,
            max_equation_output_bytes: 4 * MIB,
            max_entry_uncompressed_bytes: 64 * MIB,
            max_total_uncompressed_bytes: 128 * MIB,
            max_compression_ratio: 100,
            max_xml_depth: 256,
            max_xml_nodes: 1_000_000,
            max_part_name_bytes: 1_024,
        }
    }
}
