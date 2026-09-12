use std::io::Cursor;

use crate::pokemon::Region;
use bimap::BiHashMap;

/// A parsed representation of `names.csv`.
///
/// Used to derive filenames from Pokedex ID's, and to
/// format image filenames back into proper pokemon names.
pub struct List {
    /// The Pokedex IDs and their corresponding filenames.
    ids: BiHashMap<usize, String>,

    /// All the proper, formatted names in order of Pokedex ID.
    names: Vec<String>,
}

impl List {
    /// Reads a new [`List`] from `data/names.csv`.
    pub fn read() -> Self {
        const FILE: &str = include_str!("../data/names.csv");

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Cursor::new(FILE));

        const CAPACITY: usize = 1000;

        let mut ids = BiHashMap::with_capacity(CAPACITY);
        let mut names = Vec::with_capacity(CAPACITY);

        for (i, entry) in reader.deserialize().enumerate() {
            let record: (String, String) = entry.unwrap();

            ids.insert(i, record.1);
            names.push(record.0);
        }

        Self { ids, names }
    }

    /// Takes a filename and looks up the proper display name.
    ///
    /// # Examples
    ///
    /// ```
    /// use pokeget::list::List;
    /// let list = List::read();
    /// assert_eq!(list.format_name("mr-mime"), "Mr. Mime")
    /// ```
    pub fn format_name(&self, filename: &str) -> String {
        let Some(id) = self.ids.get_by_right(filename) else {
            return filename.to_owned();
        };

        let Some(name) = self.names.get(*id) else {
            return filename.to_owned();
        };

        name.clone()
    }

    /// Gets a pokemon filename by a Dex ID.
    pub fn get_by_id(&self, id: usize) -> Option<&String> {
        self.ids.get_by_left(&id)
    }

    /// Gets a random pokemon & returns it's filename.
    pub fn random(&self) -> String {
        let idx = rand::random_range(0..self.ids.len());
        self.ids.get_by_left(&idx).unwrap().clone()
    }

    /// Gets a random pokemon from a region and returns its filename.
    pub fn get_by_region(&self, region: Region) -> String {
        // Index ranges, not pokedex numbers: index 0 is #1.
        let range = match region {
            Region::Kanto => 0..=150,
            Region::Johto => 151..=250,
            Region::Hoenn => 251..=385,
            Region::Sinnoh => 386..=492,
            Region::Unova => 493..=648,
            Region::Kalos => 649..=720,
            Region::Alola => 721..=808,
            Region::Galar => 809..=904,
        };

        let idx = rand::random_range(range);

        self.ids
            .get_by_left(&idx)
            .expect("region index is inside the list")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::List;
    use crate::pokemon::Region;
    use crate::Data;

    #[test]
    fn nidoran_ids_match_the_pokedex() {
        let list = List::read();

        // #29 is Nidoran female and #32 is Nidoran male. `ids` is 0-based,
        // so those are indices 28 and 31.
        assert_eq!(list.get_by_id(28).map(String::as_str), Some("nidoran-f"));
        assert_eq!(list.get_by_id(31).map(String::as_str), Some("nidoran-m"));

        assert_eq!(list.format_name("nidoran-f"), "Nidoran-F");
        assert_eq!(list.format_name("nidoran-m"), "Nidoran-M");
    }

    #[test]
    fn formats_names_that_lost_punctuation_in_their_filename() {
        let list = List::read();

        assert_eq!(list.format_name("mr-mime"), "Mr. Mime");
        assert_eq!(list.format_name("farfetchd"), "Farfetch'd");
        assert_eq!(list.format_name("ho-oh"), "Ho-Oh");
        assert_eq!(list.format_name("type-null"), "Type: Null");
    }

    #[test]
    fn unknown_filenames_are_returned_unchanged() {
        let list = List::read();

        assert_eq!(list.format_name("notapokemon"), "notapokemon");
    }

    #[test]
    fn every_listed_pokemon_has_a_sprite() {
        let list = List::read();

        assert_eq!(list.names.len(), 905);

        for id in 0..list.names.len() {
            let filename = list.get_by_id(id).expect("every id has a filename");
            let path = format!("regular/{filename}.png");

            assert!(Data::get(&path).is_some(), "missing sprite: {path}");
        }
    }

    #[test]
    fn region_picks_stay_inside_their_region() {
        let list = List::read();

        // 0-based index ranges, so Kanto is #1 to #151 at indices 0 to 150.
        let regions = [
            (Region::Kanto, 0..=150),
            (Region::Johto, 151..=250),
            (Region::Hoenn, 251..=385),
            (Region::Sinnoh, 386..=492),
            (Region::Unova, 493..=648),
            (Region::Kalos, 649..=720),
            (Region::Alola, 721..=808),
            (Region::Galar, 809..=904),
        ];

        for (region, range) in regions {
            for _ in 0..500 {
                let filename = list.get_by_region(region);
                let id = *list
                    .ids
                    .get_by_right(&filename)
                    .unwrap_or_else(|| panic!("{filename} is not a known pokemon"));

                assert!(
                    range.contains(&id),
                    "{region:?} produced {filename} (#{})",
                    id + 1
                );
            }
        }
    }
}
