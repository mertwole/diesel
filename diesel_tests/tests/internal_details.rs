#![cfg(not(feature = "mysql"))]
use crate::schema::*;
use diesel::dsl::sql;
use diesel::sql_types::*;
use diesel::*;

#[diesel_test_helper::test]
#[cfg(not(feature = "mysql"))] // ? IS NULL is invalid syntax for MySQL
fn bind_params_are_passed_for_null_when_not_inserting() {
    let connection = &mut connection();
    let query =
        select(sql::<Integer>("1")).filter(None::<i32>.into_sql::<Nullable<Integer>>().is_null());
    assert_eq!(Ok(1), query.first(connection));
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn query_which_cannot_be_transmitted_gives_proper_error_message() {
    use crate::schema::comments::dsl::*;
    use diesel::result::DatabaseErrorKind::UnableToSendCommand;
    use diesel::result::Error::DatabaseError;

    // Create a query with 90000 binds, 2 binds per row
    let data: &[NewComment<'static>] = &[NewComment(1, "hi"); 45_000];
    let query_with_to_many_binds = insert_into(comments).values(data);

    match query_with_to_many_binds.execute(&mut connection()) {
        Ok(_) => panic!(
            "We successfully executed a query with 90k binds. \
             We need to find a new query to test which can't be represented by \
             the wire protocol."
        ),
        Err(DatabaseError(UnableToSendCommand, info)) => assert_ne!("", info.message()),
        Err(_) => panic!("We got back the wrong kind of error. This test is invalid."),
    }
}

#[diesel_test_helper::test]
#[cfg(feature = "postgres")]
fn empty_query_gives_proper_error_instead_of_panicking() {
    use diesel::dsl::sql;
    use diesel::result::DatabaseErrorKind::Unknown;
    use diesel::result::Error::DatabaseError;

    let connection = &mut connection();
    let query = sql::<Integer>("");

    match query.execute(connection) {
        Ok(_) => panic!("We successfully executed an empty query"),
        Err(DatabaseError(Unknown, info)) => assert_ne!("", info.message()),
        Err(_) => panic!("We got back the wrong kind of error. This test is invalid."),
    }
}

#[diesel_test_helper::test]
fn ensure_sqlite_does_not_access_dropped_buffers() {
    use diesel::connection::{DefaultLoadingMode, LoadConnection};
    let connection = &mut connection();

    let buf: Vec<u8> = vec![0, 1, 2];

    let query = diesel::select((&buf as &[u8]).into_sql::<sql_types::Binary>());

    let mut iter = LoadConnection::<DefaultLoadingMode>::load(connection, query).unwrap();

    assert!(iter.next().is_some());
    assert!(iter.next().is_none());
}
