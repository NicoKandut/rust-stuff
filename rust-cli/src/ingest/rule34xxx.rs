use crate::ingest::unified;

#[derive(Default, serde::Deserialize, Debug)]
pub struct Page {
  count: usize,
  posts: Vec<Post>,
}

#[derive(Default, serde::Deserialize, Debug)]
pub struct Post {
  height: String,
  score: String,
  file_url: String,
  parent_id: String,
  sample_url: String,
  sample_width: String,
  sample_height: String,
  preview_url: String,
  rating: String,
  tags: String,
  id: String,
  width: String,
  change: String,
  md5: String,
  creator_id: String,
  has_children: String,
  created_at: String,
  status: String,
  source: String,
  has_notes: String,
  has_comments: String,
  preview_width: String,
  preview_height: String,
}

struct Rule34xxxError {}

pub async fn query_posts(from: std::time::Instant) -> Result<Vec<unified::Post>, reqwest::Error> {
  let url = "https://api.rule34.xxx/index.php?page=dapi&s=post&q=index";
  let response = reqwest::get(url).await?;
  let text = response.text().await?;
  
  let page: Page = serde_xml_rs::from_str(&text).unwrap();

  let posts = unify(page.posts);

  return Ok(posts);
}

fn unify(posts: Vec<Post>)-> Vec<unified::Post> {
  return posts.iter()
  .map(|&post| -> unified::Post  {
    unified::Post{

    }
  })
  .collect::<Vec<_>>();
}
