use chrono::NaiveDate;
use csv::ReaderBuilder;
use db;
use diesel;
use diesel::prelude::*;
use errors::{ApiError, ApiResult, ResultExt};
use rocket::Data;
use rocket::response::status::NoContent;
use rocket_contrib::Json;
use web::models::find_writable_year;
use web::session::{SiteAdmin, User};

fn add_audit_log(year: i16, group: Option<i32>, author: &str, conn: &PgConnection, change: &str) -> ApiResult<()> {
    let log = db::NewAuditLog {
        year: year,
        author: author,
        affected_group: group,
        change: change,
    };

    diesel::insert_into(db::audit_logs::table)
        .values(&log)
        .execute(conn)
        .and_then(db::expect1)?;

    Ok(())
}

#[post("/group", data = "<group>")]
fn post_group(group: Json<db::NewGroup>, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let id: i32 = diesel::insert_into(db::groups::table)
            .values(&*group)
            .returning(db::groups::id)
            .get_result(&*conn)?;

        let year = find_writable_year(id, &*conn)?;
        user.ensure_tutor_for(year)?;

        let day_name: String = db::days::table.find(group.day_id)
            .select(db::days::name).get_result(&*conn)?;
        add_audit_log(year, Some(id), user.name(), &*conn,
            &format!("Create new group at desk {} on {} (#{}) with comment '{}'",
                group.desk, day_name, group.day_id, group.comment))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/completed/<task>")]
fn put_completion(group: i32, task: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    let completion = db::Completion {
        group_id: group,
        task_id: task,
    };

    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;
        user.ensure_tutor_for(year)?;

        diesel::insert_into(db::completions::table)
            .values(&completion)
            .on_conflict_do_nothing()
            .execute(&*conn)?;

        let (experiment_name, task_name) = db::tasks::table.find(task)
            .inner_join(db::experiments::table)
            .select((db::experiments::name, db::tasks::name))
            .get_result::<(String, String)>(&*conn)?;
        add_audit_log(year, Some(group), user.name(), &*conn,
            &format!("Mark task {} (#{}) of {} as completed",
                task_name, task, experiment_name))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/completed/<task>")]
fn delete_completion(group: i32, task: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;
        user.ensure_tutor_for(year)?;

        diesel::delete(db::completions::table
            .filter(db::completions::group_id.eq(group))
            .filter(db::completions::task_id.eq(task)))
            .execute(&*conn)?;

        let (experiment_name, task_name) = db::tasks::table.find(task)
            .inner_join(db::experiments::table)
            .select((db::experiments::name, db::tasks::name))
            .get_result::<(String, String)>(&*conn)?;
        add_audit_log(year, Some(group), user.name(), &*conn,
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
fn put_elaboration(group: i32, experiment: i32, elaboration: Json<Elaboration>, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    let elaboration = db::Elaboration {
        group_id: group,
        experiment_id: experiment,
        rework_required: elaboration.rework_required,
        accepted: elaboration.accepted,
    };

    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;
        user.ensure_tutor_for(year)?;

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
        add_audit_log(year, Some(group), user.name(), &*conn,
            &format!("Mark elaboration of {} (#{}) as {}",
                experiment_name, experiment, status))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/elaboration/<experiment>")]
fn delete_elaboration(group: i32, experiment: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;
        user.ensure_tutor_for(year)?;

        diesel::delete(db::elaborations::table
            .filter(db::elaborations::group_id.eq(group))
            .filter(db::elaborations::experiment_id.eq(experiment)))
            .execute(&*conn)?;

        let experiment_name: String = db::experiments::table.find(experiment)
            .select(db::experiments::name).get_result(&*conn)?;
        add_audit_log(year, Some(group), user.name(), &*conn,
            &format!("Mark elaboration of {} (#{}) as missing",
                experiment_name, experiment))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/comment", data = "<comment>")]
fn put_group_comment(group: i32, comment: Json<String>, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;
        user.ensure_tutor_for(year)?;

        let comment = comment.into_inner();
        diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
            .set(db::groups::comment.eq(&comment))
            .execute(&*conn)?;

        add_audit_log(year, Some(group), user.name(), &*conn,
            &format!("Change comment to '{}'", comment))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/desk", data = "<desk>")]
fn put_group_desk(group: i32, desk: Json<i32>, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;
        user.ensure_tutor_for(year)?;

        let desk = desk.into_inner();
        diesel::update(db::groups::table.filter(db::groups::id.eq(group)))
            .set(db::groups::desk.eq(desk))
            .execute(&*conn)?;

        add_audit_log(year, Some(group), user.name(), &*conn,
            &format!("Change desk to {}", desk))?;

        Ok(NoContent)
    })
}

#[put("/group/<group>/student/<student>")]
fn put_group_student(group: i32, student: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    let mapping = db::GroupMapping {
        student_id: student,
        group_id: group,
    };

    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;
        user.ensure_tutor_for(year)?;

        diesel::insert_into(db::group_mappings::table)
            .values(&mapping)
            .execute(&*conn)?;

        let student_name: String = db::students::table.find(student)
            .select(db::students::name).get_result(&*conn)?;
        add_audit_log(year, Some(group), user.name(), &*conn,
            &format!("Add {} (#{}) to group", student_name, student))?;

        Ok(NoContent)
    })
}

#[delete("/group/<group>/student/<student>")]
fn delete_group_student(group: i32, student: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let year = find_writable_year(group, &*conn)?;
        user.ensure_tutor_for(year)?;

        let num_completions: i64 = db::completions::table
            .filter(db::completions::group_id.eq(group))
            .count().get_result(&*conn)?;
        let num_elaborations: i64 = db::elaborations::table
            .filter(db::elaborations::group_id.eq(group))
            .count().get_result(&*conn)?;

        if num_completions + num_elaborations > 0 {
            return Err(ApiError::ConstraintViolation);
        }

        diesel::delete(db::group_mappings::table
            .filter(db::group_mappings::student_id.eq(student))
            .filter(db::group_mappings::group_id.eq(group)))
            .execute(&*conn)
            .and_then(db::expect1)?;

        let student_name: String = db::students::table.find(student)
            .select(db::students::name).get_result(&*conn)?;
        add_audit_log(year, Some(group), user.name(), &*conn,
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
fn search_groups(search: Json<Search>, conn: db::Conn, user: User) -> ApiResult<Json<Vec<super::models::SearchGroup>>> {
    user.ensure_tutor_for(search.year)?;

    let groups = super::models::find_groups(&search.terms, search.year, &conn)?;

    Ok(Json(groups))
}

#[post("/student/search", data = "<search>")]
fn search_students(search: Json<Search>, conn: db::Conn, user: User) -> ApiResult<Json<Vec<super::models::Student>>> {
    user.ensure_tutor_for(search.year)?;

    let students = super::models::find_students(&search.terms, search.year, &conn)?;

    Ok(Json(students))
}

#[put("/year/<year>")]
fn put_year(year: i16, conn: db::Conn, user: SiteAdmin) -> ApiResult<NoContent> {
    let db_year = db::Year {
        id: year,
        writable: true,
    };

    conn.transaction(|| {
        diesel::insert_into(db::years::table)
            .values(&db_year)
            .execute(&*conn)?;

        add_audit_log(year, None, user.name(), &conn,
            &format!("Create new year {}", year))?;

        Ok(NoContent)
    })
}

#[put("/year/<year>/closed")]
fn put_year_writable(year: i16, conn: db::Conn, user: SiteAdmin) -> ApiResult<NoContent> {
    conn.transaction(|| {
        diesel::update(db::years::table.filter(db::years::id.eq(year)))
            .set(db::years::writable.eq(false))
            .execute(&*conn)?;

        add_audit_log(year, None, user.name(), &conn,
            &format!("Close year {} (no longer modifiable)", year))?;

        Ok(NoContent)
    })
}

#[post("/experiment", data = "<experiment>")]
fn post_experiment(experiment: Json<db::NewExperiment>, conn: db::Conn, user: User) -> ApiResult<Json<i32>> {
    conn.transaction(|| {
        user.ensure_admin_for(experiment.year)?;

        let id: i32 = diesel::insert_into(db::experiments::table)
            .values(&*experiment)
            .returning(db::experiments::id)
            .get_result(&*conn)?;

        add_audit_log(experiment.year, None, user.name(), &conn,
            &format!("Create new experiment {} (#{})", experiment.name, id))?;

        Ok(Json(id))
    })
}

#[delete("/experiment/<experiment>")]
fn delete_experiment(experiment: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let full_experiment = db::experiments::table
            .find(experiment)
            .get_result::<db::Experiment>(&*conn)?;
        user.ensure_admin_for(full_experiment.year)?;

        diesel::delete(
            db::experiments::table.find(experiment))
            .execute(&*conn)
            .and_then(db::expect1)?;

        add_audit_log(full_experiment.year, None, user.name(), &conn,
            &format!("Remove experiment {} (#{})", full_experiment.name, experiment))?;

        Ok(NoContent)
    })
}

#[post("/experiment/<experiment>/task", data = "<task>")]
fn post_experiment_task(experiment: i32, task: Json<String>, conn: db::Conn, user: User) -> ApiResult<Json<i32>> {
    conn.transaction(|| {
        let full_experiment = db::experiments::table
            .find(experiment)
            .get_result::<db::Experiment>(&*conn)?;
        user.ensure_admin_for(full_experiment.year)?;

        let id: i32 = diesel::insert_into(db::tasks::table)
            .values((
                db::tasks::experiment_id.eq(experiment),
                db::tasks::name.eq(&*task),
            ))
            .returning(db::tasks::id)
            .get_result(&*conn)?;

        add_audit_log(full_experiment.year, None, user.name(), &conn,
            &format!("Create task {} (#{}) for experiment {} (#{})",
                *task, id, full_experiment.name, experiment))?;

        Ok(Json(id))
    })
}

#[delete("/experiment/<experiment>/task/<task>")]
fn delete_experiment_task(experiment: i32, task: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
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
        user.ensure_admin_for(year)?;

        diesel::delete(
            db::tasks::table.find(task))
            .execute(&*conn)
            .and_then(db::expect1)?;

        add_audit_log(year, None, user.name(), &conn,
            &format!("Remove task {} (#{}) from experiment {} (#{})",
                task_name, task, experiment_name, experiment))?;

        Ok(NoContent)
    })
}

#[put("/experiment/<experiment>/day/<day>/event", data = "<date>")]
fn put_event(experiment: i32, day: i32, date: Json<String>, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    let date: NaiveDate = date.parse().chain_err(|| "Invalid date")?;

    conn.transaction(|| {
        let record = db::Event {
            day_id: day,
            experiment_id: experiment,
            date: date,
        };

        diesel::insert_into(db::events::table)
            .values(&record)
            .on_conflict((db::events::day_id, db::events::experiment_id))
                .do_update()
                .set(&record)
            .execute(&*conn)
            .and_then(db::expect1)?;

        let experiment_name = db::experiments::table
            .find(experiment)
            .select(db::experiments::name)
            .get_result::<String>(&*conn)?;
        let (year, day_name) = db::days::table
            .find(day)
            .select((db::days::year, db::days::name))
            .get_result::<(i16, String)>(&*conn)?;

        user.ensure_admin_for(year)?;
        add_audit_log(year, None, user.name(), &conn,
            &format!("Set event date to {} for day {} (#{}) and experiment {} (#{})",
                date, day_name, day, experiment_name, experiment))?;

        Ok(NoContent)
    })
}

#[delete("/experiment/<experiment>/day/<day>/event")]
fn delete_event(experiment: i32, day: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        diesel::delete(db::events::table
            .find((day, experiment))) // beware the order of the columns!
            .execute(&*conn)
            .and_then(db::expect1)?;

        let experiment_name = db::experiments::table
            .find(experiment)
            .select(db::experiments::name)
            .get_result::<String>(&*conn)?;
        let (year, day_name) = db::days::table
            .find(day)
            .select((db::days::year, db::days::name))
            .get_result::<(i16, String)>(&*conn)?;

        user.ensure_admin_for(year)?;
        add_audit_log(year, None, user.name(), &conn,
            &format!("Remove event date for day {} (#{}) and experiment {} (#{})",
                day_name, day, experiment_name, experiment))?;

        Ok(NoContent)
    })
}

