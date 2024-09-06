CREATE TABLE `confession` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`guild_id` text NOT NULL,
	`message_id` text NOT NULL,
	`content` text NOT NULL,
	`author` integer NOT NULL,
	`timestamp` timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
	FOREIGN KEY (`guild_id`) REFERENCES `guild`(`guild_id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`author`) REFERENCES `authors`(`id`) ON UPDATE no action ON DELETE no action
);
