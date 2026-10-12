use process_mining::core::process_models::case_centric::petri_net::PetriNet;

/// A local process model: a Petri net identified by its name.
#[derive(Debug, Clone)]
pub struct Lpm {
    /// File stem of the PNML the net was read from. Unique within one archive.
    pub name: String,
    pub net: PetriNet,
}