#[post("/day", data = "<day>")]
fn post_day(day: Json<db::NewDay>, conn: db::Conn, user: User) -> ApiResult<Json<i32>> {
    conn.transaction(|| {
        user.ensure_admin_for(day.year)?;

        let id: i32 = diesel::insert_into(db::days::table)
            .values(&*day)
            .returning(db::days::id)
            .get_result(&*conn)?;

        add_audit_log(day.year, None, user.name(), &conn,
            &format!("Create new day {} (#{})", day.name, id))?;

        Ok(Json(id))
    })
}

#[delete("/day/<day>")]
fn delete_day(day: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let full_day = db::days::table
            .find(day)
            .get_result::<db::Day>(&*conn)?;
        user.ensure_admin_for(full_day.year)?;

        diesel::delete(
            db::days::table.find(day))
            .execute(&*conn)
            .and_then(db::expect1)?;

        add_audit_log(full_day.year, None, user.name(), &conn,
            &format!("Remove day {} (#{})", full_day.name, day))?;

        Ok(NoContent)
    })
}

// Insert single student and audit log without a transaction
fn insert_student(student: &db::NewStudent, conn: &PgConnection, user: &str) -> ApiResult<i32> {
    let id = diesel::insert_into(db::students::table)
        .values(student)
        .returning(db::students::id)
        .get_result(&*conn)?;

    add_audit_log(student.year, None, user, conn,
        &format!("Create new student {} ({}, {}, #{})",
            student.name, student.matrikel,
            student.username.as_ref().map_or("-", |s| s), id))?;

    Ok(id)
}

