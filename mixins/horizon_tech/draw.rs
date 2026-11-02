// Draws a machine's energy bar, flame and numbers.
fn horizon_tech_screen_draw(
    game: &crate::game::Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut horizon_client::hud::HudBatch,
    origin: [f32; 2],
) {
    crate::game::horizon_tech_glue::screen_draw(game, block, data, batch, origin);
}
