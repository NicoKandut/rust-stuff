extern crate serde;

mod ingest;

struct Source {
    name: String,
    query_posts: ingest::PostIngestFunction,
}

#[tokio::main]
async fn main() {

    let mut timer = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        time.tick().await;
        let result = tokio::spawn(async {
            // TODO: get last successfull query from db here.
            ingest::rule34xxx::query_posts(std::time::Instant::now());
        })
        .await;
    }
    match result {
        Ok(_) => print!(
            "[{:?}][rule34.xxx] Successfully synced posts",
            std::time::Instant::now()
        ),
        Err(_) => print!(
            "[{:?}][rule34.xxx] Failed to sync posts",
            std::time::Instant::now()
        ),
    };
}

async fn schedule() {
    

    
}
