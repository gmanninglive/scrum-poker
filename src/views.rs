use minijinja::Environment;

const TPL_INDEX: &str = include_str!("../templates/index.html");
const TPL_LAYOUT: &str = include_str!("../templates/layout.html");
const TPL_SESSION_SHOW: &str = include_str!("../templates/session.html");
const TPL_404: &str = include_str!("../templates/404.html");

pub fn init() -> Environment<'static> {
    let mut minijinja_env = Environment::new();
    minijinja_env
        .add_template("404", TPL_404)
        .expect("could not add template");
    minijinja_env
        .add_template("index", TPL_INDEX)
        .expect("could not find template");
    minijinja_env
        .add_template("layout", TPL_LAYOUT)
        .expect("could not find template");
    minijinja_env
        .add_template("session_show", TPL_SESSION_SHOW)
        .expect("could not find template");

    minijinja_env
}