#[post("/student", data = "<student>")]
fn post_student(student: Json<db::NewStudent>, conn: db::Conn, user: User) -> ApiResult<Json<i32>> {
    conn.transaction(|| {
        user.ensure_admin_for(student.year)?;

        Ok(Json(insert_student(&*student, &conn, user.name())?))
    })
}

#[post("/students/<year>", format = "text/csv", data = "<students>")]
fn post_students_csv(year: i16, students: Data, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    #[derive(Debug, Deserialize)]
    struct Student {
        matrikel: String,
        name: String,
        username: Option<String>,
    }

    user.ensure_admin_for(year)?;

    let mut csv_reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(students.open());

    conn.transaction(|| {
        for student in csv_reader.deserialize() {
            let student: Student = student?;
            let student = db::NewStudent {
                matrikel: student.matrikel,
                name: student.name,
                year: year,
                username: student.username,
            };

            insert_student(&student, &conn, user.name())?;
        }

        Ok(NoContent)
    })
}

#[delete("/student/<student>")]
fn delete_student(student: i32, conn: db::Conn, user: User) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let full_student = db::students::table
            .find(student)
            .get_result::<db::Student>(&*conn)?;
        user.ensure_admin_for(full_student.year)?;

        diesel::delete(
            db::students::table.find(student))
            .execute(&*conn)
            .and_then(db::expect1)?;

        add_audit_log(full_student.year, None, user.name(), &conn,
            &format!("Remove student {} ({}, {}, #{})",
                full_student.name, full_student.matrikel,
                full_student.username.as_ref().map_or("-", |s| s), student))?;

        Ok(NoContent)
    })
}

