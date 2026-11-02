// Horizon Tech's game logic, free of Horizon's types so it builds and tests
// as a plain Cargo library. The same file is injected into the game as
// `crate::game::horizon_tech`; `glue.rs` connects it to the hook points.
// Module payloads can't carry inner attributes or `//!` comments.

pub const ID: &str = "horizon_tech";
pub const NAME: &str = "Horizon Tech";

pub const GENERATOR: &str = "horizon_tech:coal_generator";
pub const BATTERY: &str = "horizon_tech:battery";
pub const CABLE: &str = "horizon_tech:energy_cable";

/// The menu types in `data/horizon_tech/horizon/menu/`.
pub const GENERATOR_MENU: &str = "horizon_tech:generator";
pub const BATTERY_MENU: &str = "horizon_tech:battery";

/// The NBT key the engine keeps a block's energy under (`SimpleEnergyHandler`).
pub const ENERGY_KEY: &str = "energy";
pub const BURN_KEY: &str = "burn";
pub const BURN_TOTAL_KEY: &str = "burn_total";

/// What kind of machine a block id is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Generator,
    Battery,
    Cable,
}

impl Kind {
    pub fn of(block: &str) -> Option<Kind> {
        match block {
            GENERATOR => Some(Kind::Generator),
            BATTERY => Some(Kind::Battery),
            CABLE => Some(Kind::Cable),
            _ => None,
        }
    }

    /// The buffer every block of this kind has: capacity, most in and most
    /// out per call.
    pub fn buffer(self) -> Buffer {
        match self {
            Kind::Generator => Buffer {
                capacity: 50_000,
                max_insert: 0,
                max_extract: GENERATOR_PUSH,
            },
            Kind::Battery => Buffer {
                capacity: 1_000_000,
                max_insert: BATTERY_RATE,
                max_extract: BATTERY_RATE,
            },
            Kind::Cable => Buffer {
                capacity: CABLE_RATE * 2,
                max_insert: CABLE_RATE,
                max_extract: CABLE_RATE,
            },
        }
    }

    /// The menu the block opens, with its slot and data slot counts.
    pub fn menu(self) -> Option<(&'static str, usize, usize)> {
        match self {
            Kind::Generator => Some((GENERATOR_MENU, 1, GENERATOR_DATA)),
            Kind::Battery => Some((BATTERY_MENU, 1, BATTERY_DATA)),
            Kind::Cable => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buffer {
    pub capacity: i32,
    pub max_insert: i32,
    pub max_extract: i32,
}

/// Energy (FE) a burning generator makes each tick.
pub const GENERATOR_RATE: i32 = 40;
/// The most a generator pushes out each tick, over all its sides.
pub const GENERATOR_PUSH: i32 = 400;
/// The most a battery takes in or gives out each tick.
pub const BATTERY_RATE: i32 = 2_000;
/// The most a cable moves each tick.
pub const CABLE_RATE: i32 = 1_000;

/// The generator menu's data slots: energy (2), capacity (2), burn, burn total.
pub const GENERATOR_DATA: usize = 6;
/// The battery menu's data slots: energy (2), capacity (2).
pub const BATTERY_DATA: usize = 4;

/// Ticks one item burns in a generator, as a furnace burns it; 0 for no fuel.
pub fn burn_ticks(item: &str) -> i32 {
    let Some(name) = item.strip_prefix("minecraft:") else {
        return 0;
    };
    match name {
        "coal" | "charcoal" => 1_600,
        "coal_block" => 16_000,
        "blaze_rod" => 2_400,
        "dried_kelp_block" => 4_001,
        "stick" => 100,
        "bamboo" | "scaffolding" => 50,
        _ if name.ends_with("_planks") || name.ends_with("_log") || name.ends_with("_wood") => 300,
        _ if name.ends_with("_stem") || name.ends_with("_hyphae") => 300,
        _ => 0,
    }
}

/// A generator's state between ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Generator {
    pub energy: i32,
    /// Ticks the current fuel still burns.
    pub burn: i32,
    /// Ticks the current fuel burned in all, for the flame.
    pub burn_total: i32,
}

/// What one generator tick did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeneratorTick {
    pub state: Generator,
    /// One item of the fuel slot was used.
    pub consumed: bool,
}

impl GeneratorTick {
    pub fn lit(&self) -> bool {
        self.state.burn > 0
    }
}

/// One tick: burn what is burning, light the next fuel when the buffer has
/// room for what it makes. It never makes more than the buffer holds.
pub fn generator_tick(mut state: Generator, fuel: Option<&str>) -> GeneratorTick {
    let capacity = Kind::Generator.buffer().capacity;
    let mut consumed = false;
    if state.burn <= 0 && state.energy + GENERATOR_RATE <= capacity {
        let ticks = fuel.map_or(0, burn_ticks);
        if ticks > 0 {
            state.burn = ticks;
            state.burn_total = ticks;
            consumed = true;
        }
    }
    if state.burn > 0 {
        state.burn -= 1;
        state.energy = (state.energy + GENERATOR_RATE).min(capacity);
        if state.burn == 0 {
            state.burn_total = 0;
        }
    }
    GeneratorTick { state, consumed }
}

/// `amount` shared out over targets that each take up to their `room`, as
/// evenly as their room allows. The shares never add up to more than
/// `amount`, and none is more than its room.
pub fn share(amount: i32, rooms: &[i32]) -> Vec<i32> {
    let mut shares = vec![0; rooms.len()];
    let mut left = amount.max(0);
    let mut open: Vec<usize> = (0..rooms.len()).filter(|&i| rooms[i] > 0).collect();
    while left > 0 && !open.is_empty() {
        let each = (left / open.len() as i32).max(1);
        let mut still = Vec::with_capacity(open.len());
        for &i in &open {
            if left == 0 {
                break;
            }
            let give = each.min(rooms[i] - shares[i]).min(left);
            shares[i] += give;
            left -= give;
            if shares[i] < rooms[i] {
                still.push(i);
            }
        }
        open = still;
    }
    shares
}

/// What a cable gives a neighbouring cable: half the difference, so a
/// line of cables levels out instead of passing energy back and forth.
pub fn cable_level(own: i32, neighbour: i32) -> i32 {
    ((own - neighbour) / 2).max(0)
}

/// The six sides in the engine's `Direction` order (vanilla's): down, up,
/// north, south, west, east, as offsets.
pub const SIDES: [[i32; 3]; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];

/// The side across from side `i` (as `Direction.getOpposite`).
pub fn opposite(i: usize) -> usize {
    i ^ 1
}

pub fn offset(pos: [i32; 3], side: usize) -> [i32; 3] {
    let d = SIDES[side];
    [pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]]
}

