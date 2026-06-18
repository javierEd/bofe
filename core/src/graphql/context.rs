use std::net::IpAddr;

use async_graphql::Context;

use crate::L10n;
use crate::models::{Application, Session, User};

pub trait CustomExt {
    fn application(&self) -> &Application<'_>;

    fn client_ip(&self) -> &IpAddr;

    fn l10n(&self) -> &L10n;

    fn session(&self) -> &Session<'_>;

    #[allow(dead_code)]
    fn session_opt(&self) -> Option<&Session<'_>>;

    fn user(&self) -> &User<'_>;

    fn user_opt(&self) -> Option<&User<'_>>;
}

impl CustomExt for Context<'_> {
    fn application(&self) -> &Application<'_> {
        self.data_unchecked::<Application<'_>>()
    }

    fn client_ip(&self) -> &IpAddr {
        self.data_unchecked::<IpAddr>()
    }

    fn l10n(&self) -> &L10n {
        self.data_unchecked::<L10n>()
    }

    fn session(&self) -> &Session<'_> {
        self.data_unchecked::<Session<'_>>()
    }

    fn session_opt(&self) -> Option<&Session<'_>> {
        self.data_opt::<Session<'_>>()
    }

    fn user(&self) -> &User<'_> {
        self.data_unchecked::<User<'_>>()
    }

    fn user_opt(&self) -> Option<&User<'_>> {
        self.data_opt::<User<'_>>()
    }
}
