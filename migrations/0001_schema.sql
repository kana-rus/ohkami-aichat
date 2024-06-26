-- Migration number: 0001 	 2024-06-12T08:39:45.467Z

CREATE TABLE IF NOT EXISTS chats (
    id    TEXT NOT NULL PRIMARY KEY, -- uuid v4
    title TEXT
);

CREATE TABLE IF NOT EXISTS messages (
    id               INTEGER NOT NULL PRIMARY KEY,
    chat_id          TEXT    NOT NULL,
    role_id          INTEGER NOT NULL, -- 0: "system", 1: "user", 2: "assistant"
    content          TEXT    NOT NULL,
    finish_reason_id INTEGER, -- 0: "stop", 1: "length", 2: "content_filter"

    FOREIGN KEY chat_id REFERENCES chats (id)
);

CREATE TABLE IF NOT EXISTS message_branches (
    id         INTEGER NOT NULL PRIMARY KEY,
    message_id INTEGER NOT NULL,
    branch_id  INTEGER NOT NULL,

    FOREIGN KEY message_id REFERENCES messages (id)
);