/// The side number a `facing` property value names.
pub fn side_named(facing: &str) -> Option<usize> {
    match facing {
        "down" => Some(0),
        "up" => Some(1),
        "north" => Some(2),
        "south" => Some(3),
        "west" => Some(4),
        "east" => Some(5),
        _ => None,
    }
}

/// A 32-bit value as two 16-bit data slots (data slots are 16-bit on the
/// wire, as in vanilla).
pub fn split32(value: i32) -> [i32; 2] {
    let v = value.max(0) as u32;
    [(v & 0xFFFF) as i32, (v >> 16) as i32]
}

/// The value `split32` sent, whatever sign the slots came back with.
pub fn join32(lo: i32, hi: i32) -> i32 {
    (((hi as u32 & 0xFFFF) << 16) | (lo as u32 & 0xFFFF)) as i32
}

/// The menu data of a generator.
pub fn generator_data(state: Generator) -> [i32; GENERATOR_DATA] {
    let [e0, e1] = split32(state.energy);
    let [c0, c1] = split32(Kind::Generator.buffer().capacity);
    [e0, e1, c0, c1, state.burn, state.burn_total]
}

/// The menu data of a battery.
pub fn battery_data(energy: i32) -> [i32; BATTERY_DATA] {
    let [e0, e1] = split32(energy);
    let [c0, c1] = split32(Kind::Battery.buffer().capacity);
    [e0, e1, c0, c1]
}

/// Energy for people: `950 FE`, `12.5 kFE`, `1.00 MFE`.
pub fn format_energy(fe: i32) -> String {
    let fe = fe.max(0) as f64;
    if fe >= 1_000_000.0 {
        format!("{:.2} MFE", fe / 1_000_000.0)
    } else if fe >= 1_000.0 {
        format!("{:.1} kFE", fe / 1_000.0)
    } else {
        format!("{fe} FE")
    }
}

/// How many of `pixels` a bar fills for `value` of `max`.
pub fn bar(value: i32, max: i32, pixels: i32) -> i32 {
    if max <= 0 {
        return 0;
    }
    ((value.clamp(0, max) as i64 * pixels as i64) / max as i64) as i32
}

/// Screen layout, in GUI pixels from the screen's top left (vanilla's 176
/// by 166 container screens).
pub mod layout {
    pub const WIDTH: f32 = 176.0;
    pub const HEIGHT: f32 = 166.0;
    pub const INVENTORY_X: f32 = 8.0;
    pub const INVENTORY_Y: f32 = 84.0;
    /// The generator's fuel slot.
    pub const FUEL_SLOT: [f32; 2] = [80.0, 53.0];
    /// The flame above it: x, y, width, height.
    pub const FLAME: [f32; 4] = [81.0, 36.0, 14.0, 14.0];
    /// The energy bar: x, y, width, height.
    pub const ENERGY_BAR: [f32; 4] = [152.0, 17.0, 16.0, 54.0];
    /// The battery's wide bar.
    pub const BATTERY_BAR: [f32; 4] = [62.0, 20.0, 52.0, 50.0];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_are_known_by_id() {
        assert_eq!(Kind::of(GENERATOR), Some(Kind::Generator));
        assert_eq!(Kind::of(CABLE), Some(Kind::Cable));
        assert_eq!(Kind::of("minecraft:furnace"), None);
    }

    #[test]
    fn fuels_burn_as_long_as_in_a_furnace() {
        assert_eq!(burn_ticks("minecraft:coal"), 1_600);
        assert_eq!(burn_ticks("minecraft:oak_planks"), 300);
        assert_eq!(burn_ticks("minecraft:crimson_stem"), 300);
        assert_eq!(burn_ticks("minecraft:diamond"), 0);
        assert_eq!(burn_ticks("othermod:coal"), 0);
    }

