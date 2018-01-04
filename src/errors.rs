error_chain!{
    foreign_links {
        Csv(::csv::Error);
        Db(::diesel::result::Error);
    }
}
