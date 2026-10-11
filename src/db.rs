//! An in-memory "database", seeded on startup, so the demo has no setup.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Contact {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub company: String,
    pub role: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Post {
    pub id: u64,
    pub author: String,
    pub title: String,
    pub body: String,
    pub likes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub score: u64,
    pub updated: u64,
}

/// A signed-in user, as the pages see it.
#[derive(Clone, Debug, Serialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

/// A user and their password hash, which never leaves the server.
#[derive(Clone, Debug)]
pub struct Account {
    pub user: User,
    pub password_hash: String,
}

/// Failed sign-ins allowed per email before it's locked for a while.
pub const MAX_LOGIN_ATTEMPTS: u32 = 5;
pub const LOGIN_LOCKOUT: Duration = Duration::from_secs(60);

#[derive(Default)]
struct Tables {
    accounts: Vec<Account>,
    next_user_id: u64,
    /// Failed sign-ins by email: how many, and when the first one was.
    login_failures: HashMap<String, (u32, Instant)>,
    contacts: Vec<Contact>,
    next_contact_id: u64,
    posts: Vec<Post>,
    players: Vec<Player>,
    rng: u64,
}

/// Shared application state.
#[derive(Clone)]
pub struct Db(Arc<Mutex<Tables>>);

const FIRST: [&str; 16] = [
    "Ada",
    "Grace",
    "Linus",
    "Ferris",
    "Margaret",
    "Dennis",
    "Barbara",
    "Ken",
    "Radia",
    "Guido",
    "Yukihiro",
    "Frances",
    "Bjarne",
    "Hedy",
    "Alan",
    "Katherine",
];
const LAST: [&str; 12] = [
    "Lovelace",
    "Hopper",
    "Torvalds",
    "Crab",
    "Hamilton",
    "Ritchie",
    "Liskov",
    "Thompson",
    "Perlman",
    "Rossum",
    "Matsumoto",
    "Allen",
];
const COMPANIES: [&str; 6] = [
    "Rustacean Labs",
    "Tokio Works",
    "Serde & Co",
    "Hyperion",
    "Tower Systems",
    "Crates Inc",
];
const ROLES: [&str; 5] = ["Engineer", "Designer", "Founder", "Product", "Support"];
const TOPICS: [&str; 10] = [
    "ownership",
    "borrowing",
    "lifetimes",
    "async",
    "traits",
    "macros",
    "generics",
    "pattern matching",
    "iterators",
    "error handling",
];

impl Db {
    pub fn seeded() -> Self {
        let mut tables = Tables {
            rng: now_millis() | 1,
            ..Tables::default()
        };

        tables.accounts.push(Account {
            user: User {
                id: 1,
                name: crate::auth::DEMO_NAME.to_owned(),
                email: crate::auth::DEMO_EMAIL.to_owned(),
            },
            password_hash: crate::auth::hash_password(crate::auth::DEMO_PASSWORD),
        });
        tables.next_user_id = 2;

        for i in 0..24u64 {
            let first = FIRST[(i as usize) % FIRST.len()];
            let last = LAST[(i as usize * 7) % LAST.len()];
            tables.contacts.push(Contact {
                id: i + 1,
                name: format!("{first} {last}"),
                email: format!(
                    "{}.{}@example.com",
                    first.to_lowercase(),
                    last.to_lowercase()
                ),
                company: COMPANIES[(i as usize * 5) % COMPANIES.len()].to_owned(),
                role: ROLES[(i as usize * 3) % ROLES.len()].to_owned(),
            });
        }
        tables.next_contact_id = 25;

        for i in 0..150u64 {
            let topic = TOPICS[(i as usize) % TOPICS.len()];
            tables.posts.push(Post {
                id: i + 1,
                author: format!("{} {}", FIRST[(i as usize * 3) % FIRST.len()], LAST[(i as usize) % LAST.len()]),
                title: format!("Post #{}: thoughts on {topic}", i + 1),
                body: format!(
                    "A short note about {topic}. Scroll down and the next page is fetched with a partial reload and merged into the list."
                ),
                likes: (i * 37) % 120,
            });
        }

        for i in 0..40u64 {
            tables.players.push(Player {
                id: i + 1,
                name: format!(
                    "{} {}",
                    FIRST[(i as usize * 5) % FIRST.len()],
                    LAST[(i as usize * 3) % LAST.len()]
                ),
                score: 1000 - i * 20,
                updated: 0,
            });
        }

        Self(Arc::new(Mutex::new(tables)))
    }

    fn lock(&self) -> MutexGuard<'_, Tables> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    // Users

    pub fn user(&self, id: u64) -> Option<User> {
        self.lock()
            .accounts
            .iter()
            .find(|account| account.user.id == id)
            .map(|account| account.user.clone())
    }

    /// Emails are stored lowercased, so callers pass a normalized one.
    pub fn account_by_email(&self, email: &str) -> Option<Account> {
        self.lock()
            .accounts
            .iter()
            .find(|account| account.user.email == email)
            .cloned()
    }

