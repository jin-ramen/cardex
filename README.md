# cardex
![Example](assets/example.png)
A command line Pokemon TCG card searcher written in Rust, backed by the [TCGdex API](https://tcgdex.dev).

## Install
```sh
git clone https://github.com/jin-ramen/cardex.git
cd cardex
cargo install --path .
```

## Usage
```
cardex <COMMAND> [OPTIONS]

Commands:
    search      Seach cards by name
    card        Show a single card by id
```

### Examples

```sh
# find every Ceruledge
cardex search ceruledge

# full detail for one card
cardex card sv08.5-147

# search for set/s by name
cardex sets -n prismatic

# full detail for a set
cardex set sv08.5
```