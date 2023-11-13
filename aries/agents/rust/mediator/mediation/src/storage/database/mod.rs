// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

// unimplemented!()
// #[cfg(any(
//     not(any(feature = "any_db", feature = "postgres_db", feature = "mysql_db")),
//     all(feature = "any_db", feature = "postgres_db", feature = "mysql_db"),
//     all(feature = "any_db", feature = "postgres_db"),
//     all(feature = "postgres_db", feature = "mysql_db"),
//     all(feature = "any_db", feature = "mysql_db")
// ))]
// compile_error!("Pick any one of \"any_db\", \"postgresql_db\", \"mysql_db\" feature flags.");

// #[cfg(feature = "any_db")]
// mod any;
// #[cfg(feature = "any_db")]
// pub use any::get_db_pool;

// #[cfg(feature = "postgres_db")]
// mod postgres;
// #[cfg(feature = "postgres_db")]
// pub use postgres::get_db_pool;

#[cfg(feature = "mysql_db")]
mod mysql;
#[cfg(feature = "mysql_db")]
pub use mysql::get_db_pool;
