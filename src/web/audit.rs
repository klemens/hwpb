use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use rocket::response::Redirect;
use rocket_contrib::Template;
use web::session::User;

#[derive(Serialize)]
struct AuditLog {
    year: i16,
    time: String,
    time_short: String,
    author: String,
    group: Option<i32>,
    change: String,
}

#[derive(Serialize)]
struct AuditContext {
    logs: Vec<AuditLog>,
    filters: AuditFilters,
    years: Vec<i16>,
    authors: Vec<String>,
}

#[derive(FromForm, Serialize)]
struct AuditFilters {
    year: Option<i16>,
    search: Option<String>,
    group: Option<i32>,
    author: Option<String>,
}

#[get("/audit")]
fn audit_index(_user: User) -> Redirect {
    Redirect::to("/audit?")
}

#[get("/audit?<filters>")]
fn audit_logs(filters: AuditFilters, conn: db::Conn, _user: User) -> Result<Template> {
    let authors = db::audit_logs::table
        .select(db::audit_logs::author)
        .distinct()
        .order(db::audit_logs::author.asc())
        .load(&*conn)?;

    let context = AuditContext {
        logs: load_audit_logs(&filters, &*conn)?,
        filters: filters,
        years: super::models::find_years(&*conn)?,
        authors: authors,
    };

    Ok(Template::render("audit", context))
}

fn load_audit_logs(filters: &AuditFilters, conn: &PgConnection) -> Result<Vec<AuditLog>> {
    let mut query = db::audit_logs::table.into_boxed();

    if let Some(year) = filters.year {
        query = query.filter(db::audit_logs::year.eq(year));
    }
    if let Some(search) = filters.search.as_ref() {
        for term in search.split_whitespace() {
            query = query.filter(db::audit_logs::change.ilike(
                format!("%{}%", term)));
        }
    }
    if let Some(group) = filters.group {
        query = query.filter(db::audit_logs::affected_group.eq(group));
    }
    if let Some(author) = filters.author.as_ref() {
        if !author.is_empty() {
            query = query.filter(db::audit_logs::author.eq(author));
        }
    }

    Ok(query
        .order(db::audit_logs::created_at.desc())
        .load::<db::AuditLog>(conn)?
        .into_iter()
        .map(|log| {
            AuditLog {
                year: log.year,
                time: log.created_at.to_rfc3339(),
                time_short: log.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                author: log.author,
                group: log.affected_group,
                change: log.change,
            }
        })
        .collect())
}