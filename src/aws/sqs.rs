use std::sync::Arc;

use async_trait::async_trait;
use aws_sdk_sqs::types::Message;
use aws_sdk_sqs::types::QueueAttributeName::ApproximateNumberOfMessages;
use aws_sdk_sqs::{Client as SqsClient, Error as SqsError};
use tokio::sync::mpsc;

#[async_trait]
pub trait SqsClientTrait {
    async fn get_approximate_number_of_messages(&self, queue_url: &String)
        -> Result<i64, SqsError>;
    
    async fn receive_messages(&self, queue_url: &String) -> Result<Vec<Message>, SqsError>;
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


    async fn receive_messages(&self, queue_url: &String) -> Result<Vec<Message>, SqsError> {
        let request = self
            .receive_message()
            .queue_url(queue_url)
            .max_number_of_messages(10)
            .send();
        let response = request.await?;
        
        Ok(response.messages.unwrap())
    }
}

#[derive(Debug)]
pub enum FileTypeOutput {
    Json,
    Csv,
}

pub struct Config {
    queue_url: String,
    destination_file_path: String,
    file_type: FileTypeOutput,
    sqs_client: SqsClient,
    max_concurrency: u16,
}

impl Config {
    pub fn new(
        queue_url: String,
        destination_file_path: String,
        file_type: FileTypeOutput,
        sqs_client: SqsClient,
        max_concurrency: u16,
    ) -> Self {
        Self {
            queue_url,
            destination_file_path,
            file_type,
            sqs_client,
            max_concurrency,
        }
    }
}

#[derive(Clone)]
pub struct SqsJob {
    config: Arc<Config>,
}

impl SqsJob {
    pub fn new(
        queue_url: String,
        destination_file_path: String,
        file_type: FileTypeOutput,
        sqs_client: SqsClient,
        max_concurrency: Option<u16>,
    ) -> Self {
        let max_concurrency = max_concurrency.unwrap_or(1);
        let config = Arc::new(Config::new(
            queue_url,
            destination_file_path,
            file_type,
            sqs_client,
            max_concurrency,
        ));

        Self {
            config,
        }
    }

    pub async fn dump_messages(&self) -> Result<(), Box<dyn std::error::Error>> {
        let n_messages = self.config.sqs_client.get_approximate_number_of_messages(&self.config.queue_url).await?;                

        
        
        let (tx, mut rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(self.config.max_concurrency as usize); 
        


        let mut tasks = Vec::new();
        for _ in 0..n_messages {
            let config = self.config.clone();
            let tx_clone = tx.clone();

            
            let task = tokio::spawn(async move {
                let messages = config.sqs_client.receive_messages(&config.queue_url).await.unwrap();
                for message in messages {
                    tx_clone.send(message).await.unwrap();
                }
            });

            tasks.push(task);
        }

        drop(tx);

        // consumer 
        tokio::spawn(async move {
            let mut count = 0;
            while let Some(message) = rx.recv().await {
                count += 1;
                // println!("Message {:?}", message);
            }

            println!("Total messages: {}", count);
        });

        for task in tasks {
            task.await?;
        }

        Ok(())

    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use aws_sdk_sqs::operation::get_queue_attributes::builders::GetQueueAttributesOutputBuilder;
//     use aws_sdk_sqs::operation::get_queue_attributes::GetQueueAttributesOutput;
//     use mockall::{mock, predicate::*};

//     trait Test {
//         fn get_queue_attributes(&self) -> GetQueueAttributesOutputBuilder;
//     }
//     mock! {
//         SqsClient {}

//         #[async_trait]
//         impl SqsClientTrait for SqsClient {
//             async fn get_approximate_number_of_messages(&self, queue_url: &String) -> Result<i64, SqsError>;
//         }

//         impl Test for SqsClient {
//             fn get_queue_attributes(&self) -> GetQueueAttributesOutputBuilder;
//         }
//     }

//     #[tokio::test]
//     async fn test_dump_messages() {
//         let queue_url = String::from("https://sqs.us-east-1.amazonaws.com/80398EXAMPLE/MyQueue");
//         let destination_file_path = String::from("output.json");
//         let file_type = FileTypeOutput::Json;
//         let mut mock_sqs_client = MockSqsClient::new();
//         let get_queue_attributes_input = GetQueueAttributesOutput::builder().set_attributes(Some(
//             vec![(ApproximateNumberOfMessages, "10".to_string())]
//                 .into_iter()
//                 .collect(),
//         ));

//         mock_sqs_client
//             .expect_get_queue_attributes()
//             .returning(move || get_queue_attributes_input.clone());

//         let sqs_job = SqsJob::new(
//             &queue_url,
//             &destination_file_path,
//             &file_type,
//             mock_sqs_client,
//         );
//         assert!(sqs_job.dump_messages().await.is_ok());
//     }
// }
