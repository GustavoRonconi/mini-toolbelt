use clap::{Parser, Subcommand, ValueEnum};
use crate::aws::sqs::{SqsReceiveMessage, FileTypeOutput};

pub mod aws;

#[derive(Parser)]
#[command(name = "mini-toolbelt")]
#[command(version = "1.0")]
#[command(about = "Some important features for working with AWS resources.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    SaveSqsContent {
        #[arg(long)]
        queue_url: String,
        #[arg(long)]
        file_path: String,
        #[arg(value_enum, long)]
        file_type: FileTypeParam,
    },
}

#[derive(Debug)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FileTypeParam {
    Json,
    Csv,
}

impl From<FileTypeParam> for FileTypeOutput {
    fn from(file_type: FileTypeParam) -> Self {
        match file_type {
            FileTypeParam::Json => FileTypeOutput::Json,
            FileTypeParam::Csv => FileTypeOutput::Csv,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        
        Commands::SaveSqsContent { queue_url, file_path, file_type } => {
            let file_type_output = FileTypeOutput::from(*file_type);
            let sqs_receive_message = SqsReceiveMessage::new(queue_url, file_path, &file_type_output);

            sqs_receive_message.process().await.unwrap();

            println!("{:?}, {:?}, {:?}", queue_url, file_path, file_type);
        }
    }
}

