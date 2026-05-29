# Bofe

Kanban style boards.

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
| DATABASE_MAX_CONNECTIONS     | Number  | 5                                               | api,monitor     |
| DATABASE_URL                 | String  | postgres://bofe:bofe@127.0.0.1:5432/bofe_dev    | api,monitor     |
| MAILER_ENABLE                | Boolean | false                                           | monitor         |
| MAILER_SENDER_ADDRESS        | String  | Bofe dev <no-reply@localhost>                   | monitor         |
| MAILER_SMTP_ADDRESS          | String  | localhost                                       | monitor         |
| MAILER_SMTP_PASSWORD         | String  |                                                 | monitor         |
| MAILER_SMTP_SECURITY         | String  | none                                            | monitor         |
| MAILER_SMTP_USERNAME         | String  |                                                 | monitor         |
| MAILER_SUPPORT_EMAIL_ADDRESS | String  | support@localhost                               | monitor         |
| MONITOR_REDIS_URL            | String  | redis://127.0.0.1:6379/1                        | api,cli,monitor |
| PUBSUB_REDIS_URL             | String  | redis://127.0.0.1:6379/2                        | api,monitor     |
| SESSION_TTL_SECS             | Number  | 2592000                                         | api             |
| SESSION_TOKEN_MIN_LENGTH     | Number  | 64                                              | api             |
| SESSION_TOKEN_MAX_LENGTH     | Number  | 128                                             | api             |
| STORAGE_FONT_PATH            | String  | /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf | api             |
| STORAGE_PATH                 | String  | ./storage/                                      | api,monitor     |
| STORAGE_URL                  | String  | http://127.0.0.1:8005/storage/                  | api             |
