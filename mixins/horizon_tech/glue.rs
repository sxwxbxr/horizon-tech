// Horizon Tech's edges in the game (not in the Cargo library): the hook
// points' arguments read into the runtime's types, and what the runtime
// decides done through the engine's helpers (block entities API 15-20,
// energy API 142).
use crate::game::Game;
use crate::game::horizon_tech as m;
use horizon_client::containers::{ModBlockEntity, ModContainer, ModEnergyBuffer};
use horizon_client::hud::HudBatch;
use horizon_client::mod_api::NativeScreen;
use horizon_core::geometry::Direction;
use horizon_items::items::ItemStack;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::cell::RefCell;
use std::collections::HashMap;

fn int(nbt: &Compound, key: &str) -> i32 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Int(v)) => *v,
        _ => 0,
    }
}

fn set_int(entity: &mut ModBlockEntity, key: &str, value: i32) {
    if int(&entity.nbt, key) != value {
        entity.nbt.insert(JavaString::from(key), Tag::Int(value));
    }
}

fn string(nbt: &Compound, key: &str) -> String {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::String(v)) => v.to_string().unwrap_or_default(),
        _ => String::new(),
    }
}

fn set_string(entity: &mut ModBlockEntity, key: &str, value: &str) {
    if string(&entity.nbt, key) != value {
        entity
            .nbt
            .insert(JavaString::from(key), Tag::String(JavaString::from(value)));
    }
}

fn set_data(entity: &mut ModBlockEntity, data: &[i32]) {
    if entity.data.as_slice() != data {
        entity.data = data.to_vec();
    }
}

pub fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && m::Kind::of(block).is_some() {
        *entity = Some((block.to_string(), Compound::new()));
    }
}

pub fn block_entity_ticks(block: &str, ticks: &mut bool) {
    if m::Kind::of(block).is_some() {
        *ticks = true;
    }
}

pub fn container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() {
        return;
    }
    let Some((menu, slots, _)) = m::Kind::of(block).and_then(m::Kind::menu) else {
        return;
    };
    // A generator's fuel slot takes items from any side, and a machine's
    // input; hoppers take a machine's results. The battery's one slot is
    // hidden by its screen and reached by no side.
    let (side, inputs, outputs) = match m::Kind::of(block) {
        Some(m::Kind::Generator(_)) => (vec![0], vec![0], Vec::new()),
        Some(m::Kind::Machine(_)) => (
            vec![m::INPUT_SLOT, m::OUTPUT_SLOT],
            vec![m::INPUT_SLOT],
            vec![m::OUTPUT_SLOT],
        ),
        _ => (Vec::new(), Vec::new(), Vec::new()),
    };
    *container = Some(ModContainer {
        slots,
        faces: std::array::from_fn(|_| side.clone()),
        inputs,
        outputs,
        menu: Some(menu.to_string()),
    });
}

pub fn energy(block: &str, energy: &mut Option<ModEnergyBuffer>) {
    if energy.is_some() {
        return;
    }
    let Some(kind) = m::Kind::of(block) else {
        return;
    };
    let b = kind.buffer();
    *energy = Some(ModEnergyBuffer {
        capacity: b.capacity,
        max_insert: b.max_insert,
        max_extract: b.max_extract,
        insert_faces: [b.max_insert > 0; 6],
        extract_faces: [b.max_extract > 0; 6],
    });
}

/// Room a neighbour on side `side` of `pos` has for energy coming from us.
fn room(game: &Game<'_>, dimension: &str, pos: [i32; 3], side: usize) -> i32 {
    let at = m::offset(pos, side);
    let face = Direction::ALL[m::opposite(side)];
    game.native_energy(dimension, at, face)
        .filter(|e| e.can_insert)
        .map_or(0, |e| (e.capacity - e.amount).min(e.max_insert).max(0))
}

