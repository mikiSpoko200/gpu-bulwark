use std::marker::PhantomData;
use std::net::Ipv4Addr;
use thiserror::Error;

pub trait AuthenticationStatus {
    type Ctx: Sized;
}

pub struct Verified;
pub struct Anonymous;

// There is no extra state associated with an anonymous in user.
impl AuthenticationStatus for Anonymous {
    type Ctx = ();
}

// Is user is logged in User data can be obtained.
impl AuthenticationStatus for Verified {
    type Ctx = User;
}

// Representation of a user
pub struct User {
    pub user_name: String,
    /* ... */
}

// Representation of a TCP connection. It is important for this connection to
// remain undisturbed, and this object not to be dropped throughout user session.
struct Connection(Ipv4Addr);

impl Connection {
    pub fn new(client: Ipv4Addr) -> Self {
        println!("connection established with: {:?}", &client);
        Self(client)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        println!("connection closed with: {:?}", &self.0);
    }
}

struct Session<AS>
where
    AS: AuthenticationStatus,
{
    type_state: PhantomData<AS>,
    connection: Connection,
    ctx: AS::Ctx,
}

impl Session<Anonymous> {
    /// We allow to create only anonymous sessions.
    fn new(ip: Ipv4Addr) -> Self {
        Self {
            type_state: PhantomData,
            connection: Connection::new(ip),
            ctx: (),
        }
    }
}

impl Session<Verified> {
    /// We provide access to confidential information for verified connections.
    fn secret(&self) -> String {
        "'a secret'".into()
    }
}

// Here we emulate authentication mechanism.

#[derive(Error, Debug)]
enum LoginError {
    #[error("invalid password for {user_name}: {password}")]
    InvalidPassword { user_name: String, password: String },
}

fn db_check_password(user: &User, password: &str) -> Result<(), LoginError> {
    (password == "password")
        .then_some(())
        .ok_or(LoginError::InvalidPassword {
            user_name: user.user_name.clone(),
            password: password.into(),
        })
}

impl Session<Anonymous> {
    /// We provide ability to log in only for anonymous users.
    pub fn log_in(self, user: User, password: &str) -> Result<Session<Verified>, LoginError> {
        db_check_password(&user, password).map(|_| Session {
            type_state: PhantomData,
            connection: self.connection,
            ctx: user,
        })
    }
}

/// Ability for different session types to display differently formatted main pages
/// depending on the type of the session.
pub trait MainPage {
    fn main_page(&self) -> String;
}

/// Anonymous sessions get generic main page.
impl MainPage for Session<Anonymous> {
    fn main_page(&self) -> String {
        format!("welcome anonymous at {:?}!", self.connection.0)
    }
}

/// Logged-inm sessions get main page with their name.
impl MainPage for Session<Verified> {
    fn main_page(&self) -> String {
        format!("welcome {} at {:?}!", self.ctx.user_name, self.connection.0)
    }
}

fn main() {
    // note: we use code blocks for to control lexical scope of variables.

    // type state enforces that user is logged in before it can access secret information.
    // This invariant is guaranteed at compile-time.
    // However, if the actual transition succeeds is determined at runtime.
    // We statically enforced a sequence of operations, and prevented
    // function invocation sequence which is semantically incorrect.
    {
        let anonymous_session = Session::new(Ipv4Addr::new(127, 0, 0, 1));

        // This method is statically dispatched to `impl MainPage for Session<Anonymous>`
        println!("{}", anonymous_session.main_page());

        let user_ok = User {
            user_name: "Mikołaj Depta".into(),
        };

        let logged_in_session = match anonymous_session.log_in(user_ok, "password") {
            Ok(ok) => ok,
            Err(err) => {
                println!("failed to log in: {}", err);
                std::process::exit(0);
            }
        };
        println!("logged in successfully!");

        // This method is statically dispatched to `impl MainPage for Session<Verified>`
        println!("{}", logged_in_session.main_page());
        println!("obtained {}", logged_in_session.secret());
    }

    {
        let _anonymous_session = Session::new(Ipv4Addr::new(127, 0, 0, 1));
        // uncommenting the line below will cause a compile-time error.
        // let unauthorized_access_to_secret = _anonymous_session.secret();
    }
}
