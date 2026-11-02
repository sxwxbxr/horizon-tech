# Horizon Tech

A tech mod for [Horizon](https://github.com/horizon-rust/horizon) (Minecraft Java 26.3 in Rust): power generation,
ore processing, quarries and storage networks. Inspired by Mekanism, Powah, BuildCraft and Refined Storage, with its
own names, textures and numbers.

## Roadmap

1. **Power system**: coal generator, energy cables, battery, all on Horizon's energy API (FE).
2. **Power tiers and ore processing** (this release): generators in three tiers, cobalt ore in the world, a crusher
   that makes two dusts of every ore and an induction smelter that turns them into ingots.
3. Quarry: mines an area with power, delivers to a chest or the network.
4. Storage network: controller, drives, terminal with search.

## Blocks

| Block | Recipe | What it does |
|---|---|---|
| Coal Generator | iron, copper, furnace, redstone | Burns furnace fuels (coal, charcoal, wood, blaze rods, wool... not lava) as long as a furnace would, for 40 FE/t; stores 50 kFE and pauses the fire, keeping the fuel, while full; pushes up to 400 FE/t into every side |
| Energy Cable (x8) | copper, redstone, copper | Carries up to 1 kFE/t; machines first, then levels out along the line |
| Battery | iron, copper, redstone blocks, copper block | Stores 1 MFE; takes energy on every side, gives up to 2 kFE/t out of its front |
| Reinforced Generator | coal generator, cobalt ingots, iron, redstone block | Burns furnace fuels twice as fast for 120 FE/t (60 FE per fuel tick, 1.5x a coal generator); stores 200 kFE |
| Overcharged Generator | reinforced generator, cobalt blocks, diamonds, redstone block | Five times as fast for 400 FE/t (80 FE per fuel tick, 2x); stores 1 MFE |
| Crusher | iron, piston, cobalt ingots, grindstone, redstone | 20 FE/t, 5 s an item: ore or raw ore → 2 dusts, raw block → 18 dusts, stone/cobblestone → gravel → sand, bone → 6 bone meal, blaze rod → 4 powder |
| Induction Smelter | iron, cobalt ingots, blast furnace, redstone | 30 FE/t, 5 s an item: dust, raw ore or ore → ingot, sand → glass, cobblestone → stone |
| Cobalt Ore | (world) | Overworld, most common around y -16, small veins up to y 64; needs an iron pickaxe; drops raw cobalt (Fortune works) |

Machines take items from hoppers on every side into their input and give their results to hoppers below or beside
them. Their recipes are data (`horizon_tech:crushing` and `horizon_tech:smelting` in `data/<ns>/recipe/`), so a data
pack or another mod can add its own:

```json
{ "type": "horizon_tech:crushing", "ingredient": "#c:ores/tin", "result": { "id": "othermod:tin_dust", "count": 2 }, "time": 100 }
```

Every dust also smelts in a furnace or blast furnace, and the cobalt ingot, dusts, raw cobalt and ores carry the
shared `c:` tags (`c:ingots/cobalt`, `c:dusts/iron`, ...).

Every buffer is Horizon's own (`native_block_entity_energy`, API 142), so other mods' machines and cables can take
and give energy too.

## Layout

```
horizon-mod.toml, assets/, data/   the content: blocks, items, models, textures, recipes, loot, tags, menus
mixins/horizon_tech/               the native code
  runtime.rs                       the logic and its tests (also this repo's Cargo library)
  glue.rs                          the logic connected to Horizon's hook points
  *.rs, horizon-mixin.toml         one small helper per hook point
compatibility.json                 the Horizon and horizon-mixins commits it is built on, and the hook API it needs
tools/horizon.py                   applies the mixins to the pinned Horizon and checks, tests or builds the game
tools/package.py                   builds target/horizon_tech-<version>.hmod and .hnmod
tools/textures.py                  draws the block and item textures
```

## Develop

```sh
cargo test                         # the logic's tests, no Horizon needed
cargo clippy --all-targets -- -D warnings
python3 tools/package.py           # target/horizon_tech-0.2.0.hmod and .hnmod
python3 tools/textures.py          # redraw the textures (needs Pillow)
```

Compile it into the game (Linux; Horizon and horizon-mixins at the commits in `compatibility.json`, beside this repo):

```sh
git clone https://github.com/horizon-rust/horizon ../horizon
git clone https://github.com/horizon-rust/horizon-mixins ../horizon-mixins
git -C ../horizon-mixins checkout ddd9700b21997c95d95ac6ee5ecafe3830cf622a
cargo fetch --locked --manifest-path ../horizon-mixins/Cargo.toml   # stage builds its CLI offline
python3 tools/horizon.py stage     # export the pinned Horizon and apply the mixins
cargo fetch --locked --manifest-path target/mixins/modded/Cargo.toml
python3 tools/horizon.py check     # cargo check of the game with the mod
python3 tools/horizon.py test      # the logic's tests, inside the game
python3 tools/horizon.py build     # target/horizon-cargo/fast/horizon-client
```

Try it in the game: put `target/horizon_tech-0.2.0.hmod` in the game folder's `horizon-mods/` and the `.hnmod` in
`horizon-native-mods/`, then turn the mod on under Options > Mods. The launcher compiles the native part into the
game. The mod needs hook API 142, which is on Horizon's `main` since 2026-11-01 and comes with the next Nightly.
