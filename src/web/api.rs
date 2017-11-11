use db;
use diesel;
use diesel::pg::upsert::*;
use diesel::prelude::*;
use errors::*;
use rocket::response::status::NoContent;
use rocket_contrib::Json;
use web::session::User;

enum AuditContext {
    Group(i32),
    Year(i16),
}

fn add_audit_log(context: AuditContext, author: &str, conn: &PgConnection, change: &str) -> Result<()> {
    let (year, group) = match context {
        AuditContext::Group(group) => {
            let year = db::groups::table
                .inner_join(db::days::table)
                .filter(db::groups::id.eq(group))
                .select(db::days::year)
                .get_result(conn)?;
            (year, Some(group))
        },
        AuditContext::Year(year) => (year, None),
    };

    let log = db::NewAuditLog {
        year: year,
        author: author,
        affected_group: group,
        change: change,
    };

    let inserted = diesel::insert(&log)
        .into(db::audit_logs::table)
        .execute(conn)?;

    if inserted != 1 {
        Err("Could not insert audit log".into())
    } else {
        Ok(())
    }
}

#[post("/group", data = "<group>")]
fn post_group(group: Json<db::NewGroup>, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let id: i32 = diesel::insert(&*group)
            .into(db::groups::table)
            .returning(db::groups::id)
            .get_result(&*conn)?;

        let day_name: String = db::days::table.find(group.day_id)
            .select(db::days::name).get_result(&*conn)?;
        add_audit_log(AuditContext::Group(id), &user.name, &*conn,
            &format!("Create new group at desk {} on {} (#{}) with comment '{}'",
                group.desk, day_name, group.day_id, group.comment))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/completed/<task>")]
fn put_completion(group: i32, task: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    let completion = db::Completion {
        group_id: group,
        task_id: task,
    };

    conn.transaction(|| {
        diesel::insert(&completion.on_conflict_do_nothing())
            .into(db::completions::table)
            .execute(&*conn)?;

        let (experiment_name, task_name) = db::tasks::table.find(task)
            .inner_join(db::experiments::table)
            .select((db::experiments::name, db::tasks::name))
            .get_result::<(String, String)>(&*conn)?;
        add_audit_log(AuditContext::Group(group), &user.name, &*conn,
            &format!("Mark task {} (#{}) of {} as completed",
                task_name, task, experiment_name))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/completed/<task>")]
fn delete_completion(group: i32, task: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        diesel::delete(db::completions::table
            .filter(db::completions::group_id.eq(group))
            .filter(db::completions::task_id.eq(task)))
            .execute(&*conn)?;

        let (experiment_name, task_name) = db::tasks::table.find(task)
            .inner_join(db::experiments::table)
            .select((db::experiments::name, db::tasks::name))
            .get_result::<(String, String)>(&*conn)?;
        add_audit_log(AuditContext::Group(group), &user.name, &*conn,
            &format!("Unmark task {} (#{}) of {} as completed",
                task_name, task, experiment_name))?;

        Ok(NoContent)
    })
}

#[derive(Deserialize)]
struct Elaboration {
    rework_required: bool,
    accepted: bool,
}

#[put("/group/<group>/elaboration/<experiment>", data = "<elaboration>")]
fn put_elaboration(group: i32, experiment: i32, elaboration: Json<Elaboration>, conn: db::Conn, user: User) -> Result<NoContent> {
    let elaboration = db::Elaboration {
        group_id: group,
        experiment_id: experiment,
        rework_required: elaboration.rework_required,
        accepted: elaboration.accepted,
    };

    conn.transaction(|| {
        diesel::insert(
            &elaboration.on_conflict(
                (db::elaborations::group_id, db::elaborations::experiment_id),
                do_update().set(&elaboration)
            )
        ).into(db::elaborations::table).execute(&*conn)?;

        let status = match (elaboration.rework_required, elaboration.accepted) {
            (false, false) => "submitted",
            (false,  true) => "accepted",
            ( true, false) => "needing rework",
            ( true,  true) => "rework accepted",
        };
        let experiment_name: String = db::experiments::table.find(experiment)
            .select(db::experiments::name).get_result(&*conn)?;
        add_audit_log(AuditContext::Group(group), &user.name, &*conn,
            &format!("Mark elaboration of {} (#{}) as {}",
                experiment_name, experiment, status))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/elaboration/<experiment>")]
fn delete_elaboration(group: i32, experiment: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        diesel::delete(db::elaborations::table
            .filter(db::elaborations::group_id.eq(group))
            .filter(db::elaborations::experiment_id.eq(experiment)))
            .execute(&*conn)?;

        let experiment_name: String = db::experiments::table.find(experiment)
            .select(db::experiments::name).get_result(&*conn)?;
        add_audit_log(AuditContext::Group(group), &user.name, &*conn,
            &format!("Mark elaboration of {} (#{}) as missing",
                experiment_name, experiment))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/comment", data = "<comment>")]
fn put_group_comment(group: i32, comment: Json<String>, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let comment = comment.into_inner();
        diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
            .set(db::groups::comment.eq(&comment))
            .execute(&*conn)?;

        add_audit_log(AuditContext::Group(group), &user.name, &*conn,
            &format!("Change comment to '{}'", comment))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/desk", data = "<desk>")]
fn put_group_desk(group: i32, desk: Json<i32>, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let desk = desk.into_inner();
        diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
            .set(db::groups::desk.eq(desk))
            .execute(&*conn)?;

        add_audit_log(AuditContext::Group(group), &user.name, &*conn,
            &format!("Change desk to {}", desk))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/student/<student>")]
fn put_group_student(group: i32, student: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    let mapping = db::GroupMapping {
        student_id: student,
        group_id: group,
    };

    conn.transaction(|| {
        diesel::insert(&mapping)
            .into(db::group_mappings::table)
            .execute(&*conn)?;

        let student_name: String = db::students::table.find(student)
            .select(db::students::name).get_result(&*conn)?;
        add_audit_log(AuditContext::Group(group), &user.name, &*conn,
            &format!("Add {} (#{}) to group", student_name, student))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/student/<student>")]
fn delete_group_student(group: i32, student: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        diesel::delete(db::group_mappings::table
            .filter(db::group_mappings::student_id.eq(student))
            .filter(db::group_mappings::group_id.eq(group)))
            .execute(&*conn)?;

        let student_name: String = db::students::table.find(student)
            .select(db::students::name).get_result(&*conn)?;
        add_audit_log(AuditContext::Group(group), &user.name, &*conn,
            &format!("Remove {} (#{}) from group", student_name, student))?;

        Ok(NoContent)
    })
}

#[derive(Deserialize)]
struct Search {
    terms: Vec<String>,
    year: i16,
}

#[post("/group/search", data = "<search>")]
fn search_groups(search: Json<Search>, conn: db::Conn, _user: User) -> Result<Json<Vec<super::models::SearchGroup>>> {
    let groups = super::models::find_groups(&search.terms, search.year, &conn)?;

    Ok(Json(groups))
}

#[post("/student/search", data = "<search>")]
fn search_students(search: Json<Search>, conn: db::Conn, _user: User) -> Result<Json<Vec<super::models::Student>>> {
    let students = super::models::find_students(&search.terms, search.year, &conn)?;

    Ok(Json(students))
}
