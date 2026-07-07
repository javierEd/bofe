use chrono::{DateTime, NaiveDate, Utc};
use fake::Fake;
use fake::faker::chrono::en::DateTimeBefore;
use fake::faker::internet::en::{FreeEmail, Password, Username};
use fake::faker::name::en::Name;
use rand::rng;

use crate::commands;
use crate::enums::{CountryCode, LanguageCode};
use crate::models::User;
use crate::params::UserParams;

fn fake_birthdate() -> NaiveDate {
    DateTimeBefore(Utc::now()).fake::<DateTime<Utc>>().date_naive()
}

fn fake_country_code() -> CountryCode {
    CountryCode::VE
}

pub fn fake_email() -> String {
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

pub fn fake_password() -> String {
    Password(6..129).fake()
}

pub fn fake_username() -> String {
    let mut username: String = Username().fake_with_rng(&mut rng());

    username.truncate(16);

    username
}

pub async fn insert_test_user<'a>(password: Option<&'_ str>) -> User<'a> {
    let username = fake_username();
    let email = fake_email();

    let password = if let Some(password) = password {
        password.to_owned()
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
        language_code: Some(language_code),
        country_code,
    })
    .await
    .expect("Could not insert user")
}
