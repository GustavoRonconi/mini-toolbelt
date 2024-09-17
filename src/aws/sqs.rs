use std::thread;
use std::time::Duration;

use aws_config::profile::ProfileFileCredentialsProvider;

#[derive(Debug)]
pub enum FileTypeOutput {
    Json,
    Csv,
}

pub struct SqsReceiveMessage<'a> {
    queue_url: &'a String,
    file_path: &'a String,
    file_type: &'a FileTypeOutput,

}

impl<'a> SqsReceiveMessage<'a> {
    pub fn new(queue_url: &'a String, file_path: &'a String, file_type: &'a FileTypeOutput) -> Self {
        // client
        let credentials_provider = ProfileFileCredentialsProvider::builder()
        .profile_name("my-profile")  // Use your desired profile
        .build();
        let client = aws_sdk_sqs::Client::new(&credentials_provider, aws_sdk_sqs::Region::default());

        Self {
            queue_url,
            file_path,
            file_type,
        }
    }

    async fn save_content(&self) {
        println!("Saving content to file, {}, {}, {:?}", self.queue_url, self.file_path, self.file_type);
        thread::sleep(Duration::from_secs(5));
    }

    pub async fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        

        self.save_content().await;
        Ok(())        
    }
}