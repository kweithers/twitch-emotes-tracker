use chrono::prelude::*;
use futures::pin_mut;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::{ToSql, Type};
use tokio_postgres::{Error, NoTls};
use std::sync::Arc;
use tokio::sync::Mutex;
use multiset::HashMultiSet;

pub struct TableRow {
    pub streamer: String,
    pub date: NaiveDate,
    pub emote: String,
    pub n: i32,
}

async fn write(
    writer: BinaryCopyInWriter,
    data: &Vec<TableRow>,
) -> Result<usize, tokio_postgres::Error> {
    pin_mut!(writer);

    let mut row: Vec<&'_ (dyn ToSql + Sync)> = Vec::new();
    for m in data {
        row.clear();
        row.push(&m.streamer);
        row.push(&m.date);
        row.push(&m.emote);
        row.push(&m.n);
        writer.as_mut().write(&row).await?;
    }

    writer.finish().await?;
    Ok(data.len())
}

pub async fn set_write(arc : &Arc<Mutex<HashMultiSet<String>>>) -> Result<(), Error> {
    let (mut postgres_client, postgres_connection) =
        tokio_postgres::connect("host=db user=kmw password=bald", NoTls).await?;
    println!("connected to postgres");

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = postgres_connection.await {
            println!("connection error: {}", e);
        }
    });

    let current_date = Local::now().date_naive();
    let types: Vec<Type> = vec![Type::VARCHAR, Type::DATE, Type::VARCHAR, Type::INT4];
    
    let mut data = Vec::new();
    let mut counter = arc.lock().await;
    for key in counter.clone().distinct_elements() {
        let val = counter.count_of(key);
        let (streamer,emote) = {
            let mut s = key.split(":");
            (s.next().unwrap(),s.next().unwrap())
        };
        println!("{} {}", key, val);
        data.push(TableRow {
            streamer: streamer.to_owned(),
            date: current_date,
            emote: emote.to_owned(),
            n: val as i32,
        });
        counter.remove_all(key);
    }
    
    let tx = postgres_client.transaction().await?;
    let sink = tx
        .copy_in("COPY EMOTES (streamer,date,emote,n) FROM STDIN BINARY")
        .await?;
    let writer = BinaryCopyInWriter::new(sink, &types);
    let num_written = write(writer, &data).await?;
    tx.commit().await?;

    println!("wrote to postgres {} rows", num_written);
    Ok(())
}