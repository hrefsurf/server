/*
    "Allocated" users are users that are pre-authorized to sign up.
    We do this by giving users we want to join ahead of time a 
    username and a secret.

    The secret is not securely stored (no real purpose for it to be),
    and is only used temporarily. Signup code should check for conflicts
    in this table.

    We'll let the user provide an email, password of their own when they
    go to sign up.
 */

#[derive(sqlx::FromRow)]
pub struct AllocatedUser {
    pub username: String,
    pub secret: String,
}
