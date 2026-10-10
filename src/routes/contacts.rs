//! Forms: validation errors, error bags, flash messages and redirects.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use inertia::{Inertia, ValidationErrors, props};
use serde::Deserialize;

use super::errors::not_found_page;
use crate::db::Db;

const ROLES: [&str; 5] = ["Engineer", "Designer", "Founder", "Product", "Support"];

#[derive(Deserialize)]
pub struct Filters {
    #[serde(default)]
    search: String,
}

/// The list. The search box reloads only the `contacts` and `filters` props.
pub async fn index(
    inertia: Inertia,
    State(db): State<Db>,
    Query(filters): Query<Filters>,
) -> inertia::Response {
    inertia.render(
        "Contacts/Index",
        props! {
            "filters" => props! { "search" => filters.search.clone() },
            "contacts" => db.contacts(&filters.search),
        },
    )
}

pub async fn show(inertia: Inertia, State(db): State<Db>, Path(id): Path<u64>) -> Response {
    match db.contact(id) {
        Some(contact) => inertia
            .render("Contacts/Show", props! { "contact" => contact })
            .into_response(),
        None => not_found_page(&inertia),
    }
}

pub async fn create(inertia: Inertia) -> inertia::Response {
    inertia.render("Contacts/Create", props! { "roles" => ROLES })
}

pub async fn edit(inertia: Inertia, State(db): State<Db>, Path(id): Path<u64>) -> Response {
    match db.contact(id) {
        Some(contact) => inertia
            .render(
                "Contacts/Edit",
                props! { "contact" => contact, "roles" => ROLES },
            )
            .into_response(),
        None => not_found_page(&inertia),
    }
}

#[derive(Deserialize)]
pub struct ContactForm {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    company: String,
    #[serde(default)]
    role: String,
}

fn validate(db: &Db, form: &ContactForm, except: Option<u64>, full: bool) -> ValidationErrors {
    let mut errors = ValidationErrors::new();

    let name = form.name.trim();
    if name.is_empty() {
        errors.add("name", "The name field is required.");
    } else if name.chars().count() < 3 {
        errors.add("name", "The name must be at least 3 characters.");
    }

    let email = form.email.trim();
    if email.is_empty() {
        errors.add("email", "The email field is required.");
    } else if !is_email(email) {
        errors.add("email", "The email must be a valid email address.");
    } else if db.email_taken(email, except) {
        errors.add("email", "That email address is already taken.");
    }

    if full {
        if form.company.trim().is_empty() {
            errors.add("company", "The company field is required.");
        }
        if !ROLES.contains(&form.role.as_str()) {
            errors.add("role", "Pick one of the listed roles.");
        }
    }

    errors
}

fn is_email(email: &str) -> bool {
    match email.split_once('@') {
        Some((user, domain)) => {
            !user.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
        }
        None => false,
    }
}

/// `POST /contacts`, from the `<Form>` component on the create page.
pub async fn store(
    inertia: Inertia,
    State(db): State<Db>,
    Json(form): Json<ContactForm>,
) -> Response {
    let errors = validate(&db, &form, None, true);
    if !errors.is_empty() {
        // Back to the form; the errors arrive in its `errors` prop.
        return inertia.back_with_errors(errors).into_response();
    }

    let contact = db.create_contact(&form.name, &form.email, &form.company, &form.role);
    inertia.flash("toast", format!("{} was added.", contact.name));

    inertia::redirect("/contacts").into_response()
}

/// `POST /contacts/quick`, from the quick-add form on the list. That form
/// sets an `errorBag`, so its errors are scoped under `errors.quickAdd` and
/// don't collide with any other form on the page.
pub async fn quick_store(
    inertia: Inertia,
    State(db): State<Db>,
    Json(form): Json<ContactForm>,
) -> Response {
    let errors = validate(&db, &form, None, false);
    if !errors.is_empty() {
        return inertia.back_with_errors(errors).into_response();
    }

    let contact = db.create_contact(&form.name, &form.email, "Independent", "Engineer");
    inertia.flash("toast", format!("{} was quick-added.", contact.name));

    inertia.back().into_response()
}

/// `PUT /contacts/{id}`, from `useForm` on the edit page. The redirect after a
/// `PUT` becomes a `303 See Other`, so the browser follows it with a `GET`.
pub async fn update(
    inertia: Inertia,
    State(db): State<Db>,
    Path(id): Path<u64>,
    Json(form): Json<ContactForm>,
) -> Response {
    if db.contact(id).is_none() {
        return not_found_page(&inertia);
    }

    let errors = validate(&db, &form, Some(id), true);
    if !errors.is_empty() {
        return inertia.back_with_errors(errors).into_response();
    }

    db.update_contact(id, &form.name, &form.email, &form.company, &form.role);
    inertia.flash("toast", "Contact saved.");

    inertia::redirect(&format!("/contacts/{id}")).into_response()
}

/// `DELETE /contacts/{id}`: also a `303` back to the list.
pub async fn destroy(inertia: Inertia, State(db): State<Db>, Path(id): Path<u64>) -> Response {
    match db.delete_contact(id) {
        Some(contact) => {
            inertia.flash("toast", format!("{} was deleted.", contact.name));
            inertia::redirect("/contacts").into_response()
        }
        None => not_found_page(&inertia),
    }
}
