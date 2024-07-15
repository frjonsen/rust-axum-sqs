use aws_sdk_sqs::Client;
use aws_sdk_sqs::types::Message;
use sqlx::PgPool;
use uuid::{uuid, Uuid};

pub struct SqsListener {
    client: Client,
    queue_url: String,
    pool: PgPool
}

impl SqsListener {
    pub async fn new(queue_url: String, pool: PgPool) -> Self {
        let config = aws_config::from_env().load().await;
        let client = Client::new(&config);

        SqsListener {
            client,
            queue_url,
            pool
        }
    }

    async fn handle_message(&self, message: &Message) {
        if let Some(m_id) = message.message_id() {
            println!("Received message with id {m_id}");
        } else {
           println!("Got message with no id") ;
        }
        
        if let Some(body) = message.body() {
            if let Ok(parsed_body) = serde_json::from_str::<crate::model::Foo>(body) {
                let insert = sqlx::query("INSERT INTO Foos (id, bar, zar) VALUES ($1, $2, $3)")
                    .bind(parsed_body.id)
                    .bind(parsed_body.bar)
                    .bind(parsed_body.zar)
                    .execute(&self.pool).await;

                if let Err(e) = insert {
                    println!("Failed to insert message: {}", e.to_string());
                }
            } else {
                println!("Failed to parse message");
            }
        }
    }

    async fn start_listen(&self) {
        let queue_url = self.queue_url.clone();
        let messages = self.client.receive_message()
            .wait_time_seconds(10)
            .queue_url(queue_url).send().await.unwrap();
        println!("Request returned {} messages", messages.messages().len());
        if let Some(messages) = messages.messages {
            for m in messages {
                self.handle_message(&m).await;

                let handle = m.receipt_handle().expect("Message had no receipt handle");
                println!("Done with message. Acknowledging it now.");
                self.client.delete_message().queue_url(self.queue_url.clone()).receipt_handle(handle).send().await.expect("Failed to delete message");
            }
        } else {
            println!("Got no messages on this iteration...")
        }
    }

    pub fn run(self) {
        tokio::spawn(async move {
            loop {
                self.start_listen().await;
            }
        });
    }
}