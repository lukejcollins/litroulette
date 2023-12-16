# LitRoulette

LitRoulette is a CLI tool that suggests random books for you to read. Whether you're looking for your next sci-fi adventure, a romantic escapade, or a deep dive into historical events, LitRoulette can help you decide.

## Features

- Get book recommendations based on genre.
- View a list of popular genres to explore.
- Each recommendation provides the book title, author(s), and a short description.

## Building from source

Before building LitRoulette from source, make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
# Clone the repository
git clone https://github.com/yourusername/litroulette.git

# Navigate to the repository directory
cd litroulette

# Build the project
cargo build --release

# Run LitRoulette
./target/release/litroulette
```

## Usage

To get a random book suggestion:

```bash
litroulette --genre <genre-name>
```

To view the list of available genres:

```bash
litroulette --genrelist
```

## Contributing

Contributions to LitRoulette are welcome!

## License

LitRoulette is released under the MIT License.

## Acknowledgments

- Google Books API for providing the book data.
- Rust community for the amazing crates used in this project.

## 
**Happy Reading!**
