use csv::Writer;
use db;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use errors::*;
use rocket::http::ContentType;
use rocket::http::hyper::header;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use std::collections::HashMap;
use std::io::Cursor;
use web::analysis;

pub struct CsvResponse {
    pub filename: String,
    pub content: Vec<u8>,
}

impl<'r> Responder<'r> for CsvResponse {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let disposition = header::ContentDisposition {
            disposition: header::DispositionType::Attachment,
            parameters: vec![
                header::DispositionParam::Filename(
                    header::Charset::Ext("UTF-8".into()),
                    None,
                    self.filename.into_bytes(),
                )
            ]
        };

        Response::build()
            .header(ContentType::CSV)
            .header(disposition)
            .sized_body(Cursor::new(self.content))
            .ok()
    }
}

pub fn create_csv(year: i16, conn: &PgConnection) -> Result<Vec<u8>> {
    let mut csv = Writer::from_writer(vec![]);

    // Load everything inside a transaction to get a consistent view
    let (students, accepted_elaborations, completed_tasks, experiments, tasks)
        = conn.transaction(|| -> Result<_> {
        let students = db::students::table
            .filter(db::students::year.eq(year))
            .order(db::students::id)
            .load::<db::Student>(conn)?;

        let (accepted_elaborations, experiments) =
            analysis::load_elaborations_by_student(year, None, Some(true), conn)?;

        let (completed_tasks, tasks) =
            analysis::load_tasks_by_student(year, true, conn)?;

        Ok((students, accepted_elaborations, completed_tasks, experiments, tasks))
    })?;

    // These may not contain a record for every student, so we just use them
    // as lookup tables and iterate over all students directly
    let accepted_elaborations: HashMap<_,_> = accepted_elaborations.into_iter()
        .map(|(student, elaborations)| (student.id, elaborations))
        .collect();
    let completed_tasks: HashMap<_,_> = completed_tasks.into_iter()
        .map(|(student, tasks)| (student.id, tasks))
        .collect();

    // Map to lookup experiment names when writing task headers
    let experiment_names: HashMap<_,_> = experiments.iter()
        .map(|experiment| (experiment.id, experiment.name.as_str()))
        .collect();

    // Write csv header
    csv.write_field("Matrikelnummer")?;
    csv.write_field("Name")?;
    csv.write_field("Benutzername")?;
    csv.write_field("Sicherheitsbelehrung")?;
    for experiment in experiments.iter() {
        csv.write_field(format!("Ausarbeitung {}", experiment.name))?;
    }
    for task in tasks.iter() {
        let experiment = experiment_names.get(&task.experiment_id)
            .expect("experiment_names map should be complete");
        csv.write_field(format!("{}, {}", experiment, task.name))?;
    }
    // Complete the header
    csv.write_record(None::<&[u8]>)?;

    let display_bool = |value| if value {"x"} else {""};

    // Writer one record for every student
    for student in students {
        csv.write_field(student.matrikel)?;
        csv.write_field(student.name)?;
        csv.write_field(student.username.as_ref().map_or("", |s| s))?;
        csv.write_field(display_bool(student.instructed))?;

        // Write accepted elaborations
        if let Some(elaborations) = accepted_elaborations.get(&student.id) {
            for accepted in elaborations {
                csv.write_field(display_bool(accepted))?;
            }
        } else {
            // Student did not hand in any elaboration yet
            for _ in 0..experiments.len() {
                csv.write_field(display_bool(false))?;
            }
        }

        // Write completed tasks
        if let Some(tasks) = completed_tasks.get(&student.id) {
            for completed in tasks {
                csv.write_field(display_bool(completed))?;
            }
        } else {
            // Student did not complete any tasks yet
            for _ in 0..tasks.len() {
                csv.write_field(display_bool(false))?;
            }
        }

        // Complete the record
        csv.write_record(None::<&[u8]>)?;
    }

    csv.into_inner().chain_err(|| "Could not finalize csv writer")
}
