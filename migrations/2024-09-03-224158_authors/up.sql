CREATE TABLE `authors`(
    `id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
    `hash` text UNIQUE NOT NULL
);
