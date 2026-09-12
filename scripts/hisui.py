"""Generate data/hisui.txt from the Hisui pokedex.

Run by hand:

    python scripts/hisui.py

The output is committed, so neither the build nor CI touches the network.
"""

import csv
import json
import sys
import urllib.request

POKEDEX = 'https://pokeapi.co/api/v2/pokedex/hisui'

# PokeAPI species slugs that pokeget spells differently. Empty until the
# validation below reports one.
ALIASES = {}


def known_filenames():
    with open('data/names.csv', newline='') as file:
        return {row[1] for row in csv.reader(file)}


def main():
    # PokeAPI rejects Python's default urllib User-Agent with a 403, so a
    # custom one is set here. This does not change what gets fetched.
    request = urllib.request.Request(POKEDEX, headers={'User-Agent': 'pokeget-rs/hisui-script'})
    with urllib.request.urlopen(request) as response:
        pokedex = json.load(response)

    entries = sorted(pokedex['pokemon_entries'], key=lambda e: e['entry_number'])
    slugs = [ALIASES.get(e['pokemon_species']['name'], e['pokemon_species']['name'])
             for e in entries]

    known = known_filenames()
    missing = [slug for slug in slugs if slug not in known]
    if missing:
        print('not in names.csv: ' + ', '.join(missing), file=sys.stderr)
        print('add each one to ALIASES and rerun', file=sys.stderr)
        return 1

    with open('data/hisui.txt', 'w', newline='\n') as file:
        for slug in slugs:
            file.write(slug + '\n')

    print(f'wrote {len(slugs)} entries to data/hisui.txt')
    return 0


if __name__ == '__main__':
    sys.exit(main())
