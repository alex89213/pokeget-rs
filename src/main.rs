//! Display pokemon sprites in your terminal.

use clap::Parser;
use pokeget::cli::{Args, ListTarget};
use pokeget::list::List;
use pokeget::pokemon::{Attributes, Pokemon, Region};
use pokeget::sprites;
use std::process::exit;

fn main() {
    let list = List::read();
    let args = Args::parse();

    if let Some(target) = args.list {
        for line in listing(&list, target) {
            println!("{line}");
        }

        return;
    }

    if args.pokemon.is_empty() {
        eprintln!("you must specify the pokemon you want to display");
        exit(1);
    }

    let attributes = Attributes::new(&args);
    let pokemons: Vec<Pokemon> = args
        .pokemon
        .into_iter()
        .map(|x| Pokemon::new(x, &list, &attributes))
        .collect();

    let combined = sprites::combine(&pokemons);
    if !args.hide_name {
        let names: Vec<&str> = pokemons.iter().map(|x| x.name.as_ref()).collect();
        eprintln!("{}", names.join(", "));
    }

    println!("{}", showie::render(&combined));
}

/// Renders the output of `--list`.
fn listing(list: &List, target: ListTarget) -> Vec<String> {
    match target {
        ListTarget::Pokemon => list.names().to_vec(),
        ListTarget::Regions => Region::ALL
            .into_iter()
            .map(|region| region.slug().to_owned())
            .collect(),
        ListTarget::Forms => list.forms(),
    }
}
