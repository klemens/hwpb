use db;
use diesel::prelude::*;
use errors::{self, ResultExt};
use rocket::{Config, Outcome, State};
use rocket::http::{Cookie, Cookies, Status};
use rocket::http::uri::URI;
use rocket::request::{self, FlashMessage, Form, FromRequest, Request};
use rocket::response::{Flash, Redirect};
use rocket_contrib::Template;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::path::PathBuf;
use user;

pub struct SiteAdmins(HashSet<String>);

impl SiteAdmins {
    pub fn new<'a, I: IntoIterator<Item = &'a str>>(users: I) -> Self {
        SiteAdmins(users.into_iter().map(|s| s.to_string()).collect())
    }
}

pub fn load_site_admins(config: &Config) -> errors::Result<SiteAdmins> {
    let site_admins = config
        .get_slice("site_admins")
        .chain_err(|| "No site_admins configured.")?
        .iter()
        .filter_map(|u| u.as_str());
    let site_admins = SiteAdmins::new(site_admins);

    if site_admins.0.is_empty() {
        return Err("You must configure at least one site_admin.".into())
    }

    Ok(site_admins)
}

#[derive(Deserialize, Serialize)]
pub struct User {
    name: String,
    site_admin: bool,
    tutor_years: HashSet<i16>,
    admin_years: HashSet<i16>,
}

impl User {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_tutor_for(&self, year: i16) -> bool {
        self.site_admin || self.tutor_years.contains(&year)
    }

    pub fn is_admin_for(&self, year: i16) -> bool {
        self.site_admin || self.admin_years.contains(&year)
    }

    pub fn is_site_admin(&self) -> bool {
        self.site_admin
    }

    pub fn ensure_tutor_for(&self, year: i16) -> errors::Result<()> {
        match self.is_tutor_for(year) {
            true => Ok(()),
            false => Err(format!("User {} is not a tutor for {}",
                self.name(), year).into()),
        }
    }

    pub fn ensure_admin_for(&self, year: i16) -> errors::Result<()> {
        match self.is_admin_for(year) {
            true => Ok(()),
            false => Err(format!("User {} is not an admin for {}",
                self.name(), year).into()),
        }
    }
}

fn load_user(cookies: &mut Cookies) -> request::Outcome<User, ()> {
    let user = cookies
        .get_private("user")
        .and_then(|cookie| {
            serde_json::from_str(cookie.value()).ok()
        });

    match user {
        Some(user) => Outcome::Success(user),
        None => Outcome::Forward(())
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        load_user(&mut request.cookies())
    }
}

pub struct SiteAdmin(User);

impl Deref for SiteAdmin {
    type Target = User;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for SiteAdmin {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<SiteAdmin, ()> {
        match load_user(&mut request.cookies()) {
            Outcome::Success(user) => {
                if user.site_admin {
                    Outcome::Success(SiteAdmin(user))
                } else {
                    Outcome::Failure((Status::Forbidden, ()))
                }
            }
            Outcome::Forward(()) => Outcome::Forward(()),
            Outcome::Failure(error) => Outcome::Failure(error),
        }
    }
}

pub struct NotLoggedIn;

impl<'a, 'r> FromRequest<'a, 'r> for NotLoggedIn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<NotLoggedIn, ()> {
        let user = request.cookies()
            .get_private("user");

        if user.is_none() {
            Outcome::Success(NotLoggedIn)
        } else {
            Outcome::Forward(())
        }
    }
}

#[get("/", rank = 2)]
fn nologin_index() -> Redirect {
    redirect_to_login("/")
}

#[get("/<_path..>", rank = 3)]
fn nologin_path(uri: &URI, _path: PathBuf, _user: NotLoggedIn) -> Redirect {
    redirect_to_login(uri.as_str())
}

#[get("/login")]
fn login_redirect(_user: User) -> Redirect {
    Redirect::to("/")
}

#[derive(FromForm)]
struct LoginOptions {
    redirect: String,
}

#[get("/login?<options>")]
fn get_login(options: LoginOptions, error: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    context.insert("redirect", options.redirect.as_str());
    if let Some(ref error) = error {
        context.insert("error", error.msg());
    }

    Template::render("login", &context)
}

#[derive(FromForm)]
struct LoginForm {
    username: String,
    password: String,
    redirect: String,
}

#[post("/login", data = "<login>")]
fn post_login(mut cookies: Cookies, login: Form<LoginForm>, site_admins: State<SiteAdmins>, conn: db::Conn) -> errors::Result<Result<Redirect, Flash<Redirect>>> {
    let login = login.into_inner();
    let redirect = URI::percent_decode_lossy(login.redirect.as_bytes());

    let mut user = User {
        site_admin: site_admins.0.contains(&login.username),
        name: login.username,
        tutor_years: HashSet::new(),
        admin_years: HashSet::new(),
    };

    if !user.site_admin {
        db::tutors::table
            .filter(db::tutors::username.eq(&user.name))
            .load::<db::Tutor>(&*conn)?
            .iter()
            .for_each(|tutor| {
                user.tutor_years.insert(tutor.year);
                if tutor.is_admin {
                    user.admin_years.insert(tutor.year);
                }
            });
    }

    if !user.site_admin && user.tutor_years.is_empty() {
        let msg = "Ungültiger Benutzername!";
        return Ok(Err(Flash::error(redirect_to_login(&redirect), msg)))
    }

    let result = user::authenticate(&user.name, &login.password);
    if result == Ok(true) {
        let user = serde_json::to_string(&user)?;
        cookies.add_private(Cookie::new("user", user));
        Ok(Ok(Redirect::to(&redirect)))
    } else {
        let msg = "Ungültiger Benutzername oder Passwort!";
        Ok(Err(Flash::error(redirect_to_login(&redirect), msg)))
    }
}

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user"));
    Redirect::to("/")
}

fn redirect_to_login(sucess_redirect: &str) -> Redirect {
    let success_redirect = URI::percent_encode(sucess_redirect)
        .replace("&", "%26"); // '&' is not encoded by default like '?'
    Redirect::to(&format!("/login?redirect={}", success_redirect))
}
