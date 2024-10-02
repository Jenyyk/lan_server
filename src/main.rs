use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, cookie::Cookie, HttpRequest};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::read_dir;
use dotenv::dotenv; // Import dotenv
use std::env; // Import std::env

// Structure for directory entries
#[derive(Serialize)]
struct DirectoryEntry {
    name: String,
    is_dir: bool,
}

// Function to handle directory listing
async fn list_directory(path: web::Path<String>, req: HttpRequest) -> impl Responder {
    // Check for valid session (simple login check)
    let cookie = req.cookie("auth");
    if cookie.is_none() || cookie.unwrap().value() != "secret" {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    // If the path is empty, serve the current directory (".")
    let dir_path = if path.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(&*path)
    };

    let mut entries: Vec<DirectoryEntry> = Vec::new();

    // Check if the directory exists
    if !dir_path.exists() {
        return HttpResponse::NotFound().json("Directory not found");
    }

    match read_dir(dir_path.clone()).await {
        Ok(mut read_dir) => {
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    entries.push(DirectoryEntry {
                        name: entry.file_name().into_string().unwrap(),
                        is_dir: file_type.is_dir(),
                    });
                }
            }
        }
        Err(_) => return HttpResponse::InternalServerError().json("Failed to read directory"),
    }

    HttpResponse::Ok().json(entries)
}

// Serve HTML page directly from Rust code
async fn index() -> impl Responder {
    let html_content = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <link rel="stylesheet" href="assets/styles.css">
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Lan Server</title>
        <script>
            async function fetchDirectory(path = '') {
                try {
                    const response = await fetch(`/list/${path}`, {
                        credentials: 'include' // Include cookies for auth
                    });

                    if (!response.ok) {
                        throw new Error(`HTTP error! status: ${response.status}`);
                    }

                    const data = await response.json();

                    const listContainer = document.getElementById("directory-list");
                    listContainer.innerHTML = "";

                    const parentPath = path.split('/').slice(0, -2).join('/') + '/';
                    const backButton = document.createElement("div");
                    backButton.innerHTML = "<button>Back</button>";
                    backButton.onclick = () => fetchDirectory(parentPath || '');
                    listContainer.appendChild(backButton);

                    data.forEach(entry => {
                        const listItem = document.createElement("div");
                        if (entry.is_dir) {
                            // Directories are clickable to navigate into
                            listItem.innerHTML = `<strong>${entry.name}/</strong>`;
                            listItem.style.cursor = "pointer";
                            listItem.onclick = () => fetchDirectory(`${path}${entry.name}/`);
                        } else {
                            // Files are clickable for download
                            listItem.innerHTML = `<a href='/${path}${entry.name}' download>${entry.name}</a>`;
                        }
                        listContainer.appendChild(listItem);
                    });
                } catch (error) {
                    console.error("Error fetching directory:", error);
                    const listContainer = document.getElementById("directory-list");
                    listContainer.innerHTML = `<div style="color: red;">Error: ${error.message}</div>`;
                }
            }

            window.onload = () => fetchDirectory();
        </script>
    </head>
    <body>
        <h1>Directory Listing</h1>
        <div id="directory-list"></div>
    </body>
    </html>
    "#;

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)
}

// Function to handle login
async fn login(req: HttpRequest) -> impl Responder {
    let username = req.match_info().get("username").unwrap_or("");
    let password = req.match_info().get("password").unwrap_or("");

    // Log incoming username and password (for debugging)
    println!("Login attempt: {} / {}", username, password);

    // Load user credentials from environment variables
    let users = env::var("USERS").unwrap_or_else(|_| "admin:password".to_string());

    // Parse users into a HashMap
    let mut credentials = HashMap::new();
    for user in users.split(",") {
        let parts: Vec<&str> = user.split(':').collect();
        if parts.len() == 2 {
            credentials.insert(parts[0], parts[1]);
        }
    }

    // Check credentials against the parsed HashMap
    if let Some(&expected_password) = credentials.get(username) {
        if password == expected_password {
            // Create a session cookie that expires when the browser is closed
            let mut cookie = Cookie::new("auth", "secret");
            cookie.set_http_only(true);
            cookie.set_path("/"); // Set path for the cookie
            cookie.set_same_site(actix_web::cookie::SameSite::Strict); // SameSite attribute

            // Do not set an expiration date, making it a session cookie
            return HttpResponse::SeeOther()
                .cookie(cookie)
                .header("Location", "/") // Redirect to the root
                .finish();
        }
    }

    HttpResponse::Unauthorized().body("Invalid credentials")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("starting server on port 8080");
    dotenv().ok(); // Load environment variables from .env file

    HttpServer::new(|| {
        App::new()
            // Route for the login (username and password are passed in the URL)
            .route("/login/{username}/{password}", web::get().to(login))
            // Serve the root URL with the HTML content
            .route("/", web::get().to(index))
            // Route for fetching the directory listing as JSON
            .route("/list/{path:.*}", web::get().to(list_directory))
            // Serve files to make them downloadable
            .service(fs::Files::new("/", ".").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
