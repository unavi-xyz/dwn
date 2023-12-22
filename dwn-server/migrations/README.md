# Migrations

SQL migrations for the `dwn-server` MariaDB database.
A `run.sh` shell script is provided for ease of use.

## Flags

| Flag     | Default  | Description |
| -------- | -------- | ----------- |
| `--env`  | `.env`   | Pass in a file to load environment variables from. |
| `--drop` | disabled | Drop all tables in the database before running migrations. |

## Usage

### Local Development

For local development, you can run the following from the root of the repo:

```bash
sh ./dwn-server/migrations/run.sh --drop
```

This will clear the local database and run all migrations.

### Remote Database

If using a remote database, you can create a new file such as `.env.local` and store your `DATABASE_URL` within it.
The following command can then be used to connect to it and run migrations:

```bash
sh ./dwn-server/migrations/run.sh --env .env.local
```
