use web::models;

#[derive(Serialize)]
pub struct Context {
    pub site: &'static str,
    pub year: i16,
    pub years: Vec<models::Year>,
}
