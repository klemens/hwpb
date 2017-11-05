use db;
use diesel;
use diesel::pg::upsert::*;
use diesel::prelude::*;
use errors::*;
use rocket::response::status::NoContent;
use rocket_contrib::Json;
use web::session::User;

#[post("/group", data = "<group>")]
fn post_group(group: Json<db::NewGroup>, conn: db::Conn, _user: User) -> Result<NoContent> {
    diesel::insert(&*group)
        .into(db::groups::table)
        .execute(&*conn)?;

    Ok(NoContent)
}

#[put("/group/<group>/completed/<task>")]
fn put_completion(group: i32, task: i32, conn: db::Conn, user: User) -> Result<NoContent> {
    let completion = db::Completion {
        group_id: group,
        task_id: task,
        tutor: Some(user.name),
        completed_at: None,
    };

    diesel::insert(&completion.on_conflict_do_nothing())
        .into(db::completions::table)
        .execute(&*conn)?;

    Ok(NoContent)
}

#[delete("/group/<group>/completed/<task>")]
fn delete_completion(group: i32, task: i32, conn: db::Conn, _user: User) -> Result<NoContent> {
    diesel::delete(db::completions::table
        .filter(db::completions::group_id.eq(group))
        .filter(db::completions::task_id.eq(task)))
        .execute(&*conn)?;

    Ok(NoContent)
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
        accepted_by: Some(user.name),
    };

    diesel::insert(
        &elaboration.on_conflict(
            (db::elaborations::group_id, db::elaborations::experiment_id),
            do_update().set(&elaboration)
        )
    ).into(db::elaborations::table).execute(&*conn)?;

    Ok(NoContent)
}

#[delete("/group/<group>/elaboration/<experiment>")]
fn delete_elaboration(group: i32, experiment: i32, conn: db::Conn, _user: User) -> Result<NoContent> {
    diesel::delete(db::elaborations::table
        .filter(db::elaborations::group_id.eq(group))
        .filter(db::elaborations::experiment_id.eq(experiment)))
        .execute(&*conn)?;

    Ok(NoContent)
}

#[put("/group/<group>/comment", data = "<comment>")]
fn put_group_comment(group: i32, comment: Json<String>, conn: db::Conn, _user: User) -> Result<NoContent> {
    diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
        .set(db::groups::comment.eq(comment.into_inner()))
        .execute(&*conn)?;

    Ok(NoContent)
}

#[put("/group/<group>/desk", data = "<desk>")]
fn put_group_desk(group: i32, desk: Json<i32>, conn: db::Conn, _user: User) -> Result<NoContent> {
    diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
        .set(db::groups::desk.eq(desk.into_inner()))
        .execute(&*conn)?;

    Ok(NoContent)
}

#[put("/group/<group>/student/<student>")]
fn put_group_student(group: i32, student: i32, conn: db::Conn, _user: User) -> Result<NoContent> {
    let mapping = db::GroupMapping {
        student_id: student,
        group_id: group,
    };

    diesel::insert(&mapping)
        .into(db::group_mappings::table)
        .execute(&*conn)?;

    Ok(NoContent)
}

#[delete("/group/<group>/student/<student>")]
fn delete_group_student(group: i32, student: i32, conn: db::Conn, _user: User) -> Result<NoContent> {
    diesel::delete(db::group_mappings::table
        .filter(db::group_mappings::student_id.eq(student))
        .filter(db::group_mappings::group_id.eq(group)))
        .execute(&*conn)?;

    Ok(NoContent)
}

#[post("/group/search", data = "<terms>")]
fn search_groups(terms: Json<Vec<String>>, conn: db::Conn, _user: User) -> Result<Json<Vec<super::models::SearchGroup>>> {
    let groups = super::models::find_groups(&*terms, &conn)?;

    Ok(Json(groups))
}

#[post("/student/search", data = "<terms>")]
fn search_students(terms: Json<Vec<String>>, conn: db::Conn, _user: User) -> Result<Json<Vec<super::models::Student>>> {
    let students = super::models::find_students(&*terms, &conn)?;

    Ok(Json(students))
}
