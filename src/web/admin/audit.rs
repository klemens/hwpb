use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use web::models;

#[derive(Serialize)]
pub struct Log {
    year: i16,
    time: String,
    time_short: String,
    author: String,
    group: Option<i32>,
    change: String,
}

#[derive(Serialize)]
pub struct Context {
    pub site: &'static str,
    pub year: i16,
    pub years: Vec<models::Year>,
    pub logs: Vec<Log>,
    pub filters: Filters,
    pub authors: Vec<String>,
}

#[derive(FromForm, Serialize)]
pub struct Filters {
    search: Option<String>,
    group: Option<i32>,
    author: Option<String>,
}

pub fn load_authors(conn: &PgConnection) -> Result<Vec<String>> {
    Ok(db::audit_logs::table
        .select(db::audit_logs::author)
        .distinct()
        .order(db::audit_logs::author.asc())
        .load(&*conn)?)
}

pub fn load_logs(year: i16, filters: &Filters, conn: &PgConnection) -> Result<Vec<Log>> {
    let mut query = db::audit_logs::table
        .filter(db::audit_logs::year.eq(year))
        .into_boxed();

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
            Log {
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