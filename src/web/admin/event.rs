use crate::db;
use crate::errors::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Context {
    pub base: super::BaseContext,
    pub days: Vec<Day>,
}

#[derive(Serialize)]
pub struct Day {
    pub id: i32,
    pub name: String,
    pub experiments: Vec<Experiment>,
}

#[derive(Serialize)]
pub struct Experiment {
    pub id: i32,
    pub name: String,
    pub date: Option<String>,
}

pub fn load_days(year: i16, conn: &PgConnection) -> Result<Vec<Day>> {
    let experiments = db::experiments::table
        .filter(db::experiments::year.eq(year))
        .order(db::experiments::name.asc())
        .load::<db::Experiment>(conn)?;

    let days = db::days::table
        .filter(db::days::year.eq(year))
        .order(db::days::name.asc())
        .load::<db::Day>(conn)?;

    let events = db::Event::belonging_to(&days)
        .load::<db::Event>(conn)?
        .grouped_by(&days);

    Ok(days.into_iter()
        .zip(events)
        .map(|(day, events)| {
            let events: HashMap<_,_> = events.into_iter()
                .map(|event| (event.experiment_id, event.date))
                .collect();

            let experiments = experiments.iter()
                .map(|experiment| Experiment {
                    id: experiment.id,
                    name: experiment.name.clone(),
                    date: events.get(&experiment.id)
                        .map(|date| date.format("%Y-%m-%d").to_string()),
                })
                .collect();

            Day {
                id: day.id,
                name: day.name,
                experiments: experiments,
            }
        })
        .collect())
}
