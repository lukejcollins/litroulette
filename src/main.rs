use clap::{Arg, Command};
use colored::*;
use reqwest;
use serde::Deserialize;
use rand::{thread_rng, Rng};

// Define the structure of the API response according to the JSON format returned by the Google Books API.
#[derive(Deserialize, Debug)]
struct ApiResponse {
    items: Vec<Book>,
}

// Define the structure of a Book within the API response.
#[derive(Deserialize, Debug)]
struct Book {
    #[serde(rename = "volumeInfo")]
    volume_info: VolumeInfo,
}

// Define the details we want to extract about the Volume, which includes title, authors, and description.
#[derive(Deserialize, Debug)]
struct VolumeInfo {
    title: String,
    authors: Option<Vec<String>>,
    description: Option<String>,
}

// The main async function to run our application.
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Setting up command line arguments using clap.
    let matches = Command::new("LitRoulette")
        .version("0.1.0")
        .author("Your Name")
        .about("Provides random book suggestions")
        .arg(Arg::new("genre")
             .short('g')
             .long("genre")
             .value_name("GENRE")
             .help("Filters books by a specified genre")
             .takes_value(true))
        .arg(Arg::new("genrelist")
             .long("genrelist")
             .help("Displays a list of available genres"))
        .get_matches();

    // Check and display the genre list if the 'genrelist' argument is present.
    if matches.is_present("genrelist") {
        println!("\n{}", "Available genres include: Fiction, SciFi, Mystery, Romance, Fantasy, History, Horror, and more!".magenta());
        // Do not return here, so that if '--genre' is also provided, it can continue to execute.
    }

    // Check if 'genre' argument is provided, and execute the book suggestion logic.
    if let Some(genre) = matches.value_of("genre") {
        let url = format!("https://www.googleapis.com/books/v1/volumes?q=subject:{}", genre);

        let resp = reqwest::get(&url).await?.json::<ApiResponse>().await?;
        if resp.items.is_empty() {
            println!("{}", format!("No books found for the genre: {}", genre).red());
        } else {
            let mut rng = thread_rng();
            let random_index = rng.gen_range(0..resp.items.len());
            let book = &resp.items[random_index].volume_info;

            // Printing out the details of the book with colors and symbols for better readability.
            println!("\n{}\n", format!("Title: {}", book.title).bold().underline().blue());
            if let Some(authors) = &book.authors {
                println!("{}", format!("Author(s): {}", authors.join(", ")).italic().yellow());
            }
            if let Some(description) = &book.description {
                println!("\n{}\n", format!("Description: {}", description).green());
            }
        }
    } else if !matches.is_present("genrelist") {
        // If neither 'genre' nor 'genrelist' is provided, show a default message or help.
        println!("Please provide a genre with --genre or use --genrelist to list available genres.");
    }

    Ok(())
}
