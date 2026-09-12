//! Display pokemon sprites in your terminal.

use clap::{CommandFactory, Parser};
use pokeget::cli::{Args, ListTarget};
use pokeget::list::List;
use pokeget::pokemon::{Attributes, Pokemon, Region};
use pokeget::sprites;
use std::process::exit;

fn main() {
    // Returns early when the shell is asking for completions.
    clap_complete::CompleteEnv::with_factory(Args::command).complete();

    let list = List::read();
    let args = Args::parse();

    if let Some(target) = args.list {
        let lines = listing(&list, target, &args.pokemon).unwrap_or_else(|message| {
            eprintln!("{message}");
            exit(1)
        });

        for line in lines {
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
///
/// `pokemon` is the positional argument. Only `--list forms` accepts one, and
/// it narrows the output to that pokemon's forms.
fn listing(list: &List, target: ListTarget, pokemon: &[String]) -> Result<Vec<String>, String> {
    if target != ListTarget::Forms && !pokemon.is_empty() {
        return Err("only --list forms takes a pokemon".to_owned());
    }

    match target {
        ListTarget::Pokemon => Ok(list.names().to_vec()),
        ListTarget::Regions => Ok(Region::ALL
            .into_iter()
            .map(|region| region.slug().to_owned())
            .collect()),
        ListTarget::Forms => match pokemon {
            [] => Ok(list.forms()),
            [name] => forms_of(list, name),
            _ => Err("--list forms takes at most one pokemon".to_owned()),
        },
    }
}

/// The forms of one pokemon, named by filename or by pokedex ID.
///
/// A pokemon with no forms is not an error: it prints nothing to stdout, and
/// the explanation goes to stderr so that redirecting stdout yields an empty
/// file rather than prose.
fn forms_of(list: &List, arg: &str) -> Result<Vec<String>, String> {
    let filename = resolve(list, arg)?;
    let forms = list.forms_for(&filename);

    if forms.is_empty() {
        eprintln!("{} has no alternate forms", list.format_name(&filename));
    }

    Ok(forms)
}

/// Resolves a pokedex ID or a name into a sprite filename.
fn resolve(list: &List, arg: &str) -> Result<String, String> {
    if let Ok(dex_id) = arg.parse::<usize>() {
        return list
            .get_by_id(dex_id.wrapping_sub(1))
            .cloned()
            .ok_or_else(|| format!("{arg} is not a valid pokedex ID"));
    }

    let filename = arg.to_lowercase();

    if list.is_pokemon(&filename) {
        Ok(filename)
    } else {
        Err(format!("{arg} is not a pokemon"))
    }
}
