use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs::read_dir;

// Structure for directory entries
#[derive(Serialize)]
struct DirectoryEntry {
    name: String,
    is_dir: bool,
}

// Function to handle directory listing
async fn list_directory(path: web::Path<String>) -> impl Responder {
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
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Interactive Directory Listing</title>
        <script>
            async function fetchDirectory(path = '') {
                try {
                    const response = await fetch(`/list/${path}`);
                    
                    if (!response.ok) {
                        throw new Error(`HTTP error! status: ${response.status}`);
                    }

                    const data = await response.json();

                    const listContainer = document.getElementById("directory-list");
                    listContainer.innerHTML = "";

                    // Add a 'Back' button if we're not at the root
                    if (path !== '') {
                        const parentPath = path.split('/').slice(0, -2).join('/') + '/';
                        const backButton = document.createElement("div");
                        backButton.innerHTML = "<button>Back</button>";
                        backButton.onclick = () => fetchDirectory(parentPath);
                        listContainer.appendChild(backButton);
                    }

                    data.forEach(entry => {
                        const listItem = document.createElement("div");
                        if (entry.is_dir) {
                            // Directories are clickable to navigate into
                            listItem.innerHTML = `<strong>${entry.name}</strong>`;
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Serve the root URL with the HTML content
            .route("/", web::get().to(index))
            // Route for fetching the directory listing as JSON
            .route("/list/{path:.*}", web::get().to(list_directory))
            // Serve files to make them downloadable
            .service(fs::Files::new("/", ".").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
