use async_trait::async_trait;
use aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessages;
use aws_sdk_sqs::{Client as SqsClient, Error as SqsError};

#[async_trait]
pub trait SqsClientTrait {
    async fn get_approximate_number_of_messages(&self, queue_url: &String)
        -> Result<i64, SqsError>;
}

#[async_trait]
impl SqsClientTrait for SqsClient {
    async fn get_approximate_number_of_messages(
        &self,
        queue_url: &String,
    ) -> Result<i64, SqsError> {
        let request = self
            .get_queue_attributes() 
            .queue_url(queue_url)
            .attribute_names(ApproximateNumberOfMessages)
            .send();
        let response = request.await?;
        let attributes = response.attributes.unwrap();
        let number_of_messages = attributes.get(&ApproximateNumberOfMessages).unwrap();
        Ok(number_of_messages.parse().unwrap())
    }
}

#[derive(Debug)]
pub enum FileTypeOutput {
    Json,
    Csv,
}

pub struct SqsJob<'a, T: SqsClientTrait> {
    queue_url: &'a String,
    destination_file_path: &'a String,
    file_type: &'a FileTypeOutput,
    sqs_client: T,
}

impl<'a, T: SqsClientTrait> SqsJob<'a, T> {
    pub fn new(
        queue_url: &'a String,
        destination_file_path: &'a String,
        file_type: &'a FileTypeOutput,
        sqs_client: T,
    ) -> Self {
        Self {
            queue_url,
            destination_file_path,
            file_type,
            sqs_client,
        }
    }

    pub async fn dump_messages(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "{}, {}, {}, {:?}",
            self.sqs_client
                .get_approximate_number_of_messages(self.queue_url)
                .await
                .unwrap(), 
            self.queue_url,
            self.destination_file_path,
            self.file_type,
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};

    mock! {
        SqsClient {}

        #[async_trait]
        impl SqsClientTrait for SqsClient {
            async fn get_approximate_number_of_messages(&self, queue_url: &String) -> Result<i64, SqsError>;
        }
    }

    #[tokio::test]
    async fn test_dump_messages() {
        let queue_url = String::from("https://sqs.us-east-1.amazonaws.com/80398EXAMPLE/MyQueue");
        let destination_file_path = String::from("output.json");
        let file_type = FileTypeOutput::Json;
        let mut mock_sqs_client = MockSqsClient::new();
        mock_sqs_client
            .expect_get_approximate_number_of_messages()
            .with(eq(queue_url.clone()))
            .returning(|_| Ok(10));

        let sqs_job = SqsJob::new(&queue_url, &destination_file_path, &file_type, mock_sqs_client);
        assert_eq!(sqs_job.dump_messages().await.unwrap(), ());
    }

    // TODO: write test for other methods

}
