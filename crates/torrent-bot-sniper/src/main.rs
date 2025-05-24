// get the list of snipe requests;
// ask OpenAI to turn these requests into search queries;
// search for each query;
// ask OpenAI to filter the results and keep only the torrents matching the requests;
// if there's a matching result, add it to bookmarks;
// remove the completed request from the list;

mod config;
mod snipe_db;

fn main() {
    println!("Hello, world!");
}
