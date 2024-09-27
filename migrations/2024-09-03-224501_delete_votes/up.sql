CREATE TABLE `delete_votes` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`confession_id` integer NOT NULL,
	`author_id` integer NOT NULL,
	`vote_type` text CHECK (`vote_type` IN ("delete_vote", "expose_vote") ) NOT NULL DEFAULT "delete_vote",
	`timestamp` timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
	FOREIGN KEY (`confession_id`) REFERENCES `confession`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`author_id`) REFERENCES `authors`(`id`) ON UPDATE no action ON DELETE no action
);
