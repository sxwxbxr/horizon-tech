// A Horizon Tech block gets its block entity.
fn horizon_tech_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    crate::game::horizon_tech_glue::block_entity(block, entity);
}
