use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

const DATA_FILE: &str = "library_data.json";

#[derive(Serialize, Deserialize, Clone)]
struct Book {
    id: String,
    title: String,
    author: String,
    copies_total: u32,
    copies_available: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct Library {
    books: Vec<Book>,
}

fn data_path() -> PathBuf {
    PathBuf::from(DATA_FILE)
}

fn default_library() -> Library {
    Library {
        books: vec![
            Book {
                id: "B001".into(),
                title: "1984".into(),
                author: "George Orwell".into(),
                copies_total: 3,
                copies_available: 3,
            },
            Book {
                id: "B002".into(),
                title: "Pride and Prejudice".into(),
                author: "Jane Austen".into(),
                copies_total: 2,
                copies_available: 2,
            },
            Book {
                id: "B003".into(),
                title: "To Kill a Mockingbird".into(),
                author: "Harper Lee".into(),
                copies_total: 4,
                copies_available: 4,
            },
            Book {
                id: "B004".into(),
                title: "The Great Gatsby".into(),
                author: "F. Scott Fitzgerald".into(),
                copies_total: 2,
                copies_available: 2,
            },
        ],
    }
}

fn save_data(library: &Library) -> Result<()> {
    let text = serde_json::to_string_pretty(library)?;
    fs::write(data_path(), text)?;
    Ok(())
}

fn load_data() -> Library {
    let path = data_path();
    if !path.exists() {
        let lib = default_library();
        if let Err(err) = save_data(&lib) {
            eprintln!("Warning: failed to write default data: {err}");
        }
        return lib;
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<Library>(&content) {
            Ok(lib) => lib,
            Err(err) => {
                eprintln!("Data file is corrupted ({err}). Resetting to defaults.");
                let lib = default_library();
                if let Err(err) = save_data(&lib) {
                    eprintln!("Warning: failed to write default data: {err}");
                }
                lib
            }
        },
        Err(err) => {
            eprintln!("Could not read data file ({err}). Using default catalog.");
            default_library()
        }
    }
}

fn borrowed_count(book: &Book) -> u32 {
    book.copies_total.saturating_sub(book.copies_available)
}

fn print_book_table(library: &Library, indices: &[usize], show_available: bool, show_borrowed: bool) {
    if indices.is_empty() {
        println!("No books to display.");
        return;
    }

    let mut headers = vec!["#".to_string(), "ID".into(), "Title".into(), "Author".into()];
    if show_available {
        headers.push("Available".into());
    }
    if show_borrowed {
        headers.push("Borrowed".into());
    }

    let mut rows: Vec<Vec<String>> = Vec::new();
    for (display_idx, book_index) in indices.iter().enumerate() {
        let book = &library.books[*book_index];
        let mut row = vec![
            (display_idx + 1).to_string(),
            book.id.clone(),
            book.title.clone(),
            book.author.clone(),
        ];
        if show_available {
            row.push(book.copies_available.to_string());
        }
        if show_borrowed {
            row.push(borrowed_count(book).to_string());
        }
        rows.push(row);
    }

    let col_widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(col_idx, header)| {
            std::iter::once(header.len())
                .chain(rows.iter().map(|r| r[col_idx].len()))
                .max()
                .unwrap_or(header.len())
        })
        .collect();

    let fmt_row = |values: &[String]| -> String {
        values
            .iter()
            .enumerate()
            .map(|(i, value)| format!("{:<width$}", value, width = col_widths[i]))
            .collect::<Vec<_>>()
            .join(" | ")
    };

    println!("{}", fmt_row(&headers));
    let divider = col_widths
        .iter()
        .map(|w| "-".repeat(*w))
        .collect::<Vec<_>>()
        .join("-+-");
    println!("{divider}");
    for row in rows {
        println!("{}", fmt_row(&row));
    }
}

fn read_choice(prompt: &str) -> Option<String> {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return None;
    }
    Some(input.trim().to_string())
}

