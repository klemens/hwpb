// inet functions and operators missing in diesel

#![allow(dead_code)]

use diesel::expression::{AsExpression, Expression};
use diesel::pg::Pg;
use diesel::sql_types::{Inet, Text};

sql_function!(fn abbrev(x: Inet) -> Text);
sql_function!(fn inet(x: Text) -> Inet);

diesel_infix_operator!(Contains, " >> ", backend: Pg);
diesel_infix_operator!(ContainsOrEqual, " >>= ", backend: Pg);

pub trait PgInetExpressionMethods: Expression<SqlType = Inet> + Sized {
    /// Creates a PostgreSQL `>>` expression.
    ///
    /// This operator returns whether an IP network contains another
    /// network (or address).
    fn contains<T>(self, other: T) -> Contains<Self, T::Expression>
    where T: AsExpression<Self::SqlType> {
        Contains::new(self, other.as_expression())
    }

    /// Creates a PostgreSQL `>>=` expression.
    ///
    /// This operator returns whether an IP network contains another
    /// network (or address) or is equal.
    fn contains_or_equals<T>(self, other: T) -> ContainsOrEqual<Self, T::Expression>
    where T: AsExpression<Self::SqlType> {
        ContainsOrEqual::new(self, other.as_expression())
    }

    /// Convert to a abbreviated display format.
    ///
    /// Hosts are displayed without the CIDR netmask.
    fn abbrev(self) -> abbrev::HelperType<Self> {
        abbrev(self)
    }
}

impl<T> PgInetExpressionMethods for T
where T: Expression<SqlType = Inet> {}
