# pokeget-rs

A better rust version of pokeget.

## Usage

`pokeget <pokemon>`

For more info, run `pokeget --help`.

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
- `--list` for discovering the available pokemon, regions, and forms.
- Shell completion for pokemon names and regions. This is upstream issue #21.

### Project

- A test suite, which the crate did not have before.
- CI running rustfmt, clippy under `-D warnings`, and the tests on every push.

This fork is not published to crates.io, so `cargo install pokeget` gets
upstream's version rather than this one. Build from source to get these
changes.

## Project status

I've decided that while I will keep fixing bugs and so on,
no more sprites will be added or modified unless it is a serious
issue. This is because firstly, pokemon is moving away from pixel
sprites, and secondly, that pokesprite has ceased updates and a
suitable alternative hasn't been found.

### .bashrc

If you're using pokeget on shell startup, such as in `.bashrc`,
then instead of running `pokeget <pokemon>`, you can write the output
to a file by doing: `pokeget <pokemon> > file.txt`
and then have something like `cat file.txt` in your bashrc.

This makes your shell initialization practically instant, but obviously
won't work with random pokemon. pokeget is already fairly fast,
so using it on shell initialization is also not a very large bottleneck.

### Shell completion

pokeget completes pokemon names, regions, and flags. Add the line for your
shell to its startup file.

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

### Examples

#### Using multiple pokemon

`pokeget bulbasaur pikachu random`

#### Using pokedex ID's

`pokeget 1 2 3`

#### Using alternative forms

`pokeget raichu sandslash meowth --alolan`

#### Using regions

`pokeget kanto`

This picks a random pokemon from that region's dex, including kanto, johto,
hoenn, sinnoh, unova, kalos, alola, galar, and hisui. Some picks come back as
a regional form, such as `raichu-alola`, since those forms are part of the
region's pool.

#### Listing what's available

`pokeget --list` prints every pokemon name. Pass `pokemon`, `regions`, or
`forms` to list just one of those, for example `pokeget --list regions`.

## Installation

### Cargo *(recommended)*

The recommended installation method is to use cargo:

```sh
cargo install pokeget
```

and making sure `$HOME/.cargo/bin` is added to `$PATH`.

### AUR

If you're on Arch, you can also use the AUR:

```sh
yay -S pokeget
```

### Git

You can also clone the repository and compile manually by doing:

```sh
git clone --recurse-submodules https://github.com/alex89213/pokeget-rs.git
cd pokeget-rs
cargo build --release
mv target/release/pokeget ~/.local/bin
```

and making sure `$HOME/.local/bin` is added to `$PATH`.

### Adding a directory to $PATH

#### Bash & Zsh

Append this to your `.bashrc` or `.zshrc`:

```sh
export PATH="<path>:$PATH"
```

#### Fish

Run this in your CLI:

```sh
fish_add_path <path>
```

## Updating

Run `git pull` on the repository and recompile. Remember `git submodule update`
if the sprite submodule has moved, since the sprites are embedded at compile
time.

## Why?

Because the first pokeget was slow, bloated, and super complicated, so I decided to make a better version in rust.

Now, instead of precomputing all the sprites and uploading them to a repo, pokeget will
be able to compute them on-demand which makes everything much more flexible.
Rust enables that computation to be done much more quickly than something like python.

It will also draw the sprites 2x smaller by using half squares.

## What about other projects?

pokeget-rs has an edge over projects like the old pokeget, pokeshell, etc... since it's in rust.
It also is significantly (5.5x) faster than krabby which is another very similar project.

For more info, go to [OTHER_PROJECTS.md](OTHER_PROJECTS.md).

## What about big sprites?

Gone. Reduced to atoms.

In all seriousness, I've just decided to not deal with them since it's significantly
extra work that I don't want to deal with. They were rarely used, and looked ugly
in small terminal windows, so there was little use in keeping them.

## Credits

This time, the sprites are from [pokesprite](https://github.com/msikma/pokesprite) and pokeget uses them with a git submodule.

Sprites are embedded into the binary, so pokeget won't download them. This is a good compromise,
since while the binary may be large, pokeget can execute almost instantly and while offline.

## License

pokeget uses the MIT license, so feel free to fork it and customize it as you please.
If you're unsure about any of the internal workings of pokeget, [open an issue](https://github.com/talwat/pokeget-rs/issues),
and I'll answer whatever question you might have.