/// Pushes up to `amount` out of the given sides, shared evenly; what went.
fn push(game: &mut Game<'_>, pos: [i32; 3], sides: &[usize], amount: i32) -> i32 {
    if amount <= 0 || sides.is_empty() {
        return 0;
    }
    let dimension = game.native_dimension();
    let rooms: Vec<i32> = sides
        .iter()
        .map(|&s| room(game, dimension, pos, s))
        .collect();
    let mut sent = 0;
    for (share, &side) in m::share(amount, &rooms).into_iter().zip(sides) {
        if share > 0 {
            let face = Direction::ALL[m::opposite(side)];
            sent += game.native_energy_insert(m::offset(pos, side), face, share, false);
        }
    }
    sent
}

/// The side a battery at `pos` gives energy out of: its front.
fn front(game: &Game<'_>, pos: [i32; 3]) -> Option<usize> {
    game.native_block_property(pos, "facing")
        .as_deref()
        .and_then(m::side_named)
}

pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(kind) = m::Kind::of(&entity.id) else {
        return;
    };
    match kind {
        m::Kind::Generator(tier) => generator(game, pos, entity, tier),
        m::Kind::Battery => battery(game, pos, entity),
        m::Kind::Cable => cable(game, pos, entity),
        m::Kind::Machine(machine) => self::machine(game, pos, entity, machine),
    }
}

/// Shows a furnace-like block lit or not (its `lit` state).
fn show_lit(game: &mut Game<'_>, pos: [i32; 3], lit: bool) {
    let lit = if lit { "true" } else { "false" };
    if game.native_block_property(pos, "lit").as_deref() != Some(lit) {
        game.native_set_block_property(pos, "lit", lit);
    }
}

fn generator(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity, tier: m::Tier) {
    let before = m::Generator {
        energy: int(&entity.nbt, m::ENERGY_KEY),
        burn: int(&entity.nbt, m::BURN_KEY),
        burn_total: int(&entity.nbt, m::BURN_TOTAL_KEY),
    };
    let fuel = entity
        .items
        .first()
        .and_then(|s| s.as_ref())
        .map(|s| s.item.to_string());
    let step = m::generator_tick(tier, before, fuel.as_deref());
    let mut state = step.state;
    if step.consumed {
        let slot = &mut entity.items[0];
        if let Some(stack) = slot {
            stack.count = stack.count.saturating_sub(1);
            if stack.count == 0 {
                *slot = None;
            }
        }
    }
    let out = state.energy.min(tier.push());
    state.energy -= push(game, pos, &[0, 1, 2, 3, 4, 5], out);
    set_int(entity, m::ENERGY_KEY, state.energy);
    set_int(entity, m::BURN_KEY, state.burn);
    set_int(entity, m::BURN_TOTAL_KEY, state.burn_total);
    set_data(entity, &m::generator_data(tier, state, step.burned));
    show_lit(game, pos, step.lit());
}

/// A machine recipe: its result and ticks.
type Found = Option<(ItemStack, i32)>;

/// What machines found for each input item, by recipe type, with the table
/// it was found in (a data pack's `/reload` makes a new one).
#[derive(Default)]
struct RecipeCache {
    table: (usize, usize, usize),
    found: HashMap<String, Found>,
}

thread_local! {
    static RECIPES: RefCell<HashMap<&'static str, RecipeCache>> = RefCell::new(HashMap::new());
}

/// The first recipe of `kind` whose ingredient takes `input`.
fn find_recipe(game: &Game<'_>, kind: &'static str, input: &ItemStack) -> Found {
    let recipes = game.native_recipes_of_type(kind);
    let table = (
        recipes.as_ptr() as usize,
        recipes.len(),
        recipes
            .first()
            .map_or(0, |r| std::sync::Arc::as_ptr(&r.json) as usize),
    );
    RECIPES.with(|cache| {
        let mut cache = cache.borrow_mut();
        let cache = cache.entry(kind).or_default();
        if cache.table != table {
            cache.table = table;
            cache.found.clear();
        }
        if let Some(found) = cache.found.get(&*input.item) {
            return found.clone();
        }
        let found = recipes.iter().find_map(|r| {
            let ingredient = game.native_recipe_ingredient(&r.json["ingredient"])?;
            if !ingredient.test(input) {
                return None;
            }
            let result = game.native_recipe_result(&r.json["result"])?;
            Some((result, m::recipe_time(r.json["time"].as_i64())))
        });
        cache.found.insert(input.item.to_string(), found.clone());
        found
    })
}

