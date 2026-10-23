//! Read-only live check. No account, cookies or personal data required.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tube = innertube::InnerTube::new(innertube::Session::default(), None)?;
    let clients = innertube::Clients::bundled();
    let client = clients.get("WEB_REMIX").unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        tube.search_songs(client, "Beethoven piano", false),
    )
    .await??;
    println!("Search songs: {}", result.items.len());
    assert!(!result.items.is_empty(), "No live results");
    Ok(())
}
