use chrono::{DateTime, Utc};
use fake::faker::chrono::en::DateTimeBefore;
use fake::faker::internet::en::{FreeEmail, Password, Username};
use fake::faker::name::en::Name;
use fake::{Fake, Faker};
use rand::rng;

use crate::commands;
use crate::enums::{CountryCode, LanguageCode};
use crate::params::UserParams;

fn fake_birthdate() -> String {
    DateTimeBefore(Utc::now())
        .fake::<DateTime<Utc>>()
        .date_naive()
        .to_string()
}

fn fake_country_code() -> CountryCode {
    CountryCode::VE
}

fn fake_email() -> String {
    FreeEmail().fake_with_rng(&mut rng())
}

fn fake_language_code() -> LanguageCode {
    LanguageCode::Es
}

pub fn fake_name() -> String {
    let mut name: String = Name().fake_with_rng(&mut rng());

    name.truncate(255);

    name
}

fn fake_password() -> String {
    Password(6..=128).fake()
}

pub fn fake_username() -> String {
    let mut username: String = Username().fake_with_rng(&mut rng());

    username.truncate(16);

    username
}

pub async fn insert_test_user(password: Option<String>) -> User {
    let username = fake_username();
    let email = fake_email();

    let password = if let Some(password) = password {
        password
    } else {
        fake_password()
    };

    let full_name = fake_name();
    let birthdate = fake_birthdate();
    let language_code = fake_language_code();
    let country_code = fake_country_code();

    commands::insert_user(UserParams {
        username,
        email,
        password,
        full_name,
        birthdate,
        language_code,
        country_code,
    })
    .await
    .expect("Could not insert user")
}
