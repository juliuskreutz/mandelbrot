use std::error::Error;

use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("../shader", "spirv-unknown-spv1.5")
        .print_metadata(MetadataPrintout::Full)
        .capability(Capability::Float64)
        .build()?;

    Ok(())
}
