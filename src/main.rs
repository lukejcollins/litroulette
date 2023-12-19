use clap::{Arg, Command}; // Command-line argument parser
use colored::*; // For colored terminal output
use serde::Deserialize; // For deserializing JSON
use rand::{thread_rng, Rng}; // For generating random numbers

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
    key: String,
    // Add other fields if necessary
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
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
    if matches.is_present("genrelist") {
        println!("{}", "\nAvailable genres include:".bold().underline().blue());
        // List all genres here
        println!("\n{}", "Arts, Architecture, Art Instruction, Art History, Dance, Design, Fashion, Film, Graphic Design, Music, Music Theory, Painting, Photography, Animals, Bears, Cats, Kittens, Dogs, Puppies, Fiction, Fantasy, Historical Fiction, Horror, Humor, Literature, Magic, Mystery and detective stories, Plays, Poetry, Romance, Science Fiction, Short Stories, Thriller, Young Adult, Science & Mathematics, Biology, Chemistry, Mathematics, Physics, Programming, Business & Finance, Management, Entrepreneurship, Business Economics, Business Success, Finance, Childrens, Kids Books, Stories in Rhyme, Baby Books, Bedtime Books, Picture Books, History, Ancient Civilization, Archaeology, Anthropology, World War II, Social Life and Customs, Health & Wellness, Cooking, Cookbooks, Mental Health, Exercise, Nutrition, Self-help, Biography, Autobiographies, History, Politics and Government, World War II, Women, Kings and Rulers, Composers, Artists, Social Sciences, Anthropology, Religion, Political Science, Psychology, Places, Brazil, India, Indonesia, United States, Textbooks, History, Mathematics, Geography, Psychology, Algebra, Education, Business & Economics, Science, Chemistry, English Language, Physics, Computer Science...".green());
	println!("{}", "\nExtra subjects here: https://openlibrary.org/subjects".italic().magenta());
    }

    // Handle genre parameter only if 'genrelist' is not the only argument
    if matches.is_present("genre") || !matches.is_present("genrelist") {
        let genre = matches.value_of("genre").unwrap_or("science_fiction").replace(' ', "_");
        let base_url = format!("https://openlibrary.org/subjects/{}.json", genre);
        let results_per_page = 12;

        let initial_resp = reqwest::get(&base_url).await?.json::<ApiResponse>().await?;
        let total_works = initial_resp.work_count;
        let max_offset = (total_works / results_per_page) * results_per_page;

        let offset = rng.gen_range(0..=max_offset);
        let paginated_url = format!("{}?offset={}", base_url, offset);

        let resp = reqwest::get(&paginated_url).await?.json::<ApiResponse>().await?;
        if resp.works.is_empty() {
            println!("\n{}", format!("No books found in the genre: {}", genre.replace('_', " ")).red());
            return Ok(());
        }

        let random_index = rng.gen_range(0..resp.works.len());
        let book = &resp.works[random_index];

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

        // Fetch description from Google Books API
        let google_books_url = format!("https://www.googleapis.com/books/v1/volumes?q=intitle:{}", book.title);
        let google_resp = reqwest::get(&google_books_url).await?.json::<GoogleApiResponse>().await?;
        if let Some(items) = google_resp.items {
            if let Some(first_book) = items.first() {
                if let Some(description) = &first_book.volume_info.description {
                    println!("\n{}\n", format!("Description: {}", description).green());
                } else {
                    println!("\n{}\n", "Description not available".italic().green());
                }
            }
        }

        println!("{}", format!("Open Library URL: https://openlibrary.org{}", book.key).italic().magenta());
    }

    Ok(())
}