fn machine(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity, machine: m::Machine) {
    let mut state = m::MachineState {
        energy: int(&entity.nbt, m::ENERGY_KEY),
        progress: int(&entity.nbt, m::PROGRESS_KEY),
    };
    let input = entity.items.get(m::INPUT_SLOT).cloned().flatten();
    // Another item than the one being worked on starts over.
    let working = input.as_ref().map_or(String::new(), |s| s.item.to_string());
    if string(&entity.nbt, m::WORKING_KEY) != working {
        state.progress = 0;
        set_string(entity, m::WORKING_KEY, &working);
    }
    let recipe = input
        .as_ref()
        .and_then(|s| find_recipe(game, machine.recipe_type(), s));
    let output = entity.items.get(m::OUTPUT_SLOT).and_then(Option::as_ref);
    // A result holds no more than its item's stack (a bucket: 16, a tool: 1).
    let stack = recipe
        .as_ref()
        .map_or(64, |(r, _)| game.native_max_stack_size(&r.item).unwrap_or(1));
    let work = recipe.as_ref().map(|(result, time)| m::Work {
        time: *time,
        fits: m::output_fits(
            output.map(|o| (&*o.item, o.count)),
            &result.item,
            result.count,
            stack,
        ) && output.is_none_or(|o| o.damage == result.damage && o.extra == result.extra),
    });
    let tick = m::machine_tick(machine, state, work);
    if tick.done
        && let Some((result, _)) = &recipe
        && entity.items.len() > m::OUTPUT_SLOT
    {
        let slot = &mut entity.items[m::INPUT_SLOT];
        if let Some(stack) = slot {
            stack.count = stack.count.saturating_sub(1);
            if stack.count == 0 {
                *slot = None;
            }
        }
        match &mut entity.items[m::OUTPUT_SLOT] {
            Some(stack) => stack.count += result.count,
            slot => *slot = Some(result.clone()),
        }
    }
    set_int(entity, m::ENERGY_KEY, tick.state.energy);
    set_int(entity, m::PROGRESS_KEY, tick.state.progress);
    let time = work.map_or(0, |w| w.time);
    set_data(entity, &m::machine_data(machine, tick, time));
    show_lit(game, pos, tick.running);
}

fn battery(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let mut energy = int(&entity.nbt, m::ENERGY_KEY);
    if let Some(side) = front(game, pos) {
        let out = energy.min(m::BATTERY_RATE);
        energy -= push(game, pos, &[side], out);
    }
    set_int(entity, m::ENERGY_KEY, energy);
    set_data(entity, &m::battery_data(energy));
}

fn cable(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let mut energy = int(&entity.nbt, m::ENERGY_KEY);
    if energy <= 0 {
        return;
    }
    let dimension = game.native_dimension();
    // Machines first, then cables that hold less, so energy flows down the
    // line and never back into a battery's output.
    let mut machines = Vec::new();
    for side in 0..6 {
        let at = m::offset(pos, side);
        let block = game.native_block(at);
        if block == m::CABLE {
            continue;
        }
        if block == m::BATTERY && front(game, at) == Some(m::opposite(side)) {
            continue;
        }
        machines.push(side);
    }
    let out = energy.min(m::CABLE_RATE);
    energy -= push(game, pos, &machines, out);
    for side in 0..6 {
        let at = m::offset(pos, side);
        if energy <= 0 || game.native_block(at) != m::CABLE {
            continue;
        }
        let face = Direction::ALL[m::opposite(side)];
        let Some(theirs) = game.native_energy(dimension, at, face) else {
            continue;
        };
        let give = m::cable_level(energy, theirs.amount).min(m::CABLE_RATE);
        if give > 0 {
            energy -= game.native_energy_insert(at, face, give, false);
        }
    }
    set_int(entity, m::ENERGY_KEY, energy);
}

