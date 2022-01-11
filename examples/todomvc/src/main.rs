#[macro_use]
extern crate derive_more;

use model::{Filter, TodoApp};
use silkenweb::{dom::mount, router::url};
use view::TodoAppView;

mod model;
mod view;

fn main() {
    console_error_panic_hook::set_once();

    let item_filter = url().signal_ref({
        |url| match url.hash().as_str() {
            "#/active" => Filter::Active,
            "#/completed" => Filter::Completed,
            _ => Filter::All,
        }
    });

    mount("app", TodoAppView::new(TodoApp::load()).render(item_filter));
}
