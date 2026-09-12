use std::collections::BTreeSet;
use std::io::Cursor;
use std::ops::RangeInclusive;

use crate::pokemon::Region;
use crate::Data;
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

    /// The filenames in the Hisui pokedex, which is not a contiguous range.
    hisui: Vec<String>,
}

impl List {
    /// Reads a new [`List`] from `data/names.csv`.
    pub fn read() -> Self {
        const FILE: &str = include_str!("../data/names.csv");
        const HISUI: &str = include_str!("../data/hisui.txt");

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

        let hisui = HISUI.lines().map(str::to_owned).collect();

        Self { ids, names, hisui }
    }

    /// Takes a filename and looks up the proper display name.
    ///
    /// Filenames carrying a form suffix are formatted with the form in
    /// parentheses, so `raichu-alola` becomes `Raichu (Alola)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pokeget::list::List;
    /// let list = List::read();
    /// assert_eq!(list.format_name("mr-mime"), "Mr. Mime")
    /// ```
    pub fn format_name(&self, filename: &str) -> String {
        if let Some(name) = self.lookup(filename) {
            return name;
        }

        let Some((base, form)) = self.split_form(filename) else {
            return filename.to_owned();
        };

        let Some(name) = self.lookup(base) else {
            return filename.to_owned();
        };

        format!("{name} ({})", title_case(form))
    }

    /// Looks up the display name for an exact filename.
    fn lookup(&self, filename: &str) -> Option<String> {
        let id = self.ids.get_by_right(filename)?;

        self.names.get(*id).cloned()
    }