pub fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    if screen.is_some() {
        return;
    }
    use m::layout as l;
    let slots = match m::Kind::of(block) {
        Some(m::Kind::Generator(_)) => vec![Some(l::FUEL_SLOT)],
        Some(m::Kind::Battery) => vec![None],
        Some(m::Kind::Machine(_)) => vec![Some(l::INPUT_SLOT), Some(l::OUTPUT_SLOT)],
        _ => return,
    };
    *screen = Some(NativeScreen {
        width: l::WIDTH,
        height: l::HEIGHT,
        slots,
        inventory_y: l::INVENTORY_Y,
        inventory_x: l::INVENTORY_X,
        own_labels: false,
        own_background: false,
    });
}

fn get(data: &[i32], i: usize) -> i32 {
    data.get(i).copied().unwrap_or(0)
}

fn draw_bar(
    game: &Game<'_>,
    batch: &mut HudBatch,
    origin: [f32; 2],
    rect: [f32; 4],
    value: i32,
    max: i32,
) {
    let [x, y, w, h] = rect;
    let (x, y) = (origin[0] + x, origin[1] + y);
    game.native_fill(batch, [x - 1.0, y - 1.0, w + 2.0, h + 2.0], 0xFF37_3737);
    game.native_fill(batch, [x, y, w, h], 0xFF1B_1B1B);
    let filled = m::bar(value, max, h as i32) as f32;
    if filled > 0.0 {
        game.native_fill(batch, [x, y + h - filled, w, filled], 0xFFD8_3A2E);
        game.native_fill(batch, [x, y + h - filled, 2.0, filled], 0xFFF0_7C5A);
    }
}

pub(super) fn screen_draw(
    game: &Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut HudBatch,
    origin: [f32; 2],
) {
    use m::layout as l;
    let Some(kind) = m::Kind::of(block) else {
        return;
    };
    let energy = m::join32(get(data, 0), get(data, 1));
    let capacity = m::join32(get(data, 2), get(data, 3));
    let text = |batch: &mut HudBatch, s: &str, x: f32, y: f32| {
        game.native_text(batch, s, [origin[0] + x, origin[1] + y], 0x40_4040, false);
    };
    match kind {
        m::Kind::Generator(_) => {
            draw_bar(game, batch, origin, l::ENERGY_BAR, energy, capacity);
            let (burn, total) = (get(data, 4), get(data, 5));
            let [x, y, w, h] = l::FLAME;
            let lit = m::bar(burn, total, h as i32) as f32;
            game.native_fill(batch, [origin[0] + x, origin[1] + y, w, h], 0xFF8B_8B8B);
            if lit > 0.0 {
                game.native_fill(
                    batch,
                    [origin[0] + x, origin[1] + y + h - lit, w, lit],
                    0xFFFF_A51F,
                );
            }
            text(batch, &m::format_energy(energy), 8.0, 20.0);
            text(batch, &format!("+{} FE/t", get(data, 6)), 8.0, 32.0);
        }
        m::Kind::Battery => {
            draw_bar(game, batch, origin, l::BATTERY_BAR, energy, capacity);
            text(batch, &m::format_energy(energy), 8.0, 20.0);
            text(
                batch,
                &format!("/ {}", m::format_energy(capacity)),
                8.0,
                32.0,
            );
        }
        m::Kind::Machine(_) => {
            draw_bar(game, batch, origin, l::ENERGY_BAR, energy, capacity);
            let (progress, time) = (get(data, 4), get(data, 5));
            let [x, y, w, h] = l::ARROW;
            let (x, y) = (origin[0] + x, origin[1] + y);
            game.native_fill(batch, [x, y, w, h], 0xFF8B_8B8B);
            let done = m::bar(progress, time, w as i32) as f32;
            if done > 0.0 {
                game.native_fill(batch, [x, y, done, h], 0xFFFF_FFFF);
            }
            text(batch, &m::format_energy(energy), 8.0, 20.0);
            text(batch, &format!("-{} FE/t", get(data, 6)), 8.0, 58.0);
        }
        m::Kind::Cable => {}
    }
}
