use clap::{Arg, Command}; // Command-line argument parser
use colored::*; // For colored terminal output
use serde::Deserialize; // For deserializing JSON
use rand::{thread_rng, Rng}; // For generating random numbers
use serde_json::Value; // For storing JSON values
use std::error::Error;

#[derive(Deserialize, Debug)]
struct GoogleApiResponse {
    items: Option<Vec<GoogleBook>>,
}

#[derive(Deserialize, Debug)]
struct GoogleBook {
    #[serde(rename = "volumeInfo")]
    volume_info: GoogleVolumeInfo,
}

#[derive(Deserialize, Debug)]
struct GoogleVolumeInfo {
    description: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    work_count: usize,
    works: Vec<Book>,
}

#[derive(Deserialize, Debug)]
struct Author {
    name: String,
}

#[derive(Deserialize, Debug)]
struct Book {
    title: String,
    authors: Vec<Author>,
    key: Option<String>,
}

async fn fetch_isbn_from_partner_api(work_key: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://openlibrary.org{}/editions.json", work_key);
    let resp = reqwest::get(&url).await?.text().await?;

    let json: Value = serde_json::from_str(&resp)?;

    if let Some(entries) = json.get("entries").and_then(|e| e.as_array()) {
        for entry in entries {
            if let Some(isbn_13) = entry.get("isbn_13").and_then(|i| i.as_array()) {
                if let Some(isbn_value) = isbn_13.first().and_then(|i| i.as_str()) {
                    return Ok(isbn_value.to_string());
                }
            }
            if let Some(isbn_10) = entry.get("isbn_10").and_then(|i| i.as_array()) {
                if let Some(isbn_value) = isbn_10.first().and_then(|i| i.as_str()) {
                    return Ok(isbn_value.to_string());
                }
            }
        }
        Err("No ISBNs found in the editions.".into())
    } else {
        Err("No 'entries' key present in JSON response.".into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = thread_rng();
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

    // Handle genrelist parameter
    let genrelist_present = matches.is_present("genrelist");
    if genrelist_present {
        println!("{}", "\nAvailable genres include:".bold().underline().blue());
        // List all genres here
        println!("\n{}", "Arts, Architecture, Art Instruction, Art History, Dance, Design, Fashion, Film, Graphic Design, Music, Music Theory, Painting, Photography, Animals, Bears, Cats, Kittens, Dogs, Puppies, Fiction, Fantasy, Historical Fiction, Horror, Humor, Literature, Magic, Mystery and detective stories, Plays, Poetry, Romance, Science Fiction, Short Stories, Thriller, Young Adult, Science & Mathematics, Biology, Chemistry, Mathematics, Physics, Programming, Business & Finance, Management, Entrepreneurship, Business Economics, Business Success, Finance, Childrens, Kids Books, Stories in Rhyme, Baby Books, Bedtime Books, Picture Books, History, Ancient Civilization, Archaeology, Anthropology, World War II, Social Life and Customs, Health & Wellness, Cooking, Cookbooks, Mental Health, Exercise, Nutrition, Self-help, Biography, Autobiographies, History, Politics and Government, World War II, Women, Kings and Rulers, Composers, Artists, Social Sciences, Anthropology, Religion, Political Science, Psychology, Places, Brazil, India, Indonesia, United States, Textbooks, History, Mathematics, Geography, Psychology, Algebra, Education, Business & Economics, Science, Chemistry, English Language, Physics, Computer Science...".green());
        println!("{}", "\nExtra subjects here: https://openlibrary.org/subjects".italic().magenta());
    }

    // Determine the genre to search for
    let genre_search = if !genrelist_present && !matches.is_present("genre") {
        // Default to "science fiction" if no genre is provided
        Some("science_fiction")
    } else {
        // Use the provided genre, if there is one
        matches.value_of("genre")
    };

    // Perform the genre search if applicable
    if let Some(genre) = genre_search {
        perform_genre_search(genre, &mut rng).await?;
    }

    Ok(())
}

async fn perform_genre_search(genre: &str, rng: &mut impl Rng) -> Result<(), Box<dyn Error>> {
    let genre = genre.replace(' ', "_");
    let base_url = format!("https://openlibrary.org/subjects/{}.json", genre);
    let results_per_page = 12;

    // Fetch the initial response as a raw string
    let response_text = reqwest::get(&base_url).await?.text().await?;

    // Attempt to deserialize the initial response
    let initial_resp = match serde_json::from_str::<ApiResponse>(&response_text) {
        Ok(resp) => resp,
        Err(e) => {
            println!("Deserialization Error: {:?}", e);
            return Err(Box::new(e));
        }
    };

    let total_works = initial_resp.work_count;
    let max_offset = std::cmp::min(total_works, (total_works / results_per_page) * results_per_page);

    let offset = rng.gen_range(0..=max_offset);
    let paginated_url = format!("{}?offset={}", base_url, offset);

    // Fetch the paginated response as a raw string
    let paginated_response_text = reqwest::get(&paginated_url).await?.text().await?;

    // Attempt to deserialize the paginated response
    let paginated_resp = match serde_json::from_str::<ApiResponse>(&paginated_response_text) {
        Ok(resp) => resp,
        Err(e) => {
            println!("Paginated Deserialization Error: {:?}", e);
            return Err(Box::new(e));
        }
    };

    if paginated_resp.works.is_empty() {
        println!("\n{}", format!("No books found in the genre: {}", genre.replace('_', " ")).red());
        return Ok(());
    }

    let random_index = rng.gen_range(0..paginated_resp.works.len());
    let book = &paginated_resp.works[random_index];

    // Initialize variables for the ISBN and the potential fallback search by title
    let mut isbn = String::new();
    let mut fallback_to_title_search = false;

    // Query the Partner API to get ISBN
    if let Some(key) = &book.key {
        match fetch_isbn_from_partner_api(key).await {
            Ok(fetched_isbn) => {
                isbn = fetched_isbn;
            },
            Err(_) => {
                // If ISBN is not found, set the flag to use title search later
                fallback_to_title_search = true;
            }
        }
    }

    // If the ISBN is not empty, attempt to search with it
    if !isbn.is_empty() {
	// Use is_err() to check if search_google_books_by_isbn returns an Err
	if search_google_books_by_isbn(&isbn, book).await.is_err() {
            // If ISBN search fails, fallback to title search
            fallback_to_title_search = true;
	}
    } else {
	// If the ISBN is empty, set fallback to title search
	fallback_to_title_search = true;
    }

    // Execute the fallback title search if necessary
    if fallback_to_title_search {
	search_google_books_by_title(&book.title, book).await?;
    }

    Ok(())
}

// Function to search Google Books by ISBN and display results
async fn search_google_books_by_isbn(isbn: &str, book: &Book) -> Result<(), Box<dyn Error>> {
    let google_books_url = format!("https://www.googleapis.com/books/v1/volumes?q=isbn:{}", isbn);
    let resp = reqwest::get(&google_books_url).await?.text().await?;
    let google_resp: GoogleApiResponse = serde_json::from_str(&resp)?;
    display_google_books_results(google_resp, book)
}

// Function to search Google Books by title and display results
async fn search_google_books_by_title(title: &str, book: &Book) -> Result<(), Box<dyn Error>> {
    let google_books_url = format!("https://www.googleapis.com/books/v1/volumes?q=intitle:\"{}\"", title);
    let resp = reqwest::get(&google_books_url).await?.text().await?;
    let google_resp: GoogleApiResponse = serde_json::from_str(&resp)?;
    display_google_books_results(google_resp, book)
}

// Function to display Google Books results
fn display_google_books_results(google_resp: GoogleApiResponse, book: &Book) -> Result<(), Box<dyn Error>> {
    println!("\n{}\n", format!("Title: {}", book.title).bold().underline().blue());

    // Display authors
    if !book.authors.is_empty() {
	let authors_list = book.authors.iter()
            .map(|author| author.name.clone()) // Clone the String to get an owned String
            .collect::<Vec<String>>() // Now you have Vec<String> instead of Vec<&String>
            .join(", "); // You can now use join on a Vec<String>
	println!("{}", format!("Author(s): {}", authors_list).italic().yellow());
    } else {
	println!("{}", "Author information not available".italic().yellow());
    }


    // Display description if available
    if let Some(items) = google_resp.items {
        if let Some(first_book) = items.first() {
            if let Some(description) = &first_book.volume_info.description {
                println!("\n{}\n", format!("Description: {}", description).green());
            } else {
                println!("\n{}\n", "Description: Not available".italic().green());
            }
        }
    } else {
        println!("\n{}\n", "Description: Not available".italic().green());
    }

    // Display Open Library URL
    if let Some(key) = &book.key {
        println!("Open Library URL: https://openlibrary.org{}", key);
    }

    Ok(())
}