    /// Splits a filename into its base name and its form.
    ///
    /// Hyphens are tried from the right, so the longest known base name wins
    /// and `mr-mime-galar` splits into `mr-mime` and `galar` rather than `mr`
    /// and `mime-galar`. Returns `None` when no prefix is a known pokemon,
    /// which is what keeps `ho-oh` in one piece.
    fn split_form<'a>(&self, filename: &'a str) -> Option<(&'a str, &'a str)> {
        filename.rmatch_indices('-').find_map(|(i, _)| {
            let (base, form) = (&filename[..i], &filename[i + 1..]);

            self.ids.get_by_right(base).map(|_| (base, form))
        })
    }

    /// Gets a pokemon filename by a Dex ID.
    pub fn get_by_id(&self, id: usize) -> Option<&String> {
        self.ids.get_by_left(&id)
    }

    /// The filenames in the Hisui pokedex, in dex order.
    pub fn hisui(&self) -> &[String] {
        &self.hisui
    }

    /// Every display name, in pokedex order.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Every filename, in pokedex order.
    pub fn filenames(&self) -> Vec<&String> {
        (0..self.names.len())
            .filter_map(|id| self.ids.get_by_left(&id))
            .collect()
    }

    /// Every form suffix present in the embedded sprites, sorted.
    pub fn forms(&self) -> Vec<String> {
        Data::iter()
            .filter_map(|path| {
                let file = path.strip_prefix("regular/")?.strip_suffix(".png")?;

                // Skip the female/ subdirectory, and skip filenames that are
                // themselves a pokemon, so porygon-z does not become a form
                // called `z`.
                if file.contains('/') || self.ids.get_by_right(file).is_some() {
                    return None;
                }

                let (_, form) = self.split_form(file)?;

                Some(form.to_owned())
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Gets a random pokemon & returns it's filename.
    pub fn random(&self) -> String {
        let idx = rand::random_range(0..self.ids.len());
        self.ids.get_by_left(&idx).unwrap().clone()
    }

    /// The index range covering a region's own species.
    ///
    /// Returns `None` for Hisui, whose dex is not contiguous and lives in
    /// `data/hisui.txt` instead.
    fn index_range(region: Region) -> Option<RangeInclusive<usize>> {
        // Index ranges, not pokedex numbers: index 0 is #1.
        Some(match region {
            Region::Kanto => 0..=150,
            Region::Johto => 151..=250,
            Region::Hoenn => 251..=385,
            Region::Sinnoh => 386..=492,
            Region::Unova => 493..=648,
            Region::Kalos => 649..=720,
            Region::Alola => 721..=808,
            Region::Galar => 809..=904,
            Region::Hisui => return None,
        })
    }

    /// Every sprite filename a region can produce.
    ///
    /// This is the region's species with each one replaced by its regional
    /// variant where a sprite exists, plus every sprite carrying the region's
    /// suffix. Those two sets overlap for Hisui and are disjoint for Alola and
    /// Galar, whose regional forms belong to species from earlier regions.
    pub fn region_pool(&self, region: Region) -> Vec<String> {
        let base: Vec<String> = match Self::index_range(region) {
            Some(range) => range
                .filter_map(|id| self.ids.get_by_left(&id).cloned())
                .collect(),
            None => self.hisui.clone(),
        };

        let Some(suffix) = region.suffix() else {
            return base;
        };

        let mut pool: BTreeSet<String> = base
            .into_iter()
            .map(|name| {
                let variant = format!("{name}-{suffix}");

                if sprite_exists(&variant) {
                    variant
                } else {
                    name
                }
            })
            .collect();

        pool.extend(sprites_with_suffix(suffix));

        pool.into_iter().collect()
    }

    /// Gets a random pokemon from a region and returns its filename.
    pub fn get_by_region(&self, region: Region) -> String {
        let pool = self.region_pool(region);

        pool[rand::random_range(0..pool.len())].clone()
    }
}

/// Whether a sprite with this filename exists.
fn sprite_exists(filename: &str) -> bool {
    Data::get(&format!("regular/{filename}.png")).is_some()
}

/// Capitalizes each hyphen separated word, so `hisui-noble` becomes
/// `Hisui Noble`.
fn title_case(form: &str) -> String {
    form.split('-')
        .map(|word| {
            let mut chars = word.chars();

            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every sprite filename ending in a form suffix, such as `raichu-alola`.
fn sprites_with_suffix(suffix: &str) -> Vec<String> {
    let tail = format!("-{suffix}");

    Data::iter()
        .filter_map(|path| {
            let file = path.strip_prefix("regular/")?.strip_suffix(".png")?;

            // `file.contains('/')` skips the female/ subdirectory, whose
            // sprites are reached through --female instead.
            if file.contains('/') || !file.ends_with(&tail) {
                return None;
            }

            Some(file.to_owned())
        })
        .collect()
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
    fn the_hisui_dex_resolves_to_sprites() {
        let list = List::read();

        assert_eq!(list.hisui().len(), 242);

        for filename in list.hisui() {
            let path = format!("regular/{filename}.png");
            assert!(Data::get(&path).is_some(), "missing sprite: {path}");
        }
    }

    #[test]
    fn hisui_picks_come_from_the_hisui_dex() {
        let list = List::read();

        for _ in 0..500 {
            let filename = list.get_by_region(Region::Hisui);

            // Task 7 substitutes 16 species for their hisui-suffixed variant,
            // so a pick may not appear literally in `list.hisui()`. Strip the
            // suffix back off before checking dex membership.
            let base = filename.strip_suffix("-hisui").unwrap_or(&filename);

            assert!(
                list.hisui().contains(&base.to_owned()),
                "{filename} is not in the hisui dex"
            );
        }
    }

    #[test]
    fn region_pools_include_that_regions_forms() {
        let list = List::read();

        let alola = list.region_pool(Region::Alola);
        assert!(alola.contains(&"raichu-alola".to_owned()));
        assert_eq!(alola.len(), 88 + 18);

        let galar = list.region_pool(Region::Galar);
        assert!(galar.contains(&"mr-mime-galar".to_owned()));
        assert_eq!(galar.len(), 96 + 19);

        // Every hisui form belongs to a species already in the hisui dex, so
        // the pool stays at 242 with 16 entries swapped for their variant.
        let hisui = list.region_pool(Region::Hisui);
        assert!(hisui.contains(&"zorua-hisui".to_owned()));
        assert!(!hisui.contains(&"zorua".to_owned()));
        assert_eq!(hisui.len(), 242);

        // Nobles are reachable through --noble, not through a region.
        assert!(!hisui.contains(&"arcanine-hisui-noble".to_owned()));
        assert!(!hisui.contains(&"kleavor-noble".to_owned()));

        // Regions with no forms are untouched.
        assert_eq!(list.region_pool(Region::Kanto).len(), 151);
    }

    #[test]
    fn every_region_pool_resolves_to_sprites() {
        let list = List::read();

        for region in Region::ALL {
            let pool = list.region_pool(region);
            assert!(!pool.is_empty(), "{region:?} has an empty pool");

            for filename in pool {
                let path = format!("regular/{filename}.png");
                assert!(Data::get(&path).is_some(), "missing sprite: {path}");
            }
        }
    }

    #[test]
    fn region_picks_stay_inside_their_region() {
        let list = List::read();

        // 0-based index ranges, so Kanto is #1 to #151 at indices 0 to 150.
        //
        // Alola and Galar are covered by `region_pools_include_that_regions_forms`
        // instead: since Task 7, their pools include form filenames such as
        // `raichu-alola` that are not keys in `ids`, so this id-range check no
        // longer applies to them. Hisui was never covered here; it has its own
        // `hisui_picks_come_from_the_hisui_dex` test.
        let regions = [
            (Region::Kanto, 0..=150),
            (Region::Johto, 151..=250),
            (Region::Hoenn, 251..=385),
            (Region::Sinnoh, 386..=492),
            (Region::Unova, 493..=648),
            (Region::Kalos, 649..=720),
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

    #[test]
    fn formats_names_that_carry_a_form() {
        let list = List::read();

        assert_eq!(list.format_name("raichu-alola"), "Raichu (Alola)");
        assert_eq!(list.format_name("charizard-mega-x"), "Charizard (Mega X)");
        assert_eq!(
            list.format_name("arcanine-hisui-noble"),
            "Arcanine (Hisui Noble)"
        );

        // The longest base name wins, so this is not "Mr (Mime Galar)".
        assert_eq!(list.format_name("mr-mime-galar"), "Mr. Mime (Galar)");
    }

    #[test]
    fn hyphenated_names_are_not_mistaken_for_forms() {
        let list = List::read();

        assert_eq!(list.format_name("porygon-z"), "Porygon-Z");
        assert_eq!(list.format_name("ho-oh"), "Ho-Oh");
        assert_eq!(list.format_name("jangmo-o"), "Jangmo-o");
        assert_eq!(list.format_name("nidoran-f"), "Nidoran-F");
    }

    #[test]
    fn lists_every_pokemon_in_dex_order() {
        let list = List::read();

        assert_eq!(list.names().len(), 905);
        assert_eq!(list.names()[0], "Bulbasaur");
        assert_eq!(list.names()[904], "Enamorus");

        assert_eq!(list.filenames().len(), 905);
        assert_eq!(list.filenames()[0], "bulbasaur");
    }

    #[test]
    fn lists_real_forms_and_not_fragments_of_names() {
        let list = List::read();
        let forms = list.forms();

        for expected in ["alola", "galar", "hisui", "mega", "mega-x", "gmax"] {
            assert!(
                forms.contains(&expected.to_owned()),
                "missing form: {expected}"
            );
        }

        // Only `oh` and `mime` work as sentinels for mis-splitting: `ho-oh` and
        // `mr-mime` are themselves name table entries, so nothing else can
        // produce those strings. `f`, `z` and `o` cannot be used the same way,
        // because Unown ships a sprite per letter and those are real forms.
        for fragment in ["oh", "mime"] {
            assert!(
                !forms.contains(&fragment.to_owned()),
                "bogus form: {fragment}"
            );
        }

        // The Unown letters are real forms, and their presence also shows the
        // name table guard does not over-exclude.
        for letter in ["b", "f", "o", "z"] {
            assert!(
                forms.contains(&letter.to_owned()),
                "missing unown form: {letter}"
            );
        }
    }
}
