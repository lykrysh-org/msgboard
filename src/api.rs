use actix::prelude::Addr;
use actix_web::middleware::Response;
use actix_web::middleware::identity::RequestIdentity;
use actix_web::{
    error, fs::NamedFile, http, AsyncResponder, Form, FutureResponse, HttpRequest,
    HttpResponse, Path, Responder, Result, 
    dev, Error, HttpMessage, multipart,
};
use futures::{future, Future, Stream};
use tera::{Context, Tera};
use db::{AllTasks, CreateTask, DbExecutor, DeleteTask, ToggleTask, UploadTask, CancelTask};
use session::{self, FlashMessage};

use std::{fs as sfs};
use std::io::Write;

pub struct AppState {
    pub template: Tera,
    pub db: Addr<DbExecutor>,
}

pub fn save(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    Box::new(
        req.multipart()
            .map_err(error::ErrorInternalServerError)
            .map(handle_multipart_item)
            .flatten()
            .collect()
            .map(|sizes| HttpResponse::Ok().json(sizes))
            .map_err(|e| {
                println!("failed: {}", e);
                e
            }),
    )
}

fn handle_multipart_item(
    item: multipart::MultipartItem<dev::Payload>,
) -> Box<Stream<Item = i64, Error = Error>> {
    match item {
        multipart::MultipartItem::Field(field) => { 
            Box::new(save_file(field).into_stream())
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
) -> Box<Future<Item = i64, Error = Error>> {

    let file_path_string = "TEST.file";
    let mut file = match sfs::File::create(file_path_string) {
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
            .map_err(|e| {
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}

pub fn index(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(AllTasks)
        .from_err()
        .and_then(move |res| match res {
            Ok(tasks) => {
                let mut context = Context::new();
                context.insert("tasks", &tasks);

                //Session
                if let Some(flash) = session::get_flash(&req)? {
                    context.insert("msg", &(flash.kind, flash.message));
                    session::clear_flash(&req);
                }

                //Identity
                if let Some(id) = req.identity() {
                    context.insert("welcomeback", &id)
                }

                let rendered = req.state()
                    .template
                    .render("index.tera", &context)
                    .map_err(|e| {
                        error::ErrorInternalServerError(e.description().to_owned())
                    })?;

                Ok(HttpResponse::Ok().body(rendered))
            }
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct CreateForm {
    inheritedid: String,
    secret: String,
    whosent: String,
    description: String,
}

pub fn create(
    (req, params): (HttpRequest<AppState>, Form<CreateForm>),
) -> FutureResponse<HttpResponse> {
    let replyid: Option<i32> = match params.inheritedid.clone().as_ref() {
        "none" => None,
        _whatever => {
            let i: i32 = _whatever.parse().unwrap_or(0);
            Some(i)
        },
    };
    let name = params.whosent.clone();
    req.state()
        .db
        .send(CreateTask {
            inheritedid: replyid,
            secret: params.secret.clone(),
            whosent: name.to_string(),
            description: params.description.clone().trim().to_string(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => {
                session::set_flash(
                    &req,
                    FlashMessage::success(&format!("Thanks {}!", name)),
                )?;
                &req.remember(name.to_owned());
                Ok(redirect_to("/"))
            }
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct UpdateParams {
    id: i32,
}

#[derive(Deserialize)]
pub struct UpdateForm {
    _method: String,
    password: String,
}

pub fn update(
    (req, params, form): (HttpRequest<AppState>, Path<UpdateParams>, Form<UpdateForm>),
) -> FutureResponse<HttpResponse> {
    match form._method.as_ref() {
        "put" => toggle(req, params, &form.password),
        "delete" => delete(req, params, &form.password),
        unsupported_method => {
            let msg = format!("Unsupported HTTP method: {}", unsupported_method);
            future::err(error::ErrorBadRequest(msg)).responder()
        }
    }
}

fn toggle(
    req: HttpRequest<AppState>,
    params: Path<UpdateParams>,
    mypw: &String,
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(ToggleTask { id: params.id, pw: mypw.to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(999) => {
                session::set_flash(&req, FlashMessage::error("Wrong password."))?;
                Ok(redirect_to("/"))
            },
            Ok(_) => Ok(redirect_to("/")),
            Err(e) => Err(e),
        })
        .responder()
}

fn delete(
    req: HttpRequest<AppState>,
    params: Path<UpdateParams>,
    mypw: &String,
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(DeleteTask { id: params.id, pw: mypw.to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(999) => {
                session::set_flash(&req, FlashMessage::error("Wrong password."))?;
                Ok(redirect_to("/"))
            },
            Ok(_) => {
                session::set_flash(&req, FlashMessage::success("Deleted."))?;
                Ok(redirect_to("/"))
            },
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct EditForm {
    description: String,
}

pub fn edit(
    (req, params, form): (HttpRequest<AppState>, Path<UpdateParams>, Form<EditForm>),
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(UploadTask { id: params.id, desc: form.description.trim().to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => {
                Ok(redirect_to("/"))
            },
            Err(e) => Err(e),
        })
        .responder()
}

pub fn cancel(
    (req, params): (HttpRequest<AppState>, Path<UpdateParams>),
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(CancelTask { id: params.id })
        .from_err()
        .and_then(move |res| match res {
            Ok(_) => {
                Ok(redirect_to("/"))
            },
            Err(e) => Err(e),
        })
        .responder()
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::Found()
        .header(http::header::LOCATION, location)
        .finish()
}

pub fn bad_request<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("static/errors/400.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}

pub fn not_found<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("static/errors/404.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}

pub fn internal_server_error<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("static/errors/500.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}

