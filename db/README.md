# DB

For DB i've picked SQLite3, because it's easy to use. If I'd want to use something more powerful, I'd use PostgreSQL.

As a migrations tool I've picked FlywayDB.

## Pre-requisites

1. flyway cli - https://flywaydb.org/documentation/commandline/#download-and-installation
2. sqlite3 cli - https://sqlite.org/download.html

## How to run

1. After pre-requisites are installed, run `flyway migrate` to run migrations.
   That's all DB is ready to use. Just set `DATABASE_URL` env variable to `sqlite:db/ticker-server.db` and run server.
