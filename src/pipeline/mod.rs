pub use self::input_section::InputSection;
pub use self::filter_section::FilterSection;
pub use self::output_section::OutputSection;

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
