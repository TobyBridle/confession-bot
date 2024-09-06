CREATE TABLE `guild` (
    `guild_id` text PRIMARY KEY NOT NULL,
	`confession_channel_id` text,
	`config` text NOT NULL,
	`timestamp` timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL
);
