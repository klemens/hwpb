use crate::db;
use crate::errors::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;

#[derive(Serialize)]
pub struct Context {
    pub base: super::BaseContext,
    pub experiments: Vec<Experiment>,
}

#[derive(Serialize)]
pub struct Experiment {
    pub id: i32,
    pub name: String,
    pub tasks: Vec<Task>,
}

#[derive(Serialize)]
pub struct Task {
    pub id: i32,
    pub name: String,
}

pub fn load_experiments(year: i16, conn: &PgConnection) -> Result<Vec<Experiment>> {
    let experiments = db::experiments::table
        .filter(db::experiments::year.eq(year))
        .order(db::experiments::name.asc())
        .load::<db::Experiment>(conn)?;

    let tasks = db::Task::belonging_to(&experiments)
        .order(db::tasks::name.asc())
        .load::<db::Task>(conn)?
        .grouped_by(&experiments);

    Ok(experiments.into_iter()
        .zip(tasks)
        .map(|(experiment, tasks)| {
            let tasks = tasks.into_iter()
                .map(|task| Task {
                     id: task.id,
                     name: task.name,
                })
                .collect();

            Experiment {
                id: experiment.id,
                name: experiment.name,
                tasks: tasks,
            }
        })
        .collect())
}
