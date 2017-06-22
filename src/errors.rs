error_chain!{
    foreign_links {
        Db(::diesel::result::Error);
    }
}
