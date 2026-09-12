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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
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

#[test]
fn list_needs_no_pokemon_argument() {
    let output = pokeget(&["--list"]);

    assert!(output.status.success());
    assert!(stdout(&output).lines().count() == 905);
}

#[test]
fn list_regions_names_every_region() {
    let output = pokeget(&["--list", "regions"]);

    assert!(output.status.success());

    let listed: Vec<String> = stdout(&output).lines().map(str::to_owned).collect();
    assert_eq!(listed.len(), 9);
    assert!(listed.contains(&"kanto".to_owned()));
    assert!(listed.contains(&"hisui".to_owned()));
}

#[test]
fn list_forms_includes_the_regional_ones() {
    let output = pokeget(&["--list", "forms"]);

    assert!(output.status.success());
    assert!(stdout(&output).lines().any(|line| line == "alola"));
}

#[test]
fn a_registration_script_is_printed_when_complete_is_set() {
    let output = Command::new(env!("CARGO_BIN_EXE_pokeget"))
        .env("COMPLETE", "bash")
        .output()
        .expect("failed to run pokeget");

    assert!(output.status.success());
    assert!(stdout(&output).contains("_clap_complete_pokeget"));
}

#[test]
fn list_forms_narrows_to_a_named_pokemon() {
    let output = pokeget(&["--list", "forms", "shaymin"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output).trim(), "sky");
}

#[test]
fn list_forms_accepts_a_dex_id() {
    // #386 is Deoxys, which has three alternate forms.
    let output = pokeget(&["--list", "forms", "386"]);

    assert!(output.status.success());

    let forms: Vec<String> = stdout(&output).lines().map(str::to_owned).collect();
    assert_eq!(forms, ["attack", "defense", "speed"]);
}

#[test]
fn a_pokemon_with_no_forms_prints_nothing_to_stdout() {
    let output = pokeget(&["--list", "forms", "bulbasaur"]);

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(stderr(&output).contains("no alternate forms"));
}

#[test]
fn list_forms_rejects_an_unknown_pokemon() {
    let output = pokeget(&["--list", "forms", "notapokemon"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("notapokemon"));
}

#[test]
fn a_pokemon_cannot_be_combined_with_the_other_list_targets() {
    for target in ["pokemon", "regions"] {
        let output = pokeget(&["--list", target, "pikachu"]);

        assert!(!output.status.success(), "--list {target} pikachu");
        assert!(stderr(&output).contains("only --list forms"));
    }
}

#[test]
fn list_forms_without_a_pokemon_still_lists_everything() {
    let output = pokeget(&["--list", "forms"]);

    assert!(output.status.success());
    assert!(stdout(&output).lines().count() > 200);
}
