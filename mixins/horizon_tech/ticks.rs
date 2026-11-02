// Every Horizon Tech block entity ticks on the server.
fn horizon_tech_block_entity_ticks(block: &str, ticks: &mut bool) {
    crate::game::horizon_tech_glue::block_entity_ticks(block, ticks);
}
