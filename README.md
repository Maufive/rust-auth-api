## Connecting to pgAdmin in Docker

After running `docker compose up -d` navigate to localhost:5050 to login to the pgAdmin dashboard.

Username: admin@admin.com
Password: password123

---

Or if you have other credentials setup in your .env file, use those.
When logged in to pgAdmin, use this information to connect to the Postgres database running inside your Docker container:

```
name: container-postgresdb
host: host.docker.internal
database: postgres
user: postgres
password: admin
```
