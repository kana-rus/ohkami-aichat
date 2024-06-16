mod api;
mod models;

use ohkami::prelude::*;


#[ohkami::bindings]
struct Bindings;

#[ohkami::worker]
async fn my_worker() -> Ohkami {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let fangs = {
        #[cfg(debug_assertions)]
        ohkami::builtin::fang::CORS::new("http://127.0.0.1:8080")
    };

    Ohkami::with(fangs, (
        /* in production, `./dist` contents are served by `--assets dist` of `deploy` script in package.json */

        "/chats"
            .GET(api::list_chats)
            .POST(api::create_chat),
        "/chats/:chat_id"
            .PATCH(api::set_title),
        "/chats/:chat_id/:branch"
            .GET(api::load_messages)
            .POST(api::post_message),
        "/chats/:chat_id/:branch/regenerate"
            .GET(api::regenerate_response),
        "/messages/:message_id"
            .PUT(api::create_new_branch),
    ))
}
