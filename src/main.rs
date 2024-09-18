use aws::clients;
use clap::{Parser, Subcommand, ValueEnum};
use crate::aws::sqs::{SqsJob, FileTypeOutput};

pub mod aws;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    SqsDumpMessages {
        #[arg(long)]
        queue_url: String,
        #[arg(long)]
        destination_file_path: String,
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
    let sqs_client = clients::get_sqs_client().await;

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        
        Commands::SqsDumpMessages { queue_url, destination_file_path, file_type } => {
            let file_type_output = FileTypeOutput::from(*file_type);
            let sqs_job = SqsJob::new(queue_url, destination_file_path, &file_type_output, sqs_client);

            sqs_job.dump_messages().await.unwrap();

            println!("{:?}, {:?}, {:?}", queue_url, destination_file_path, file_type);
        }
    }
}

