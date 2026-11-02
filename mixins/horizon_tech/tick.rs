// A server tick of one of the mod's block entities.
fn horizon_tech_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::horizon_tech_glue::tick(game, pos, entity);
}
