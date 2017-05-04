pub mod models;
pub mod schema;

pub use models::*;
pub use schema::*;

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::collections::HashSet;

pub fn print_table(date: &NaiveDate, conn: &PgConnection) -> QueryResult<()> {
    let event: Event = match events::table.filter(events::date.eq(date)).first(conn) {
        Ok(event) => event,
        _ => {
            println!(" Kein Praktikum am {}!", date);
            return Ok(());
        }
    };

    let tasks = tasks::table.filter(tasks::experiment_id.eq(event.experiment_id))
        .order(tasks::name.asc()).load::<Task>(conn)?;
    let groups = groups_with_students(&event.day_id, conn)?;

    // belonging_to uses eq_any internally, but supports only one parent table
    let task_ids: Vec<_> = tasks.iter().map(Identifiable::id).collect();
    let group_ids: Vec<_> = groups.iter().map(|&(ref g, _)| g.id).collect();
    let completions = completions::table.filter(completions::task_id.eq_any(task_ids))
        .filter(completions::group_id.eq_any(group_ids)).load::<Completion>(conn)?;

    // build set with all groups that completed a task
    let completions: HashSet<_> = completions.into_iter()
        .map(|c| (c.group_id, c.task_id)).collect();

    // print task legent
    println!("                                       {}", tasks.join(" "));

    for (group, students) in groups {
        print!("{}, {:35} ", group.desk, students.join(", "));

        for task in &tasks {
            if completions.contains(&(group.id, task.id)) {
                print!("✓  ");
            } else {
                print!("   ");
            }
        }
        println!();
    }

    Ok(())
}

pub fn groups_with_students(day: &str, conn: &PgConnection) -> QueryResult<Vec<(Group, Vec<Student>)>> {
    let groups = groups::table.filter(groups::day_id.eq(day)).load::<Group>(conn)?;
    let students = Student::belonging_to(&groups).load::<Student>(conn)?.grouped_by(&groups);
    Ok(groups.into_iter().zip(students).collect())
}
