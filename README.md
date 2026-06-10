<h1 align="center"><img height="36" src="https://about.bofe.app/icon.svg"/>&nbsp;<b>Bofe</b></h1>

<p align="center">Kanban style boards.</p>

<p align="center">
  <a href="https://github.com/javierEd/bofe/blob/main/LICENSE">
    <img
      src="https://img.shields.io/github/license/javierEd/bofe?logo=open-source-initiative&logoColor=white&style=flat-square"
      alt="license"
    /></a>
  <a href="https://github.com/javierEd/bofe/commits/main">
    <img
      src="https://img.shields.io/github/last-commit/javierEd/bofe?logo=git&logoColor=white&style=flat-square"
      alt="last commit"
    /></a>
  <a href="https://github.com/javierEd/bofe/actions/workflows/ci.yaml">
    <img
      src="https://img.shields.io/github/actions/workflow/status/javierEd/bofe/ci.yaml?label=CI&logo=github&style=flat-square"
      alt="CI"
    /></a>
  <a href="https://github.com/javierEd/bofe/actions/workflows/cd.yaml">
    <img
      src="https://img.shields.io/github/actions/workflow/status/javierEd/bofe/cd.yaml?label=CD&logo=github&style=flat-square"
      alt="CD"
    /></a>
  <a href="https://github.com/javierEd/bofe/network/dependencies">
    <img
      src="https://img.shields.io/deps-rs/repo/github/javierEd/bofe?logo=dependabot&logoColor=white&style=flat-square"
      alt="dependencies"
    /></a>
  <a href="https://github.com/javierEd/bofe/releases/latest">
    <img
      src="https://img.shields.io/github/v/release/javierEd/bofe?include_prereleases&logo=rocket&logoColor=white&style=flat-square"
      alt="release"
    /></a>
</p>

With **Bofe** you can easily build kanban style boards for your projects, personal workflows, or just for fun.

> [!NOTE]
> This repository contains the backend of the project.
> - To see the frontend (mobile/web) application, go to [github.com/javierEd/bofe_app](https://github.com/javierEd/bofe_app).

## Features

- Create and manage **Boards**.
- Add, rename and reorder **Lists**.
- Write **Cards** and move then between lists.
- Create **Labels** and assign them to cards.

## Build Requirements

- Rust 1.93.x
- PostgreSQL 18.x

## Run Requirements

- PostgreSQL 18.x
- Redis 8.x

## Environment variables

| Name                         | Type    | Default                                         | Packages        |
| ---------------------------- | ------- | ----------------------------------------------- | --------------- |
| API_ADDRESS                  | String  | 127.0.0.1:8005                                  | api             |
| API_CLIENT_IP_SOURCE         | String  | ConnectInfo                                     | api             |
| APPLICATION_TOKEN_MIN_LENGTH | Number  | 64                                              | api             |
| APPLICATION_TOKEN_MAX_LENGTH | Number  | 128                                             | api             |
| APPLICATION_TTL_SECS         | Number  | 31104000                                        | api             |
| CACHE_REDIS_URL              | String  | redis://127.0.0.1:6379/0                        | api,monitor     |
| CACHE_TTL_SECS               | Number  | 3600                                            | api,monitor     |
| CONFIRMATION_CODE_LENGTH     | Number  | 6                                               | api             |
| DATABASE_MAX_CONNECTIONS     | Number  | 5                                               | api,monitor     |
| DATABASE_URL                 | String  | postgres://bofe:bofe@127.0.0.1:5432/bofe_dev    | api,monitor     |
| IM_DATABASE_URL              | String  | redis://127.0.0.1:6379/2                        | api             |
| MAILER_ENABLE                | Boolean | false                                           | monitor         |
| MAILER_SENDER_ADDRESS        | String  | Bofe dev <no-reply@localhost>                   | monitor         |
| MAILER_SMTP_ADDRESS          | String  | localhost                                       | monitor         |
| MAILER_SMTP_PASSWORD         | String  |                                                 | monitor         |
| MAILER_SMTP_SECURITY         | String  | none                                            | monitor         |
| MAILER_SMTP_USERNAME         | String  |                                                 | monitor         |
| MAILER_SUPPORT_EMAIL_ADDRESS | String  | support@localhost                               | monitor         |
| MONITOR_REDIS_URL            | String  | redis://127.0.0.1:6379/1                        | api,cli,monitor |
| SESSION_TTL_SECS             | Number  | 2592000                                         | api             |
| SESSION_TOKEN_MIN_LENGTH     | Number  | 64                                              | api             |
| SESSION_TOKEN_MAX_LENGTH     | Number  | 128                                             | api             |
| STORAGE_FONT_PATH            | String  | /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf | api             |
| STORAGE_PATH                 | String  | ./storage/                                      | api,monitor     |
| STORAGE_URL                  | String  | http://127.0.0.1:8005/storage/                  | api             |

## License

This project is open-source and available under the GNU Affero General Public License v3.0 (AGPL v3). Please see the [LICENSE](https://github.com/javierEd/bofe/blob/main/LICENSE) file for more information.
