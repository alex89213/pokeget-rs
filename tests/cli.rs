//! End to end tests that run the real binary.
//!
//! These go through a subprocess because `Pokemon::new` calls `exit(1)` on a
//! missing sprite, which cannot be caught in process.

use std::process::{Command, Output};

fn pokeget(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pokeget"))
        .args(args)
        .output()
        .expect("failed to run pokeget")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn no_arguments_is_an_error() {
    let output = pokeget(&[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("you must specify"));
}

#[test]
fn an_unknown_pokemon_is_an_error() {
    let output = pokeget(&["notapokemon"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("pokemon not found"));
}

#[test]
fn an_out_of_range_dex_id_reports_the_number_the_user_typed() {
    let output = pokeget(&["9999"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("9999 is not a valid pokedex ID"));
}

#[test]
fn hide_name_writes_nothing_to_stderr() {
    let output = pokeget(&["pikachu", "--hide-name"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(!output.stdout.is_empty());
}

#[test]
fn the_name_is_printed_to_stderr_by_default() {
    let output = pokeget(&["pikachu"]);

    assert!(output.status.success());
    assert!(stderr(&output).contains("Pikachu"));
}
