// The energy buffers of generators, batteries and cables.
fn horizon_tech_energy(
    block: &str,
    energy: &mut Option<horizon_client::containers::ModEnergyBuffer>,
) {
    crate::game::horizon_tech_glue::energy(block, energy);
}
