use crate::list::List;
use crate::pokemon::Region;
use clap::{Parser, ValueEnum};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

/// What `--list` should print.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ListTarget {
    /// Every pokemon, in pokedex order.
    Pokemon,

    /// Every region that can be used in place of a pokemon.
    Regions,

    /// Every form that `--form` accepts.
    Forms,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The pokemon to display, use "random" to get a random pokemon, use a region to get a random pokemon from that region
    #[arg(add = ArgValueCandidates::new(pokemon_candidates))]
    pub pokemon: Vec<String>,

    /// List the available pokemon, regions, or forms, then exit
    #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "pokemon")]
    pub list: Option<ListTarget>,

    /// Whether to hide the pokemon's name which appears above it
    #[arg(long, default_value_t = false)]
    pub hide_name: bool,

    /// The form of the pokemon
    #[arg(short, long, default_value = "")]
    pub form: String,

    /// Display the pokemon as it's mega form
    #[arg(short, long, default_value_t = false)]
    pub mega: bool,

    /// Display the pokemon as it's mega X form
    #[arg(long, default_value_t = false)]
    pub mega_x: bool,

    /// Display the pokemon as it's mega Y form
    #[arg(long, default_value_t = false)]
    pub mega_y: bool,

    /// Display the pokemon as shiny
    #[arg(short, long, default_value_t = false)]
    pub shiny: bool,

    /// Display the alolan variant of the pokemon
    #[arg(short, long, default_value_t = false)]
    pub alolan: bool,

    /// Display the gigantamax variant of the pokemon
    #[arg(short, long, default_value_t = false)]
    pub gmax: bool,

    /// Display the hisui variant of the pokemon
    #[arg(long, default_value_t = false)]
    pub hisui: bool,

    /// Display the noble variant of the pokemon, this option often times only works in tandom with --hisui
    #[arg(short, long, default_value_t = false)]
    pub noble: bool,

    /// Display the galarian variant of the pokemon
    #[arg(long, default_value_t = false)]
    pub galar: bool,

    /// Display the female variant of the pokemon if it exists. This doesn't apply to nidoran, for some reason
    #[arg(long, default_value_t = false)]
    pub female: bool,
}

/// Every value the positional pokemon argument accepts.
fn pokemon_candidates() -> Vec<CompletionCandidate> {
    let list = List::read();

    let pokemon = list
        .filenames()
        .into_iter()
        .map(|filename| CompletionCandidate::new(filename.as_str()));

    let regions = Region::ALL
        .into_iter()
        .map(|region| CompletionCandidate::new(region.slug()));

    pokemon
        .chain(regions)
        .chain(std::iter::once(CompletionCandidate::new("random")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::pokemon_candidates;

    #[test]
    fn candidates_cover_pokemon_regions_and_random() {
        let candidates: Vec<String> = pokemon_candidates()
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect();

        assert!(candidates.contains(&"bulbasaur".to_owned()));
        assert!(candidates.contains(&"mr-mime".to_owned()));
        assert!(candidates.contains(&"hisui".to_owned()));
        assert!(candidates.contains(&"random".to_owned()));

        assert_eq!(candidates.len(), 905 + 9 + 1);
    }
}
