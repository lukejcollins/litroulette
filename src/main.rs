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
    key: Option<String>,  // Updated to handle null values
    // ... Add other fields as necessary, marking them as optional if they can be null
}

async fn fetch_isbn_from_partner_api(work_key: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://openlibrary.org{}/editions.json", work_key);
    let resp = reqwest::get(&url).await?.text().await?;
    println!("Response: {}", resp); // Log the full response

    let json: Value = serde_json::from_str(&resp)?;

    if let Some(entries) = json.get("entries").and_then(|e| e.as_array()) {
        for entry in entries {
            if let Some(isbn_13) = entry.get("isbn_13").and_then(|i| i.as_array()) {
                if let Some(isbn_value) = isbn_13.first().and_then(|i| i.as_str()) {
                    println!("Found ISBN: {}", isbn_value); // Log each found ISBN
                    return Ok(isbn_value.to_string());
                }
            }
            if let Some(isbn_10) = entry.get("isbn_10").and_then(|i| i.as_array()) {
                if let Some(isbn_value) = isbn_10.first().and_then(|i| i.as_str()) {
                    println!("Found ISBN: {}", isbn_value); // Log each found ISBN
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
    println!("Raw API Response: {}", response_text);

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
    println!("Raw Paginated API Response: {}", paginated_response_text);

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

    // Try to get the ISBN from the editions page using the work key
    let mut search_query = String::new();
    if let Some(key) = &book.key {
        match fetch_isbn_from_partner_api(key).await {
            Ok(fetched_isbn) => {
                println!("ISBN found: {}", fetched_isbn);
                search_query = format!("isbn:{}", fetched_isbn);
            },
            Err(e) => {
                println!("Error fetching ISBN: {}", e);
                println!("Falling back to title search.");
                search_query = format!("intitle:\"{}\"", book.title);
            }
        }
    } else {
        println!("No key available for the book, falling back to title search.");
        search_query = format!("intitle:\"{}\"", book.title);
    }

    // Perform search with either ISBN or title
    let google_books_url = format!("https://www.googleapis.com/books/v1/volumes?q={}", search_query);
    let google_books_response = reqwest::get(&google_books_url).await?.text().await?;
    let google_resp: GoogleApiResponse = serde_json::from_str(&google_books_response)?;

    // Display the results
    println!("\n{}\n", format!("Title: {}", book.title).bold().underline().blue());
    if !book.authors.is_empty() {
        let authors_list = book.authors.iter()
            .map(|author| author.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
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
                println!("\n{}\n", "Description not available".italic().green());
            }
        }
    } else {
        println!("\nDescription not available");
    }

    // Display Open Library URL
    if let Some(key) = &book.key {
        println!("Open Library URL: https://openlibrary.org{}", key);
    }

    Ok(())
}
