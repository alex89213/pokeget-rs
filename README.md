# pokeget-rs

Display pokemon sprites in your terminal.

```sh
pokeget pikachu
```

Sprites are embedded in the binary at compile time, so pokeget runs instantly
and works offline. It draws at half-block resolution, which fits a sprite into
half the terminal rows a naive renderer would need.

## This fork

This is a fork of [talwat/pokeget-rs](https://github.com/talwat/pokeget-rs).
It carries the following on top of upstream.

### Fixes

- Nidoran was swapped in the name list, so `pokeget 29` showed the male sprite
  and `pokeget 32` the female one. This is upstream issue #37.
- Region ranges were written as pokedex numbers but used to index a 0-based
  map, so every region was shifted up by one. `pokeget kanto` could return
  Chikorita, and `pokeget galar` panicked on roughly 1 run in 96 by indexing
  past the end of the list.
- The galar range ran to #905, seven species past the end of the galar dex at
  #898 Calyrex. Those seven are Legends: Arceus species and belong to hisui.
- `scripts/list.py` wrote to a file the crate never read. Pointing it at
  `names.csv` revealed that regenerating the list degraded two display names,
  which is fixed as well.

### Features

- `hisui` as a region, covering the 242 species of the Legends: Arceus dex.
- Region picks can return that region's alternate forms, so `pokeget alola`
  may give you an Alolan Raichu.
- Form suffixes appear in the displayed name, so `raichu-alola` prints as
  `Raichu (Alola)`.
- `--list` for discovering the available pokemon, regions, and forms, including
  the forms of one specific pokemon.
- Shell completion for pokemon names and regions. This is upstream issue #21.

### Project

- A test suite, which the crate did not have before.
- CI running rustfmt, clippy under `-D warnings`, and the tests on every push.

## Installation

This fork is not published to crates.io, so `cargo install pokeget` gets
upstream's version rather than this one. Build from source:

```sh
git clone --recurse-submodules https://github.com/alex89213/pokeget-rs.git
cd pokeget-rs
cargo install --path .
```

`--recurse-submodules` is not optional. The sprites live in the
`data/pokesprite` submodule and are embedded at compile time, so the build
fails without it. If you forget, run `git submodule update --init --recursive`.

`cargo install --path .` puts the binary in `~/.cargo/bin`. To build without
installing, use `cargo build --release` and take the binary from
`target/release/pokeget`.

To update, `git pull`, then `git submodule update` if the sprite submodule
moved, then reinstall.

## Usage

```
pokeget [OPTIONS] [POKEMON]...
```

### Picking a pokemon

By name, by pokedex ID, or several at once:

```sh
pokeget pikachu
pokeget 25
pokeget bulbasaur pikachu mew
```

`random` gives you any pokemon, and `0` does the same:

```sh
pokeget random
```

A region name gives you a random pokemon from that region's dex. The nine
regions are kanto, johto, hoenn, sinnoh, unova, kalos, alola, galar, and hisui:

```sh
pokeget hisui
```

Alola, galar, and hisui pools include that region's alternate forms, so
`pokeget alola` can return an Alolan Raichu alongside the Alola-native species.

### Alternate forms

`--form` takes a form suffix, and the common ones have their own flags:

```sh
pokeget shaymin --form sky
pokeget charizard --mega-x
pokeget raichu sandslash meowth --alolan
pokeget arcanine --hisui --noble
```

Forms are ignored for `random` and region picks, since the pool already decides
which sprite you get.

### Discovering what is available

```sh
pokeget --list             # every pokemon, in pokedex order
pokeget --list regions     # the nine regions
pokeget --list forms       # every form suffix in the sprite set
```

Naming a pokemon alongside `--list forms` narrows the output to that pokemon,
which is usually what you want:

```sh
pokeget --list forms shaymin    # sky
pokeget --list forms kyogre     # primal
pokeget --list forms deoxys     # attack, defense, speed
pokeget --list forms arceus     # all 18 type forms
```

A pokedex ID works in place of a name, so `pokeget --list forms 386` is Deoxys.
Feed the result straight back in with `--form`.

## Options

The pokemon's name is printed above the sprite on stderr, and the sprite goes to
stdout. `--hide-name` suppresses the name, which is what you want when piping or
redirecting.

| Option | Short | Description |
| --- | --- | --- |
| `[POKEMON]...` | | One or more names, pokedex IDs, `random`, or a region |
| `--list [WHAT]` | | List `pokemon`, `regions`, or `forms`, then exit. Defaults to `pokemon`. With `forms`, an optional pokemon narrows the output |
| `--hide-name` | | Do not print the pokemon's name above the sprite |
| `--form <FORM>` | `-f` | Show a named form, such as `sky` or `primal` |
| `--mega` | `-m` | Mega form |
| `--mega-x` | | Mega X form |
| `--mega-y` | | Mega Y form |
| `--alolan` | `-a` | Alolan form |
| `--galar` | | Galarian form |
| `--hisui` | | Hisuian form |
| `--noble` | `-n` | Noble form, usually alongside `--hisui` |
| `--gmax` | `-g` | Gigantamax form |
| `--shiny` | `-s` | Shiny colors |
| `--female` | | Female variant, where one exists. Does not apply to Nidoran, which is two separate species |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

If you pass more than one form flag, exactly one wins, and they are checked in
this order: `--mega`, `--mega-x`, `--mega-y`, `--alolan`, `--gmax`, `--hisui`,
`--galar`, and finally `--form`. So `--gmax --hisui` gives you the gigantamax
sprite, and `--form sky` is only consulted when no form flag is set. `--noble`
is the exception: it appends to whichever form was chosen, which is why
`--hisui --noble` works.

## Environment variables

### `POKEGET_SHINY_RATE`

Every pokemon has a random chance of coming out shiny, defaulting to 1 in 8192
to match the games. Set this to change the odds:

```sh
POKEGET_SHINY_RATE=100 pokeget random   # 1 in 100
POKEGET_SHINY_RATE=1 pokeget pikachu    # always shiny
```

A value of `0` is treated as `1`. A value that is not a number prints a warning
to stderr and falls back to the default. `--shiny` overrides the roll entirely.

## Shell completion

pokeget completes pokemon names, regions, and flags. Add the line for your shell
to its startup file. This requires `pokeget` to be on your `PATH`, since the
shell runs the binary to ask it for candidates.

Bash, in `.bashrc`:

```sh
source <(COMPLETE=bash pokeget)
```

Zsh, in `.zshrc`:

```sh
source <(COMPLETE=zsh pokeget)
```

Fish, in `~/.config/fish/config.fish`:

```fish
COMPLETE=fish pokeget | source
```

Elvish, in `~/.config/elvish/rc.elv`:

```elvish
eval (E:COMPLETE=elvish pokeget | slurp)
```

PowerShell, in `$PROFILE`:

```powershell
$env:COMPLETE = "powershell"
pokeget | Out-String | Invoke-Expression
Remove-Item Env:\COMPLETE
```

## Running on shell startup

pokeget is fast enough to run from `.bashrc` directly. If you want shell
initialization to stay untouched regardless, write the output to a file once and
print that instead:

```sh
pokeget pikachu --hide-name > ~/.cache/pokeget.txt
```

Then put `cat ~/.cache/pokeget.txt` in your shell config. This trades away
`random`, since the sprite is fixed until you regenerate the file.

## Development

```sh
cargo test                                  # about 2 seconds
cargo clippy --all-targets -- -D warnings   # what CI enforces
cargo fmt --all --check
```

`scripts/list.py` regenerates `data/names.csv` from `data/pokemon.txt`, and
`scripts/hisui.py` regenerates `data/hisui.txt` from PokeAPI. Both are run by
hand, and their output is committed, so neither the build nor CI touches the
network.

## Credits

Sprites come from [pokesprite](https://github.com/msikma/pokesprite), pulled in
as a git submodule.

For a comparison against similar projects, see
[OTHER_PROJECTS.md](OTHER_PROJECTS.md).

## License

MIT. See [LICENSE](LICENSE).
