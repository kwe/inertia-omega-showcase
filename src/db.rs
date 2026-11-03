//! An in-memory "database", seeded on startup, so the demo has no setup.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::broadcast::Broadcaster;

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

/// One "Play a round" on the leaderboard.
#[derive(Clone, Debug, Serialize)]
pub struct Round {
    pub id: u64,
    pub at: u64,
    pub points: u64,
    /// The name of the player who scored the most this round.
    pub best: String,
}

/// A stock on the broadcasts page's ticker.
#[derive(Clone, Debug, Serialize)]
pub struct Quote {
    pub symbol: &'static str,
    /// In cents, so the demo needs no decimals.
    pub price: u64,
    /// In cents, since the previous tick.
    pub change: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatMessage {
    pub id: u64,
    pub author: String,
    pub body: String,
    pub at: u64,
}

/// Chat messages kept, newest last.
pub const MAX_MESSAGES: usize = 12;

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
    latest_round: Option<Round>,
    quotes: Vec<Quote>,
    ticks: u64,
    messages: Vec<ChatMessage>,
    next_message_id: u64,
    /// When each user was nudged, newest last.
    nudges: HashMap<u64, Vec<u64>>,
    /// Users with a nudge on its way, so a user can't queue up many.
    pending_nudges: HashSet<u64>,
    rng: u64,
}

