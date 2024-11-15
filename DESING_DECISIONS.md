## Database testing

SeaORM provides a way to test the database with the `MockDatabase` interface.

<https://www.sea-ql.org/sea-orm-tutorial/ch01-07-mock-testing.html>

However, I find it more reliable to use test containers and create actual PostgreSQL instances for testing.