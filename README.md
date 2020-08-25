# qwest_r

Qwestr API Server, built with [Express](https://expressjs.com/) and [Prisma](https://www.prisma.io/).

## Deployment

1.  Install all required packages using `yarn install`

2.  Create a file in the `/prisma` dir called `dev.db`.

2.  Create a file in the `/prisma` dir called `.env` and `DATABASE_URL="file:./dev.db"`

4.  Migrate the database to the latest version using `yarn migrate:update`

5.  Run `yarn start` to start the application

## Migration

This application utilizes [Prisma-Migrate](https://www.prisma.io/docs/reference/tools-and-interfaces/prisma-migrate) to create and run its database migrations.

Once you've made your model changes in `/prisma/schema.prisma`, simply run `yarn migrate:create`.

A new migration folder/ files will be created.  You can then execute the migration using<b>
`yarn migrate:update` or `yarn migrate:update-verbose`.

Once you execute a migration, you will also have to generate new Prisma Client code by running<b>
`yarn generate`

## Available Scripts

In the project directory, you can run:

### `yarn start`

Runs the API at [http://localhost:3001](http://localhost:3001).

### `yarn generate`

Generates the latest Prisma Client module based on your schema.

### `yarn migrate:create`

Creates a new migration folder/ files under `/prisma/migrations` using<b>
`prisma migrate save --experimental`.

### `yarn migrate:update`

Migrates the database using<b>
`prisma migrate up --experimental`.

### `yarn migrate:update-verbose`

Migrates the database using<b>
`prisma migrate up --experimental --verbose`.

## License

Copyright © 2016-2020 Qwestr LLC. This source code is licensed under the MIT
license found in the [LICENSE.txt](https://github.com/Qwestr-API/Qwestr/blob/master/LICENSE.txt)
file. The documentation to the project is licensed under the
[CC BY-SA 4.0](http://creativecommons.org/licenses/by-sa/4.0/) license.

---
Made with ♥ by the [QwestrDevs](https://github.com/Qwestr/Qwestr-API/graphs/contributors)