/// Shared application state: the tables, and the broadcaster for live props.
#[derive(Clone)]
pub struct Db(Arc<Mutex<Tables>>, Broadcaster);

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

        for (symbol, price) in [
            ("FERR", 4_200),
            ("CRAB", 1_337),
            ("RUST", 9_900),
            ("AXUM", 2_750),
        ] {
            tables.quotes.push(Quote {
                symbol,
                price,
                change: 0,
            });
        }

        for (author, body) in [
            (
                "Ada Lovelace",
                "Open this page in a second tab, then post here.",
            ),
            (
                "Grace Hopper",
                "The other tab gets the message without a reload.",
            ),
        ] {
            tables.next_message_id += 1;
            tables.messages.push(ChatMessage {
                id: tables.next_message_id,
                author: author.to_owned(),
                body: body.to_owned(),
                at: now_millis(),
            });
        }

        Self(Arc::new(Mutex::new(tables)), Broadcaster::default())
    }

    pub fn broadcaster(&self) -> &Broadcaster {
        &self.1
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

    /// Counts a sign-in attempt against the email before its password is
    /// checked, or says how long it's locked out for. Checking and counting
    /// under one lock means a burst of concurrent attempts can't all slip in
    /// before any of them is counted. A successful sign-in clears the count.
    pub fn begin_login_attempt(&self, email: &str) -> Result<(), Duration> {
        let mut tables = self.lock();
        // Forget stale entries, so made-up emails can't grow the map forever.
        tables
            .login_failures
            .retain(|_, (_, since)| since.elapsed() < LOGIN_LOCKOUT);
        let (count, since) = tables
            .login_failures
            .entry(email.to_owned())
            .or_insert((0, Instant::now()));
        if *count >= MAX_LOGIN_ATTEMPTS {
            return Err(LOGIN_LOCKOUT.saturating_sub(since.elapsed()));
        }
        *count += 1;
        Ok(())
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
        let mut scored: HashMap<u64, (u64, String)> = HashMap::new();
        for _ in 0..6 {
            let roll = tables.next_random();
            let index = (roll % tables.players.len() as u64) as usize;
            let points = 10 + (roll >> 16) % 90;
            let player = &mut tables.players[index];
            player.score += points;
            player.updated = round;
            changed.push(player.id);
            scored
                .entry(player.id)
                .or_insert_with(|| (0, player.name.clone()))
                .0 += points;
        }
        // A player can score more than once in a round, so the best is the
        // one with the most points in total. Ties go to the lowest id.
        let total = scored.values().map(|(points, _)| points).sum();
        let best = scored
            .into_iter()
            .max_by(|(a_id, (a, _)), (b_id, (b, _))| a.cmp(b).then(b_id.cmp(a_id)))
            .map(|(_, (_, name))| name)
            .unwrap_or_default();
        let id = tables.latest_round.as_ref().map_or(1, |round| round.id + 1);
        tables.latest_round = Some(Round {
            id,
            at: round,
            points: total,
            best,
        });
        changed
    }

    pub fn latest_round(&self) -> Option<Round> {
        self.lock().latest_round.clone()
    }

    // Broadcasts

    pub fn market(&self) -> (u64, Vec<Quote>) {
        let tables = self.lock();
        (tables.ticks, tables.quotes.clone())
    }

    /// Move every price a little, as a market feed would.
    pub fn tick_market(&self) -> u64 {
        let mut tables = self.lock();
        tables.ticks += 1;
        for index in 0..tables.quotes.len() {
            let roll = tables.next_random();
            let quote = &mut tables.quotes[index];
            let change = (roll % 101) as i64 - 50;
            let price = quote.price.saturating_add_signed(change).max(100);
            quote.change = price as i64 - quote.price as i64;
            quote.price = price;
        }
        tables.ticks
    }

    pub fn messages(&self) -> Vec<ChatMessage> {
        self.lock().messages.clone()
    }

    /// Add a message, and hand the new list to `publish` before anyone else
    /// can post, so broadcasts of the list go out in the order it changed.
    pub fn post_message(&self, author: &str, body: &str, publish: impl FnOnce(&[ChatMessage])) {
        let mut tables = self.lock();
        tables.next_message_id += 1;
        let message = ChatMessage {
            id: tables.next_message_id,
            author: author.to_owned(),
            body: body.to_owned(),
            at: now_millis(),
        };
        tables.messages.push(message);
        let excess = tables.messages.len().saturating_sub(MAX_MESSAGES);
        tables.messages.drain(..excess);
        publish(&tables.messages);
    }

    pub fn nudges(&self, user: u64) -> Vec<u64> {
        self.lock().nudges.get(&user).cloned().unwrap_or_default()
    }

    /// Start a nudge, unless one is already on its way.
    pub fn begin_nudge(&self, user: u64) -> bool {
        self.lock().pending_nudges.insert(user)
    }

    pub fn finish_nudge(&self, user: u64) {
        let mut tables = self.lock();
        tables.pending_nudges.remove(&user);
        let nudges = tables.nudges.entry(user).or_default();
        nudges.push(now_millis());
        let excess = nudges.len().saturating_sub(5);
        nudges.drain(..excess);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attempts_are_counted_before_they_are_checked() {
        let db = Db::seeded();
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            assert!(db.begin_login_attempt("a@example.com").is_ok());
        }
        let wait = db.begin_login_attempt("a@example.com").unwrap_err();
        assert!(wait <= LOGIN_LOCKOUT && !wait.is_zero());
        assert!(db.begin_login_attempt("b@example.com").is_ok());

        db.clear_login_failures("a@example.com");
        assert!(db.begin_login_attempt("a@example.com").is_ok());
    }

    #[test]
    fn the_best_player_of_a_round_has_the_most_points_in_total() {
        let db = Db::seeded();
        for _ in 0..50 {
            let before: HashMap<u64, u64> = db.players().iter().map(|p| (p.id, p.score)).collect();
            db.play_round();
            let gains: Vec<(String, u64)> = db
                .players()
                .into_iter()
                .map(|p| (p.name, p.score - before[&p.id]))
                .collect();
            let round = db.latest_round().unwrap();
            let most = gains.iter().map(|(_, gain)| *gain).max().unwrap();
            assert_eq!(
                round.points,
                gains.iter().map(|(_, gain)| gain).sum::<u64>()
            );
            assert!(gains.contains(&(round.best.clone(), most)));
        }
    }
}