    #[test]
    fn generator_lights_fuel_and_makes_energy() {
        let tick = generator_tick(Generator::default(), Some("minecraft:coal"));
        assert!(tick.consumed);
        assert!(tick.lit());
        assert_eq!(tick.state.energy, GENERATOR_RATE);
        assert_eq!(tick.state.burn, 1_599);
        assert_eq!(tick.state.burn_total, 1_600);
    }

    #[test]
    fn generator_keeps_burning_without_new_fuel() {
        let state = Generator {
            energy: 0,
            burn: 10,
            burn_total: 1_600,
        };
        let tick = generator_tick(state, Some("minecraft:coal"));
        assert!(!tick.consumed);
        assert_eq!(tick.state.burn, 9);
    }

    #[test]
    fn generator_goes_out_after_the_last_tick() {
        let state = Generator {
            energy: 0,
            burn: 1,
            burn_total: 100,
        };
        let tick = generator_tick(state, None);
        assert!(!tick.lit());
        assert_eq!(tick.state.burn_total, 0);
        assert_eq!(tick.state.energy, GENERATOR_RATE);
    }

    #[test]
    fn full_generator_keeps_its_fuel() {
        let full = Kind::Generator.buffer().capacity;
        let tick = generator_tick(
            Generator {
                energy: full,
                burn: 0,
                burn_total: 0,
            },
            Some("minecraft:coal"),
        );
        assert!(!tick.consumed);
        assert!(!tick.lit());
        assert_eq!(tick.state.energy, full);
    }

    #[test]
    fn generator_never_overfills() {
        let capacity = Kind::Generator.buffer().capacity;
        let state = Generator {
            energy: capacity - 10,
            burn: 50,
            burn_total: 100,
        };
        assert_eq!(generator_tick(state, None).state.energy, capacity);
    }

    #[test]
    fn share_is_even_and_respects_room() {
        assert_eq!(share(90, &[100, 100, 100]), vec![30, 30, 30]);
        assert_eq!(share(90, &[10, 100, 100]), vec![10, 40, 40]);
        assert_eq!(share(5, &[0, 0]), vec![0, 0]);
        assert_eq!(share(0, &[10]), vec![0]);
        let shares = share(7, &[5, 5, 5]);
        assert_eq!(shares.iter().sum::<i32>(), 7);
        assert!(shares.iter().all(|&s| s <= 5));
    }

    #[test]
    fn share_never_gives_more_than_it_has() {
        let shares = share(10, &[100, 3, 100, 1]);
        assert_eq!(shares.iter().sum::<i32>(), 10);
        assert!(shares[1] <= 3 && shares[3] <= 1);
    }

    #[test]
    fn cables_level_out() {
        assert_eq!(cable_level(1_000, 200), 400);
        assert_eq!(cable_level(200, 1_000), 0);
        assert_eq!(cable_level(5, 4), 0);
    }

    #[test]
    fn sides_are_vanillas_and_opposite() {
        assert_eq!(opposite(0), 1);
        assert_eq!(opposite(2), 3);
        assert_eq!(opposite(5), 4);
        for i in 0..6 {
            let back = offset(offset([0, 0, 0], i), opposite(i));
            assert_eq!(back, [0, 0, 0]);
        }
        assert_eq!(side_named("north"), Some(2));
        assert_eq!(offset([0, 64, 0], side_named("east").unwrap()), [1, 64, 0]);
    }

    #[test]
    fn big_values_survive_16_bit_slots() {
        for v in [0, 1, 40_000, 65_535, 65_536, 1_000_000, i32::MAX] {
            let [lo, hi] = split32(v);
            // A client may read a slot back sign-extended from 16 bits.
            let (lo16, hi16) = (lo as u16 as i16 as i32, hi as u16 as i16 as i32);
            assert_eq!(join32(lo, hi), v);
            assert_eq!(join32(lo16, hi16), v);
        }
    }

    #[test]
    fn menu_data_carries_energy_and_capacity() {
        let data = battery_data(123_456);
        assert_eq!(join32(data[0], data[1]), 123_456);
        assert_eq!(join32(data[2], data[3]), 1_000_000);
        let data = generator_data(Generator {
            energy: 7,
            burn: 3,
            burn_total: 9,
        });
        assert_eq!(data[4..], [3, 9]);
    }

    #[test]
    fn energy_reads_well() {
        assert_eq!(format_energy(950), "950 FE");
        assert_eq!(format_energy(12_500), "12.5 kFE");
        assert_eq!(format_energy(1_000_000), "1.00 MFE");
    }

    #[test]
    fn bars_fill_in_proportion() {
        assert_eq!(bar(0, 100, 50), 0);
        assert_eq!(bar(50, 100, 50), 25);
        assert_eq!(bar(500, 100, 50), 50);
        assert_eq!(bar(1_000_000, 1_000_000, 54), 54);
        assert_eq!(bar(5, 0, 50), 0);
    }
}
