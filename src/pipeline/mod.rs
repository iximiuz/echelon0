pub use pipeline::input_section::InputSection;
pub use pipeline::filter_section::FilterSection;
pub use pipeline::output_section::OutputSection;

mod input_section;
mod filter_section;
mod output_section;

pub struct Pipeline {
    inputs: InputSection,
    filters: FilterSection,
    outputs: OutputSection,
}

impl Pipeline {
    pub fn new(inputs: InputSection, filters: FilterSection, outputs: OutputSection) -> Pipeline {
        Pipeline {
            inputs: inputs,
            filters: filters,
            outputs: outputs,
        }
    }
}
