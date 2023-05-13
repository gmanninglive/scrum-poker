use handlebars::Handlebars;
use std::path::PathBuf;

pub fn init() -> handlebars::Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    let base_path = PathBuf::from("templates");

    handlebars
        .register_template_file("layout", base_path.join("layout.hbs"))
        .unwrap();
    handlebars
        .register_template_file("index", base_path.join("index.hbs"))
        .unwrap();
    handlebars
        .register_template_file("session_form", base_path.join("session/_form.hbs"))
        .unwrap();
    handlebars
        .register_template_file("session_show", base_path.join("session/show.hbs"))
        .unwrap();

    handlebars
}
