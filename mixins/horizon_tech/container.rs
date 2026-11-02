// The generator's fuel slot and the machines' menus.
fn horizon_tech_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::horizon_tech_glue::container(block, container);
}