fn select_book_index(library: &Library, indices: &[usize], prompt: &str) -> Option<usize> {
    let input = read_choice(prompt)?;
    if input.is_empty() || input.eq_ignore_ascii_case("q") {
        return None;
    }

    if let Ok(num) = input.parse::<usize>() {
        if num >= 1 && num <= indices.len() {
            return Some(indices[num - 1]);
        }
        println!("Invalid selection.");
        return None;
    }

    let lowered = input.to_lowercase();
    for idx in indices {
        if library.books[*idx].id.to_lowercase() == lowered {
            return Some(*idx);
        }
    }

    println!("Book not found.");
    None
}

fn view_available(library: &Library) {
    let available_indices: Vec<usize> = library
        .books
        .iter()
        .enumerate()
        .filter(|(_, book)| book.copies_available > 0)
        .map(|(idx, _)| idx)
        .collect();
    println!("\nAvailable books:");
    print_book_table(library, &available_indices, true, false);
}

fn view_borrowed(library: &Library) {
    let borrowed_indices: Vec<usize> = library
        .books
        .iter()
        .enumerate()
        .filter(|(_, book)| borrowed_count(book) > 0)
        .map(|(idx, _)| idx)
        .collect();
    println!("\nCurrently borrowed books:");
    print_book_table(library, &borrowed_indices, false, true);
}

fn borrow_book(library: &mut Library) {
    let available_indices: Vec<usize> = library
        .books
        .iter()
        .enumerate()
        .filter(|(_, book)| book.copies_available > 0)
        .map(|(idx, _)| idx)
        .collect();

    if available_indices.is_empty() {
        println!("\nNo books are currently available to borrow.");
        return;
    }

    println!("\nSelect a book to borrow:");
    print_book_table(library, &available_indices, true, false);
    if let Some(book_idx) =
        select_book_index(library, &available_indices, "\nEnter # or ID (or press Enter to cancel): ")
    {
        let title;
        {
            let book = &mut library.books[book_idx];
            if book.copies_available == 0 {
                println!("No copies left to borrow.");
                return;
            }
            book.copies_available -= 1;
            title = book.title.clone();
        } // ✅ mutable borrow ends here

        if let Err(err) = save_data(library) {
        eprintln!("Warning: could not save data: {err}");
        }

        println!("You borrowed \"{}\".", title);

    }
}

fn return_book(library: &mut Library) {
    let borrowed_indices: Vec<usize> = library
        .books
        .iter()
        .enumerate()
        .filter(|(_, book)| borrowed_count(book) > 0)
        .map(|(idx, _)| idx)
        .collect();

    if borrowed_indices.is_empty() {
        println!("\nYou have no borrowed books to return.");
        return;
    }

    println!("\nSelect a book to return:");
    print_book_table(library, &borrowed_indices, false, true);
    if let Some(book_idx) =
        select_book_index(library, &borrowed_indices, "\nEnter # or ID (or press Enter to cancel): ")
    {
        let title;
    {
        let book = &mut library.books[book_idx];
        if book.copies_available >= book.copies_total {
            println!("All copies are already in the library.");
            return;
        }
        book.copies_available += 1;
        title = book.title.clone();
    } // ✅ mutable borrow ENDS HERE

    if let Err(err) = save_data(library) {
        eprintln!("Warning: could not save data: {err}");
    }

println!("Thank you for returning \"{}\".", title);

    }
}

fn menu() -> Option<String> {
    println!(
        "\nLibrary Menu
1) View available books
2) View borrowed books
3) Borrow a book
4) Return a book
5) Exit"
    );
    read_choice("Choose an option: ")
}

fn main() {
    let mut library = load_data();

    loop {
        match menu().as_deref() {
            Some("1") => view_available(&library),
            Some("2") => view_borrowed(&library),
            Some("3") => borrow_book(&mut library),
            Some("4") => return_book(&mut library),
            Some("5") => {
                println!("Goodbye!");
                break;
            }
            Some(_) => println!("Please choose a valid option (1-5)."),
            None => {
                println!("Input error. Exiting.");
                break;
            }
        }

        let _ = read_choice("\nPress Enter to continue...");
    }
}

