use md5;
use reqwest::Client;

#[derive(Debug)]
pub struct AvatarClient {
    pub http_client: Client,
    pub base_url: String,
    pub default_img: String,
}

impl AvatarClient {
    pub fn new(base_url: String, default_img: String) -> Self {
        AvatarClient {
            http_client: Client::new(),
            base_url,
            default_img,
        }
    }
    pub async fn send_request(&self, email: &str) -> Result<String, reqwest::Error> {
        //create hash
        let hash = md5::compute(email.as_bytes());

        Ok(format!(
            "{}/{:x}?d={}",
            self.base_url, hash, self.default_img
        ))
    }
}