#[post("/tutor", data = "<tutor>")]
fn post_tutor(tutor: Json<db::NewTutor>, conn: db::Conn, user: SiteAdmin) -> ApiResult<Json<i32>> {
    conn.transaction(|| {
        let id = diesel::insert_into(db::tutors::table)
            .values(&*tutor)
            .returning(db::tutors::id)
            .get_result(&*conn)?;

        add_audit_log(tutor.year, None, user.name(), &conn,
            &format!("Create new tutor {} (#{}, {})", tutor.username,
            id, if tutor.is_admin { "admin" } else { "no admin" }))?;

        Ok(Json(id))
    })
}

#[delete("/tutor/<tutor>")]
fn delete_tutor(tutor: i32, conn: db::Conn, user: SiteAdmin) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let full_tutor = db::tutors::table
            .find(tutor)
            .get_result::<db::Tutor>(&*conn)?;

        diesel::delete(
            db::tutors::table.find(tutor))
            .execute(&*conn)
            .and_then(db::expect1)?;

        add_audit_log(full_tutor.year, None, user.name(), &conn,
            &format!("Remove tutor {} (#{}, {})", full_tutor.username,
            tutor, if full_tutor.is_admin { "admin" } else { "no admin" }))?;

        Ok(NoContent)
    })
}

#[put("/tutor/<tutor>/is_admin", data = "<is_admin>")]
fn put_tutor_admin(tutor: i32, is_admin: Json<bool>, conn: db::Conn, user: SiteAdmin) -> ApiResult<NoContent> {
    conn.transaction(|| {
        let full_tutor = db::tutors::table
            .find(tutor)
            .get_result::<db::Tutor>(&*conn)?;

        diesel::update(db::tutors::table.find(tutor))
            .set(db::tutors::is_admin.eq(*is_admin))
            .execute(&*conn)
            .and_then(db::expect1)?;

        add_audit_log(full_tutor.year, None, user.name(), &conn,
            &format!("Tutor {} (#{}) is {} admin", full_tutor.username,
            tutor, if *is_admin { "now" } else { "no longer" }))?;

        Ok(NoContent)
    })
}