    /// `None` if the email was taken in the meantime.
    pub fn create_user(&self, name: &str, email: &str, password_hash: String) -> Option<User> {
        let mut tables = self.lock();
        if tables
            .accounts
            .iter()
            .any(|account| account.user.email == email)
        {
            return None;
        }
        let user = User {
            id: tables.next_user_id,
            name: name.trim().to_owned(),
            email: email.to_owned(),
        };
        tables.next_user_id += 1;
        tables.accounts.push(Account {
            user: user.clone(),
            password_hash,
        });
        Some(user)
    }

    // Sign-in throttling

    /// How long the email is locked out for, if it has too many failures.
    pub fn login_lockout(&self, email: &str) -> Option<Duration> {
        let tables = self.lock();
        let (count, since) = tables.login_failures.get(email)?;
        let remaining = LOGIN_LOCKOUT.checked_sub(since.elapsed())?;
        (*count >= MAX_LOGIN_ATTEMPTS && !remaining.is_zero()).then_some(remaining)
    }

    pub fn record_login_failure(&self, email: &str) {
        let mut tables = self.lock();
        // Forget stale entries, so made-up emails can't grow the map forever.
        tables
            .login_failures
            .retain(|_, (_, since)| since.elapsed() < LOGIN_LOCKOUT);
        let entry = tables
            .login_failures
            .entry(email.to_owned())
            .or_insert((0, Instant::now()));
        entry.0 += 1;
    }

    pub fn clear_login_failures(&self, email: &str) {
        self.lock().login_failures.remove(email);
    }

    // Contacts

    pub fn contacts(&self, search: &str) -> Vec<Contact> {
        let search = search.trim().to_lowercase();
        self.lock()
            .contacts
            .iter()
            .filter(|contact| {
                search.is_empty()
                    || contact.name.to_lowercase().contains(&search)
                    || contact.email.to_lowercase().contains(&search)
                    || contact.company.to_lowercase().contains(&search)
            })
            .cloned()
            .collect()
    }

    pub fn contact_count(&self) -> usize {
        self.lock().contacts.len()
    }

    pub fn contact(&self, id: u64) -> Option<Contact> {
        self.lock()
            .contacts
            .iter()
            .find(|contact| contact.id == id)
            .cloned()
    }

    pub fn email_taken(&self, email: &str, except: Option<u64>) -> bool {
        self.lock().contacts.iter().any(|contact| {
            contact.email.eq_ignore_ascii_case(email.trim()) && Some(contact.id) != except
        })
    }

    pub fn create_contact(&self, name: &str, email: &str, company: &str, role: &str) -> Contact {
        let mut tables = self.lock();
        let contact = Contact {
            id: tables.next_contact_id,
            name: name.trim().to_owned(),
            email: email.trim().to_owned(),
            company: company.trim().to_owned(),
            role: role.trim().to_owned(),
        };
        tables.next_contact_id += 1;
        tables.contacts.insert(0, contact.clone());
        contact
    }

    pub fn update_contact(
        &self,
        id: u64,
        name: &str,
        email: &str,
        company: &str,
        role: &str,
    ) -> bool {
        let mut tables = self.lock();
        let Some(contact) = tables.contacts.iter_mut().find(|contact| contact.id == id) else {
            return false;
        };
        contact.name = name.trim().to_owned();
        contact.email = email.trim().to_owned();
        contact.company = company.trim().to_owned();
        contact.role = role.trim().to_owned();
        true
    }

    pub fn delete_contact(&self, id: u64) -> Option<Contact> {
        let mut tables = self.lock();
        let index = tables
            .contacts
            .iter()
            .position(|contact| contact.id == id)?;
        Some(tables.contacts.remove(index))
    }

    // Feed

    pub fn posts(&self) -> Vec<Post> {
        self.lock().posts.clone()
    }

    // Leaderboard

    pub fn players(&self) -> Vec<Player> {
        let mut players = self.lock().players.clone();
        players.sort_by(|a, b| b.score.cmp(&a.score).then(a.id.cmp(&b.id)));
        players
    }

    /// Give a handful of players new points, as if games had been played.
    pub fn play_round(&self) -> Vec<u64> {
        let mut tables = self.lock();
        let round = now_millis();
        let mut changed = Vec::new();
        for _ in 0..6 {
            let roll = tables.next_random();
            let index = (roll % tables.players.len() as u64) as usize;
            let points = 10 + (roll >> 16) % 90;
            let player = &mut tables.players[index];
            player.score += points;
            player.updated = round;
            changed.push(player.id);
        }
        changed
    }

    /// A pseudo-random number for live metrics.
    pub fn random(&self) -> u64 {
        self.lock().next_random()
    }
}

impl Tables {
    /// xorshift64: plenty for demo data, and no extra dependency.
    fn next_random(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }
}

pub fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
