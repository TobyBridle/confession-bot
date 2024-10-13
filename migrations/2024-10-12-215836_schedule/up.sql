CREATE TABLE schedule (
    `id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
    `guild_id` text NOT NULL,
    `victim_id` text NOT NULL,
    `ends_at` integer NOT NULL,
    `start_at` integer NOT NULL
);
