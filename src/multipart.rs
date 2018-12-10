use actix_web::{
    dev, Error, multipart,
    error, 
};
use futures::{future, Future, Stream,};
use std::{fs as sfs};
use std::io::Write;
use chrono::prelude::{Utc};

pub fn handle_multipart_item(
    item: multipart::MultipartItem<dev::Payload>,
) -> Box<Stream<Item = String, Error = Error>> {
    match item {
        multipart::MultipartItem::Field(field) => { 
            let mut store: String = "bin/".to_owned();
            let now = Utc::now().timestamp_millis().to_string().to_owned();
            store.push_str(&now);
            Box::new(save_file(field, store).into_stream())
        }
        multipart::MultipartItem::Nested(mp) => Box::new(
            mp.map_err(error::ErrorInternalServerError)
                .map(handle_multipart_item)
                .flatten(),
        ),
    }
}

fn save_file(
    field: multipart::Field<dev::Payload>,
    fname: String,
) -> Box<Future<Item = String, Error = Error>> {
    let mut file = match sfs::File::create(&*fname) {
        Ok(file) => file,
        Err(e) => return Box::new(future::err(error::ErrorInternalServerError(e))),
    };
    Box::new(
        field
            .fold(0i64, move |acc, bytes| {
                let rt = file
                    .write_all(bytes.as_ref())
                    .map(|_| acc + bytes.len() as i64)
                    .map_err(|e| {
                        println!("file.write_all failed: {:?}", e);
                        error::MultipartError::Payload(error::PayloadError::Io(e))
                    });
                future::result(rt)
            })
            .map(|_| fname)
            .map_err(|e| {
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}
