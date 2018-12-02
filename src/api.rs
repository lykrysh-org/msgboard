use actix::prelude::Addr;
use actix_web::middleware::Response;
use actix_web::middleware::identity::RequestIdentity;
use actix_web::{
    error, fs::NamedFile, http, AsyncResponder, Form, FutureResponse, HttpRequest,
    HttpResponse, Path, Responder, Result, 
};
use futures::{future, Future};
use tera::{Context, Tera};

use db::{AllTasks, CreateTask, DbExecutor, DeleteTask, ToggleTask, UploadTask};
use session::{self, FlashMessage};

pub struct AppState {
    pub template: Tera,
    pub db: Addr<DbExecutor>,
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

                //Session is set during operations on other endpoints
                //that can redirect to index
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
                    .render("index.html.tera", &context)
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
    secret: String,
    whosent: String,
    description: String,
}

pub fn create(
    (req, params): (HttpRequest<AppState>, Form<CreateForm>),
) -> FutureResponse<HttpResponse> {
    let name = params.whosent.clone();
    req.state()
        .db
        .send(CreateTask {
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
    println!("{:?}", req);
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

