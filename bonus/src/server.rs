use crate::task::{Task, TaskType};
use once_cell::sync::Lazy;
use std::error::Error;
use std::sync::mpsc as other_mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Semaphore};
use tokio::task;

pub trait ServerTrait {
    fn start_server(
        &self,
        address: String,
        tx: other_mpsc::Sender<Result<(), Box<dyn Error + Send>>>,
    );
}

pub struct Server;

static TASK_LIMIT: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(40));

impl ServerTrait for Server {
    #[tokio::main]
    async fn start_server(
        &self,
        address: String,
        tx: other_mpsc::Sender<Result<(), Box<dyn Error + Send>>>,
    ) {
        println!("Starting the server");
        let listener = TcpListener::bind(address)
            .await
            .expect("Failed to bind address");
        let _ = tx.send(Ok(())).unwrap();

        loop {
            let Ok((stream, _)) = listener.accept().await else {
                todo!()
            };
            tokio::spawn(async move {
                Self::handle_connection(stream).await;
            });
        }
    }
}

impl Server {
    async fn handle_connection(stream: TcpStream) {
        let (read, write) = tokio::io::split(stream);
        let mut buf_reader = BufReader::new(read);
        let mut buf_writer = BufWriter::new(write);
        // let mut clone = buf_writer.clone();
        let mut line = String::new();
        let (tx, mut rx) = mpsc::channel(50);

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if buf_writer.write_all(&[message]).await.is_err() {
                    eprintln!("Failed to write response");
                    break;
                }
                buf_writer.flush().await.expect("Failed to flush writer");
            }
        });

        while let Ok(bytes_read) = buf_reader.read_line(&mut line).await {
            if bytes_read == 0 {
                break;
            } // End of stream
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                Self::get_task_value(line, tx_clone).await;
            });
            line = String::new();
        }
    }

    async fn get_task_value(buf: String, tx: mpsc::Sender<u8>) {
        let numbers: Vec<&str> = buf.trim().split(':').collect();

        let task_type = match numbers.first().and_then(|n| n.parse::<u8>().ok()) {
            Some(t) => t,
            None => 0,
        };

        let seed = match numbers.last().and_then(|n| n.parse::<u64>().ok()) {
            Some(s) => s,
            None => 0,
        };
        let v = Task::execute_async(task_type, seed).await;
        let _ = tx.send(v).await;
    }
}
