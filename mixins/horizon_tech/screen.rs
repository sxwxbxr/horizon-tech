// Lays out a machine's screen (asked on the client and the server).
fn horizon_tech_screen(block: &str, screen: &mut Option<horizon_client::mod_api::NativeScreen>) {
    crate::game::horizon_tech_glue::screen(block, screen);
}
