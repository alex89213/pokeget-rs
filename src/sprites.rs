use image::{DynamicImage, GenericImage};

use crate::pokemon::Pokemon;

/// Combines several pokemon sprites into one by stitching them horizontally.
pub fn combine(pokemons: &[Pokemon]) -> DynamicImage {
    let mut width: u32 = 0;
    let mut height: u32 = 0;

    for pokemon in pokemons {
        width += pokemon.sprite.width() + 1;
        if pokemon.sprite.height() > height {
            height = pokemon.sprite.height();
        }
    }

    let mut combined = DynamicImage::new_rgba8(width - 1, height);
    let mut shift = 0;

    for pokemon in pokemons {
        combined
            .copy_from(&pokemon.sprite, shift, height - pokemon.sprite.height())
            .unwrap();
        shift += pokemon.sprite.width() + 1;
    }

    combined
}

#[cfg(test)]
mod tests {
    use super::combine;
    use crate::pokemon::{Attributes, Pokemon};
    use image::DynamicImage;

    fn attributes() -> Attributes {
        Attributes {
            form: String::new(),
            female: false,
            shiny: false,
        }
    }

    fn pokemon(attributes: &Attributes, width: u32, height: u32) -> Pokemon<'_> {
        Pokemon {
            path: String::new(),
            name: String::new(),
            sprite: DynamicImage::new_rgba8(width, height),
            attributes,
        }
    }

    #[test]
    fn one_sprite_keeps_its_size() {
        let attributes = attributes();
        let combined = combine(&[pokemon(&attributes, 4, 6)]);

        assert_eq!((combined.width(), combined.height()), (4, 6));
    }

    #[test]
    fn sprites_are_stitched_with_one_pixel_between_them() {
        let attributes = attributes();
        let combined = combine(&[pokemon(&attributes, 4, 6), pokemon(&attributes, 8, 10)]);

        // 4 + 1 + 8 wide, and as tall as the tallest sprite.
        assert_eq!((combined.width(), combined.height()), (13, 10));
    }

    #[test]
    fn three_sprites_are_stitched_in_order() {
        let attributes = attributes();
        let combined = combine(&[
            pokemon(&attributes, 2, 2),
            pokemon(&attributes, 3, 3),
            pokemon(&attributes, 4, 4),
        ]);

        // 2 + 1 + 3 + 1 + 4 wide.
        assert_eq!((combined.width(), combined.height()), (11, 4));
    }
}
