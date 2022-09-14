/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

#[cfg(any(test, feature = "pg_test"))]
#[pgx::pg_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as pgx_tests;

    use pgx::*;

    #[pg_test(error = "syntax error at or near \"THIS\"")]
    fn test_spi_failure() {
        Spi::execute(|client| {
            client.select("THIS IS NOT A VALID QUERY", None, None);
        });
    }

    #[pg_test]
    fn test_spi_can_nest() {
        Spi::execute(|_| {
            Spi::execute(|_| {
                Spi::execute(|_| {
                    Spi::execute(|_| {
                        Spi::execute(|_| {});
                    });
                });
            });
        });
    }

    #[pg_test]
    fn test_spi_returns_primitive() {
        let rc = Spi::connect(|client| {
            Ok(client
                .select("SELECT 42", None, None)
                .first()
                .get_datum::<i32>(1))
        });

        assert_eq!(42, rc.expect("SPI failed to return proper value"))
    }

    #[pg_test]
    fn test_spi_returns_str() {
        let rc = Spi::connect(|client| {
            Ok(client
                .select("SELECT 'this is a test'", None, None)
                .first()
                .get_datum::<&str>(1))
        });

        assert_eq!(
            "this is a test",
            rc.expect("SPI failed to return proper value")
        )
    }

    #[pg_test]
    fn test_spi_returns_string() {
        let rc = Spi::connect(|client| {
            Ok(client
                .select("SELECT 'this is a test'", None, None)
                .first()
                .get_datum::<String>(1))
        });

        assert_eq!(
            "this is a test",
            rc.expect("SPI failed to return proper value")
        )
    }

    #[pg_test]
    fn test_spi_get_one() {
        Spi::execute(|client| {
            let i = client
                .select("SELECT 42::bigint", None, None)
                .first()
                .get_one::<i64>();
            assert_eq!(42, i.unwrap());
        });
    }

    #[pg_test]
    fn test_spi_get_two() {
        Spi::execute(|client| {
            let (i, s) = client
                .select("SELECT 42, 'test'", None, None)
                .first()
                .get_two::<i64, &str>();

            assert_eq!(42, i.unwrap());
            assert_eq!("test", s.unwrap());
        });
    }

    #[pg_test]
    fn test_spi_get_three() {
        Spi::execute(|client| {
            let (i, s, b) = client
                .select("SELECT 42, 'test', true", None, None)
                .first()
                .get_three::<i64, &str, bool>();

            assert_eq!(42, i.unwrap());
            assert_eq!("test", s.unwrap());
            assert_eq!(true, b.unwrap());
        });
    }

    #[pg_test]
    fn test_spi_get_two_with_failure() {
        Spi::execute(|client| {
            let (i, s) = client
                .select("SELECT 42", None, None)
                .first()
                .get_two::<i64, &str>();

            assert_eq!(42, i.unwrap());
            assert!(s.is_none());
        });
    }

    #[pg_test]
    fn test_spi_get_three_failure() {
        Spi::execute(|client| {
            let (i, s, b) = client
                .select("SELECT 42, 'test'", None, None)
                .first()
                .get_three::<i64, &str, bool>();

            assert_eq!(42, i.unwrap());
            assert_eq!("test", s.unwrap());
            assert!(b.is_none());
        });
    }

    #[pg_test]
    fn test_spi_select_zero_rows() {
        assert!(Spi::get_one::<i32>("SELECT 1 LIMIT 0").is_none());
    }

    #[pg_test]
    fn test_spi_run() {
        Spi::run("SELECT 1")
    }

    #[pg_test]
    fn test_spi_run_with_args() {
        let i = 1 as i32;
        let j = 2 as i64;

        Spi::run_with_args(
            "SELECT $1 + $2 = 3",
            Some(vec![
                (PgBuiltInOids::INT4OID.oid(), i.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), j.into_datum()),
            ]),
        )
    }

    #[pg_test]
    fn test_spi_explain() {
        let result = Spi::explain("SELECT 1");
        let expected: serde_json::Value = serde_json::from_str(r#"
        [{"Plan": {"Node Type": "Result", "Parallel Aware": false, "Plan Rows": 1, "Plan Width": 4, "Startup Cost": 0.0, "Total Cost": 0.01}}]
        "#).unwrap();
        assert_eq!(result.0, expected);
    }

    #[pg_test]
    fn test_spi_explain_with_args() {
        let i = 1 as i32;
        let j = 2 as i64;

        let result = Spi::explain_with_args(
            "SELECT $1 + $2 = 3",
            Some(vec![
                (PgBuiltInOids::INT4OID.oid(), i.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), j.into_datum()),
            ]),
        );

        let expected: serde_json::Value = serde_json::from_str(r#"
        [{"Plan": {"Node Type": "Result", "Parallel Aware": false, "Plan Rows": 1, "Plan Width": 1, "Startup Cost": 0.0, "Total Cost": 0.01}}]
        "#).unwrap();
        assert_eq!(result.0, expected);
    }

    #[pg_extern]
    fn do_panic() {
        panic!("did a panic");
    }

    #[pg_test(error = "did a panic")]
    fn test_panic_via_spi() {
        Spi::run("SELECT tests.do_panic();");
    }

    #[pg_test]
    fn test_inserting_null() {
        Spi::execute(|mut client| {
            client.update("CREATE TABLE tests.null_test (id uuid)", None, None);
        });
        let result = Spi::get_one_with_args::<i32>(
            "INSERT INTO tests.null_test VALUES ($1) RETURNING 1",
            vec![(PgBuiltInOids::UUIDOID.oid(), None)],
        );
        assert_eq!(result, Some(1));
    }
}
