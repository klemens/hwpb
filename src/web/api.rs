use db;
use diesel;
use diesel::prelude::*;
use errors::*;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket_contrib::Json;
use web::models::find_writable_year;
use web::session::User;

fn add_audit_log(year: i16, group: Option<i32>, author: &str, conn: &PgConnection, change: &str) -> Result<()> {
    let log = db::NewAuditLog {
        year: year,
        author: author,
        affected_group: group,
        change: change,
    };

    let inserted = diesel::insert_into(db::audit_logs::table)
        .values(&log)
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
        let id: i32 = diesel::insert_into(db::groups::table)
            .values(&*group)
            .returning(db::groups::id)
            .get_result(&*conn)?;

        let year = find_writable_year(id, &*conn)?;

        let day_name: String = db::days::table.find(group.day_id)
            .select(db::days::name).get_result(&*conn)?;
        add_audit_log(year, Some(id), &user.name, &*conn,
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
        let year = find_writable_year(group, &*conn)?;

        diesel::insert_into(db::completions::table)
            .values(&completion)
            .on_conflict_do_nothing()
            .execute(&*conn)?;

        let (experiment_name, task_name) = db::tasks::table.find(task)
            .inner_join(db::experiments::table)
            .select((db::experiments::name, db::tasks::name))
            .get_result::<(String, String)>(&*conn)?;
        add_audit_log(year, Some(group), &user.name, &*conn,
            &format!("Mark task {} (#{}) of {} as completed",
                task_name, task, experiment_name))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/completed/<task>")]
fn delete_completion(group: i32, task: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;

        diesel::delete(db::completions::table
            .filter(db::completions::group_id.eq(group))
            .filter(db::completions::task_id.eq(task)))
            .execute(&*conn)?;

        let (experiment_name, task_name) = db::tasks::table.find(task)
            .inner_join(db::experiments::table)
            .select((db::experiments::name, db::tasks::name))
            .get_result::<(String, String)>(&*conn)?;
        add_audit_log(year, Some(group), &user.name, &*conn,
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
        let year = find_writable_year(group, &*conn)?;

        diesel::insert_into(db::elaborations::table)
            .values(&elaboration)
            .on_conflict((db::elaborations::group_id, db::elaborations::experiment_id))
                .do_update()
                .set(&elaboration)
            .execute(&*conn)?;

        let status = match (elaboration.rework_required, elaboration.accepted) {
            (false, false) => "submitted",
            (false,  true) => "accepted",
            ( true, false) => "needing rework",
            ( true,  true) => "rework accepted",
        };
        let experiment_name: String = db::experiments::table.find(experiment)
            .select(db::experiments::name).get_result(&*conn)?;
        add_audit_log(year, Some(group), &user.name, &*conn,
            &format!("Mark elaboration of {} (#{}) as {}",
                experiment_name, experiment, status))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/elaboration/<experiment>")]
fn delete_elaboration(group: i32, experiment: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;

        diesel::delete(db::elaborations::table
            .filter(db::elaborations::group_id.eq(group))
            .filter(db::elaborations::experiment_id.eq(experiment)))
            .execute(&*conn)?;

        let experiment_name: String = db::experiments::table.find(experiment)
            .select(db::experiments::name).get_result(&*conn)?;
        add_audit_log(year, Some(group), &user.name, &*conn,
            &format!("Mark elaboration of {} (#{}) as missing",
                experiment_name, experiment))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/comment", data = "<comment>")]
fn put_group_comment(group: i32, comment: Json<String>, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;

        let comment = comment.into_inner();
        diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
            .set(db::groups::comment.eq(&comment))
            .execute(&*conn)?;

        add_audit_log(year, Some(group), &user.name, &*conn,
            &format!("Change comment to '{}'", comment))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/desk", data = "<desk>")]
fn put_group_desk(group: i32, desk: Json<i32>, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;

        let desk = desk.into_inner();
        diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
            .set(db::groups::desk.eq(desk))
            .execute(&*conn)?;

        add_audit_log(year, Some(group), &user.name, &*conn,
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
        let year = find_writable_year(group, &*conn)?;

        diesel::insert_into(db::group_mappings::table)
            .values(&mapping)
            .execute(&*conn)?;

        let student_name: String = db::students::table.find(student)
            .select(db::students::name).get_result(&*conn)?;
        add_audit_log(year, Some(group), &user.name, &*conn,
            &format!("Add {} (#{}) to group", student_name, student))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/student/<student>")]
fn delete_group_student(group: i32, student: i32, conn: db::Conn, user: User) -> Result<Custom<()>> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;

        let num_completions: i64 = db::completions::table
            .filter(db::completions::group_id.eq(group))
            .count().get_result(&*conn)?;
        let num_elaborations: i64 = db::elaborations::table
            .filter(db::elaborations::group_id.eq(group))
            .count().get_result(&*conn)?;

        if num_completions + num_elaborations > 0 {
            // Return 423 Locked when deletion is not possible
            return Ok(Custom(Status::Locked, ()));
        }

        diesel::delete(db::group_mappings::table
            .filter(db::group_mappings::student_id.eq(student))
            .filter(db::group_mappings::group_id.eq(group)))
            .execute(&*conn)
            .and_then(db::expect1)?;

        let student_name: String = db::students::table.find(student)
            .select(db::students::name).get_result(&*conn)?;
        add_audit_log(year, Some(group), &user.name, &*conn,
            &format!("Remove {} (#{}) from group", student_name, student))?;

        Ok(Custom(Status::NoContent, ()))
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

#[put("/year/<year>")]
fn put_year(year: i16, conn: db::Conn, user: User) -> Result<NoContent> {
    let db_year = db::Year {
        id: year,
        writable: true,
    };

    conn.transaction(|| {
        diesel::insert_into(db::years::table)
            .values(&db_year)
            .execute(&*conn)?;

        add_audit_log(year, None, &user.name, &conn,
            &format!("Create new year {}", year))?;

        Ok(NoContent)
    })
}

#[put("/year/<year>/closed")]
fn put_year_writable(year: i16, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        diesel::update(db::years::table.filter(db::years::id.eq(year)))
            .set(db::years::writable.eq(false))
            .execute(&*conn)?;

        add_audit_log(year, None, &user.name, &conn,
            &format!("Close year {} (no longer modifiable)", year))?;

        Ok(NoContent)
    })
}

#[post("/experiment", data = "<experiment>")]
fn post_experiment(experiment: Json<db::NewExperiment>, conn: db::Conn, user: User) -> Result<Json<i32>> {
    conn.transaction(|| {
        let id: i32 = diesel::insert_into(db::experiments::table)
            .values(&*experiment)
            .returning(db::experiments::id)
            .get_result(&*conn)?;

        add_audit_log(experiment.year, None, &user.name, &conn,
            &format!("Create new experiment {} (#{})", experiment.name, id))?;

        Ok(Json(id))
    })
}

#[delete("/experiment/<experiment>")]
fn delete_experiment(experiment: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let full_experiment = db::experiments::table
            .find(experiment)
            .get_result::<db::Experiment>(&*conn)?;

        diesel::delete(
            db::experiments::table.find(experiment))
            .execute(&*conn)
            .and_then(db::expect1)?;

        add_audit_log(full_experiment.year, None, &user.name, &conn,
            &format!("Remove experiment {} (#{})", full_experiment.name, experiment))?;

        Ok(NoContent)
    })
}

#[post("/experiment/<experiment>/task", data = "<task>")]
fn post_experiment_task(experiment: i32, task: Json<String>, conn: db::Conn, user: User) -> Result<Json<i32>> {
    conn.transaction(|| {
        let full_experiment = db::experiments::table
            .find(experiment)
            .get_result::<db::Experiment>(&*conn)?;

        let id: i32 = diesel::insert_into(db::tasks::table)
            .values((
                db::tasks::experiment_id.eq(experiment),
                db::tasks::name.eq(&*task),
            ))
            .returning(db::tasks::id)
            .get_result(&*conn)?;

        add_audit_log(full_experiment.year, None, &user.name, &conn,
            &format!("Create task {} (#{}) for experiment {} (#{})",
                *task, id, full_experiment.name, experiment))?;

        Ok(Json(id))
    })
}

#[delete("/experiment/<experiment>/task/<task>")]
fn delete_experiment_task(experiment: i32, task: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    conn.transaction(|| {
        let (task_name, experiment_name, year) = db::tasks::table
            .inner_join(db::experiments::table)
            .filter(db::tasks::id.eq(task))
            .filter(db::experiments::id.eq(experiment))
            .select((
                db::tasks::name,
                db::experiments::name,
                db::experiments::year,
            ))
            .get_result::<(String, String, i16)>(&*conn)?;

        diesel::delete(
            db::tasks::table.find(task))
            .execute(&*conn)
            .and_then(db::expect1)?;

        add_audit_log(year, None, &user.name, &conn,
            &format!("Remove task {} (#{}) from experiment {} (#{})",
                task_name, task, experiment_name, experiment))?;

        Ok(NoContent)
    })
}
