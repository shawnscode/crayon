//! Uniqued resource location.

use inlinable_string::InlinableString;

use crate::utils::hash::FastHashMap;

const SCHEMA: usize = 0;
const SCHEMA_END: usize = 1;
const USERNAME: usize = 2;
const USERNAME_END: usize = 3;
const PASSWORD: usize = 4;
const PASSWORD_END: usize = 5;
const HOST: usize = 6;
const HOST_END: usize = 7;
const PORT: usize = 8;
const PORT_END: usize = 9;
const PATH: usize = 10;
const PATH_END: usize = 11;
const QUERY: usize = 12;
const QUERY_END: usize = 13;
const FRAGMENT: usize = 14;
const FRAGMENT_END: usize = 15;
const MAX_COMPONENTS: usize = 16;

/// All resource paths in crayon are expressed as URLs. On creation, the URL will be
/// parsed and indices to its parts will be stored internally.
#[derive(Debug, Clone)]
pub struct Url {
    url: String,
    components: [usize; MAX_COMPONENTS],
}

impl Url {
    /// Creates a new URL.
    pub fn new<T: Into<String>>(url: T) -> Result<Self, failure::Error> {
        let url = url.into();
        let mut components = [0; MAX_COMPONENTS];

        let schema_index = url
            .find("://")
            .ok_or_else(|| format_err!("URL({}) must have a schema!", url))?;

        components[SCHEMA] = 0;
        components[SCHEMA_END] = schema_index;

        unsafe {
            let mut iter = schema_index + 3;
            let mut iter_end = iter + url
                .get_unchecked(iter..)
                .find('/')
                .ok_or_else(|| format_err!("URL({}) must have a hostname!", url))?;

            if let Some(info_end_index) = url.get_unchecked(iter..iter_end).find('@') {
                let info_end_index = info_end_index + iter;
                components[USERNAME] = iter;

                // extract user and password
                if let Some(user_end_index) = url.get_unchecked(iter..info_end_index).find(':') {
                    let user_end_index = user_end_index + iter;

                    components[USERNAME_END] = user_end_index;
                    components[PASSWORD] = user_end_index + 1;
                    components[PASSWORD_END] = info_end_index;
                } else {
                    components[USERNAME_END] = info_end_index;
                }

                iter = info_end_index + 1;
            }

            components[HOST] = iter;
            if let Some(host_end_index) = url.get_unchecked(iter..iter_end).find(':') {
                let host_end_index = host_end_index + iter;

                components[HOST_END] = host_end_index;
                components[PORT] = host_end_index + 1;
                components[PORT_END] = iter_end;
            } else {
                components[HOST_END] = iter_end;
            }

            iter = iter_end + 1;
            iter_end = url.len();

            components[PATH] = iter - 1;
            components[PATH_END] = iter_end;

            // extract query
            if let Some(query_index) = url.get_unchecked(iter..).find('?') {
                let query_index = query_index + iter;

                components[PATH_END] = query_index;
                components[QUERY] = query_index + 1;
                components[QUERY_END] = iter_end;

                iter = query_index + 1;
            }

            // extract fragment
            if let Some(fragment_index) = url.get_unchecked(iter..).find('#') {
                let fragment_index = fragment_index + iter;

                if components[QUERY_END] == 0 {
                    components[PATH_END] = fragment_index;
                } else {
                    components[QUERY_END] = fragment_index;
                }

                components[FRAGMENT] = fragment_index + 1;
                components[FRAGMENT_END] = iter_end;
            }

            if components[PATH_END] <= components[PATH] {
                bail!("The path of URL({}) could not be empty!", url);
            }
        }

        Ok(Url { url, components })
    }

    /// Get the queries of this URL if exists.
    pub fn queries(&self) -> Option<FastHashMap<InlinableString, Option<InlinableString>>> {
        if self.components[QUERY_END] > self.components[QUERY] {
            let mut queries = FastHashMap::default();
            let (mut lhs, rhs) = (self.components[QUERY], self.components[QUERY_END]);

            unsafe {
                while rhs > lhs {
                    let end = self
                        .url
                        .get_unchecked(lhs..rhs)
                        .find('&')
                        .map(|v| v + lhs)
                        .unwrap_or(rhs);

                    if let Some(eq_index) = self.url.get_unchecked(lhs..end).find('=') {
                        let eq_index = eq_index + lhs;

                        if (eq_index > lhs) && (end > (eq_index + 1)) {
                            let k = self.url.get_unchecked(lhs..eq_index);
                            let v = self.url.get_unchecked((eq_index + 1)..end);
                            queries.insert(k.into(), Some(v.into()));
                        }
                    } else if end > lhs {
                        let k = self.url.get_unchecked(lhs..end);
                        queries.insert(k.into(), None);
                    }

                    lhs = end + 1;
                }
            }

            Some(queries)
        } else {
            None
        }
    }
}

impl std::ops::Deref for Url {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

macro_rules! field {
    ($name: ident, $start: ident, $end: ident) => {
        #[inline]
        pub fn $name(&self) -> &str {
            unsafe {
                self.url
                    .get_unchecked(self.components[$start]..self.components[$end])
            }
        }
    };
}

macro_rules! optional_field {
    ($name: ident, $start: ident, $end: ident) => {
        #[inline]
        pub fn $name(&self) -> Option<&str> {
            unsafe {
                if self.components[$end] > self.components[$start] {
                    Some(
                        self.url
                            .get_unchecked(self.components[$start]..self.components[$end]),
                    )
                } else {
                    None
                }
            }
        }
    };
}

impl Url {
    field!(schema, SCHEMA, SCHEMA_END);
    field!(host, HOST, HOST_END);
    field!(path, PATH, PATH_END);

    optional_field!(username, USERNAME, USERNAME_END);
    optional_field!(password, PASSWORD, PASSWORD_END);
    optional_field!(port, PORT, PORT_END);
    optional_field!(fragment, FRAGMENT, FRAGMENT_END);
    optional_field!(query, QUERY, QUERY_END);
}
